use crate::game::{BattleCard, GameEngine, PlayerView, SimpleRng};
use crate::ws::{
    accept_websocket, read_http_request, read_ws_message, write_http_response, write_ws_pong,
    write_ws_text, HttpRequest, WsMessage,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

const INDEX_HTML: &str = include_str!("../web/index.html");
const STYLES_CSS: &str = include_str!("../web/styles.css");
const APP_JS: &str = include_str!("../web/app.js");

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientCommand {
    UpdateName { display_name: String },
    StartGame,
    AcknowledgeDisguise,
    ProposeTeam { team_ids: Vec<String> },
    SubmitVote { approve: bool },
    AdvanceFromBallot,
    SubmitBattleAction { card: BattleCard },
    AdvanceFromBattleOutcome,
    SubmitUnmasking { target_id: String },
    PlayAgain,
    SendChat { text: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    State { view: Box<PlayerView> },
    Error { message: String },
}

struct RoomState {
    engine: GameEngine,
    connections: HashMap<String, mpsc::Sender<String>>,
}

impl RoomState {
    fn new(room_code: String) -> Self {
        Self {
            engine: GameEngine::new(room_code),
            connections: HashMap::new(),
        }
    }

    fn broadcast_personalized_views(&self) {
        for (player_id, tx) in &self.connections {
            if let Some(view) = self.engine.view_for_player(player_id) {
                let event = ServerEvent::State {
                    view: Box::new(view),
                };
                if let Ok(json) = serde_json::to_string(&event) {
                    let _ = tx.send(json);
                }
            }
        }
    }

    fn send_error(&self, player_id: &str, message: String) {
        if let Some(tx) = self.connections.get(player_id) {
            let event = ServerEvent::Error { message };
            if let Ok(json) = serde_json::to_string(&event) {
                let _ = tx.send(json);
            }
        }
    }
}

pub struct ServerState {
    rooms: Mutex<HashMap<String, RoomState>>,
    assets_dir: PathBuf,
}

impl ServerState {
    pub fn new(assets_dir: PathBuf) -> Self {
        Self {
            rooms: Mutex::new(HashMap::new()),
            assets_dir,
        }
    }
}

pub fn run_server(addr: &str, assets_dir: PathBuf) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    let state = Arc::new(ServerState::new(assets_dir));
    println!("Battle of Matsya: Shadows in the Court listening on http://{addr}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = Arc::clone(&state);
                thread::spawn(move || {
                    let _ = handle_connection(stream, state);
                });
            }
            Err(err) => {
                eprintln!("Accept error: {err}");
            }
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, state: Arc<ServerState>) -> std::io::Result<()> {
    let Some(req) = read_http_request(&stream)? else {
        return Ok(());
    };

    if is_websocket_upgrade(&req) && req.path == "/ws" {
        return handle_websocket(stream, req, state);
    }

    if req.method != "GET" {
        return write_http_response(
            &mut stream,
            "405 Method Not Allowed",
            "text/plain; charset=utf-8",
            b"Method Not Allowed",
        );
    }

    match req.path.as_str() {
        "/" | "/index.html" => write_http_response(
            &mut stream,
            "200 OK",
            "text/html; charset=utf-8",
            INDEX_HTML.as_bytes(),
        ),
        "/styles.css" => write_http_response(
            &mut stream,
            "200 OK",
            "text/css; charset=utf-8",
            STYLES_CSS.as_bytes(),
        ),
        "/app.js" => write_http_response(
            &mut stream,
            "200 OK",
            "application/javascript; charset=utf-8",
            APP_JS.as_bytes(),
        ),
        "/api/new-room" => {
            let code = create_unique_room_code(&state);
            let body = format!("{{\"room\":\"{code}\"}}");
            write_http_response(
                &mut stream,
                "200 OK",
                "application/json; charset=utf-8",
                body.as_bytes(),
            )
        }
        p if p.starts_with("/assets/") => serve_asset(&mut stream, &state.assets_dir, &p[8..]),
        _ => write_http_response(
            &mut stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            b"Not Found",
        ),
    }
}

fn is_websocket_upgrade(req: &HttpRequest) -> bool {
    req.headers
        .get("upgrade")
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}

fn serve_asset(stream: &mut TcpStream, assets_dir: &Path, rel: &str) -> std::io::Result<()> {
    let rel_path = Path::new(rel);
    for comp in rel_path.components() {
        match comp {
            Component::Normal(_) => {}
            _ => {
                return write_http_response(
                    stream,
                    "403 Forbidden",
                    "text/plain; charset=utf-8",
                    b"Forbidden",
                );
            }
        }
    }

    let full_path = assets_dir.join(rel_path);
    match fs::read(&full_path) {
        Ok(bytes) => {
            let content_type = match full_path.extension().and_then(|e| e.to_str()) {
                Some("png") => "image/png",
                Some("webp") => "image/webp",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("mp3") => "audio/mpeg",
                Some("svg") => "image/svg+xml",
                _ => "application/octet-stream",
            };
            write_http_response(stream, "200 OK", content_type, &bytes)
        }
        Err(_) => write_http_response(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            b"Asset Not Found",
        ),
    }
}

fn handle_websocket(
    mut stream: TcpStream,
    req: HttpRequest,
    state: Arc<ServerState>,
) -> std::io::Result<()> {
    accept_websocket(&mut stream, &req)?;

    let room_code = req
        .query
        .get("room")
        .map(|s| normalize_room_code(s))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "MATSYA".to_string());
    let player_id = req
        .query
        .get("playerId")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("guest_{}", current_millis()));
    let display_name = req
        .query
        .get("name")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Matsya Envoy".to_string());

    let (tx, rx) = mpsc::channel::<String>();
    {
        let mut rooms = state.rooms.lock().unwrap();
        let room = rooms
            .entry(room_code.clone())
            .or_insert_with(|| RoomState::new(room_code.clone()));

        match room
            .engine
            .add_or_reconnect_player(player_id.clone(), display_name)
        {
            Ok(()) => {
                room.connections.insert(player_id.clone(), tx.clone());
                room.broadcast_personalized_views();
            }
            Err(err) => {
                let msg = serde_json::to_string(&ServerEvent::Error { message: err })
                    .unwrap_or_default();
                let _ = write_ws_text(&mut stream, &msg);
                return Ok(());
            }
        }
    }

    let mut write_stream = stream.try_clone()?;
    thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            if write_ws_text(&mut write_stream, &msg).is_err() {
                break;
            }
        }
    });

    loop {
        match read_ws_message(&mut stream) {
            Ok(Some(WsMessage::Text(text))) => {
                process_client_message(&state, &room_code, &player_id, &text);
            }
            Ok(Some(WsMessage::Ping(payload))) => {
                if write_ws_pong(&mut stream, &payload).is_err() {
                    break;
                }
            }
            Ok(Some(WsMessage::Close)) | Ok(None) | Err(_) => break,
        }
    }

    let mut rooms = state.rooms.lock().unwrap();
    if let Some(room) = rooms.get_mut(&room_code) {
        room.connections.remove(&player_id);
        room.engine.set_connected(&player_id, false);
        room.broadcast_personalized_views();
    }

    Ok(())
}

fn process_client_message(state: &Arc<ServerState>, room_code: &str, player_id: &str, text: &str) {
    let cmd: ClientCommand = match serde_json::from_str(text) {
        Ok(c) => c,
        Err(e) => {
            let rooms = state.rooms.lock().unwrap();
            if let Some(room) = rooms.get(room_code) {
                room.send_error(player_id, format!("Invalid command: {e}"));
            }
            return;
        }
    };

    let mut rooms = state.rooms.lock().unwrap();
    let Some(room) = rooms.get_mut(room_code) else {
        return;
    };

    let result = match cmd {
        ClientCommand::UpdateName { display_name } => {
            room.engine.update_display_name(player_id, &display_name)
        }
        ClientCommand::StartGame => room.engine.start_game(player_id),
        ClientCommand::AcknowledgeDisguise => room.engine.acknowledge_disguise(player_id),
        ClientCommand::ProposeTeam { team_ids } => room.engine.propose_team(player_id, team_ids),
        ClientCommand::SubmitVote { approve } => room.engine.submit_vote(player_id, approve),
        ClientCommand::AdvanceFromBallot => room.engine.advance_from_ballot_reveal(player_id),
        ClientCommand::SubmitBattleAction { card } => {
            room.engine.submit_battle_action(player_id, card)
        }
        ClientCommand::AdvanceFromBattleOutcome => {
            room.engine.advance_from_battle_outcome(player_id)
        }
        ClientCommand::SubmitUnmasking { target_id } => {
            room.engine.submit_unmasking(player_id, &target_id)
        }
        ClientCommand::PlayAgain => room.engine.play_again(player_id),
        ClientCommand::SendChat { text } => {
            room.engine
                .add_chat_message(player_id, &text, current_millis())
        }
    };

    match result {
        Ok(()) => room.broadcast_personalized_views(),
        Err(err) => room.send_error(player_id, err),
    }
}

fn create_unique_room_code(state: &Arc<ServerState>) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = SimpleRng::from_entropy();
    let rooms = state.rooms.lock().unwrap();
    for _ in 0..64 {
        let mut code = String::with_capacity(4);
        for _ in 0..4 {
            let idx = rng.gen_range(ALPHABET.len());
            code.push(ALPHABET[idx] as char);
        }
        if !rooms.contains_key(&code) {
            return code;
        }
    }
    "MATS".to_string()
}

fn normalize_room_code(raw: &str) -> String {
    raw.trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(4)
        .collect::<String>()
        .to_ascii_uppercase()
}

fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
