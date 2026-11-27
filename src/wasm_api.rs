use crate::game::{BattleCard, GameEngine};
use serde::Deserialize;
use std::cell::RefCell;
use std::slice;

thread_local! {
    static ENGINE: RefCell<Option<GameEngine>> = const { RefCell::new(None) };
    static LAST_OUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WasmCommand {
    Join { player_id: String, display_name: String },
    SetConnected { player_id: String, connected: bool },
    UpdateName { display_name: String },
    KickPlayer { target_id: String },
    StartGame,
    AcknowledgeDisguise,
    ProposeTeam { team_ids: Vec<String> },
    SubmitVote { approve: bool },
    AdvanceFromBallot,
    SubmitBattleAction { card: BattleCard },
    AdvanceFromBattleOutcome,
    SubmitUnmasking { target_id: String },
    PlayAgain,
    ResetGame,
    SendChat { text: String, timestamp_ms: u64 },
}

#[no_mangle]
pub extern "C" fn wasm_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn wasm_dealloc(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        let _ = Vec::from_raw_parts(ptr, 0, len);
    }
}

#[no_mangle]
pub extern "C" fn wasm_out_ptr() -> *const u8 {
    LAST_OUT.with(|o| o.borrow().as_ptr())
}

#[no_mangle]
pub extern "C" fn wasm_out_len() -> usize {
    LAST_OUT.with(|o| o.borrow().len())
}

fn set_out(s: String) -> i32 {
    LAST_OUT.with(|o| {
        let mut out = o.borrow_mut();
        *out = s.into_bytes();
    });
    0
}

fn set_err(s: String) -> i32 {
    LAST_OUT.with(|o| {
        let mut out = o.borrow_mut();
        *out = s.into_bytes();
    });
    -1
}

#[no_mangle]
pub unsafe extern "C" fn wasm_init_room(room_ptr: *const u8, room_len: usize, seed: u64) -> i32 {
    let bytes = slice::from_raw_parts(room_ptr, room_len);
    let room_code = String::from_utf8_lossy(bytes).into_owned();
    ENGINE.with(|e| {
        *e.borrow_mut() = Some(GameEngine::new_with_seed(room_code, seed));
    });
    set_out("ok".to_string())
}

#[no_mangle]
pub unsafe extern "C" fn wasm_dispatch(
    player_ptr: *const u8,
    player_len: usize,
    cmd_ptr: *const u8,
    cmd_len: usize,
) -> i32 {
    let player_id = String::from_utf8_lossy(slice::from_raw_parts(player_ptr, player_len)).into_owned();
    let cmd_str = String::from_utf8_lossy(slice::from_raw_parts(cmd_ptr, cmd_len)).into_owned();

    let cmd: WasmCommand = match serde_json::from_str(&cmd_str) {
        Ok(c) => c,
        Err(err) => return set_err(format!("Invalid command: {err}")),
    };

    ENGINE.with(|e| {
        let mut borrow = e.borrow_mut();
        let Some(engine) = borrow.as_mut() else {
            return set_err("Game engine not initialized.".to_string());
        };

        let res = match cmd {
            WasmCommand::Join {
                player_id,
                display_name,
            } => engine.add_or_reconnect_player(player_id, display_name),
            WasmCommand::SetConnected {
                player_id,
                connected,
            } => {
                engine.set_connected(&player_id, connected);
                Ok(())
            }
            WasmCommand::UpdateName { display_name } => {
                engine.update_display_name(&player_id, &display_name)
            }
            WasmCommand::KickPlayer { target_id } => {
                engine.kick_lobby_player(&player_id, &target_id)
            }
            WasmCommand::StartGame => engine.start_game(&player_id),
            WasmCommand::AcknowledgeDisguise => engine.acknowledge_disguise(&player_id),
            WasmCommand::ProposeTeam { team_ids } => engine.propose_team(&player_id, team_ids),
            WasmCommand::SubmitVote { approve } => engine.submit_vote(&player_id, approve),
            WasmCommand::AdvanceFromBallot => engine.advance_from_ballot_reveal(&player_id),
            WasmCommand::SubmitBattleAction { card } => {
                engine.submit_battle_action(&player_id, card)
            }
            WasmCommand::AdvanceFromBattleOutcome => {
                engine.advance_from_battle_outcome(&player_id)
            }
            WasmCommand::SubmitUnmasking { target_id } => {
                engine.submit_unmasking(&player_id, &target_id)
            }
            WasmCommand::PlayAgain => engine.play_again(&player_id),
            WasmCommand::ResetGame => engine.reset_game(&player_id),
            WasmCommand::SendChat { text, timestamp_ms } => {
                engine.add_chat_message(&player_id, &text, timestamp_ms)
            }
        };

        match res {
            Ok(()) => {
                engine.bump_version();
                set_out("ok".to_string())
            }
            Err(err) => set_err(err),
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn wasm_view_for_player(
    player_ptr: *const u8,
    player_len: usize,
) -> i32 {
    let player_id = String::from_utf8_lossy(slice::from_raw_parts(player_ptr, player_len)).into_owned();
    ENGINE.with(|e| {
        let borrow = e.borrow();
        let Some(engine) = borrow.as_ref() else {
            return set_err("Game engine not initialized.".to_string());
        };
        match engine.view_for_player(&player_id) {
            Some(view) => match serde_json::to_string(&view) {
                Ok(json) => set_out(json),
                Err(err) => set_err(format!("Serialization error: {err}")),
            },
            None => set_err("Player not found.".to_string()),
        }
    })
}

#[no_mangle]
pub extern "C" fn wasm_view_for_spectator() -> i32 {
    ENGINE.with(|e| {
        let borrow = e.borrow();
        let Some(engine) = borrow.as_ref() else {
            return set_err("Game engine not initialized.".to_string());
        };
        let view = engine.view_for_spectator();
        match serde_json::to_string(&view) {
            Ok(json) => set_out(json),
            Err(err) => set_err(format!("Serialization error: {err}")),
        }
    })
}

#[no_mangle]
pub extern "C" fn wasm_player_ids() -> i32 {
    ENGINE.with(|e| {
        let borrow = e.borrow();
        let Some(engine) = borrow.as_ref() else {
            return set_err("Game engine not initialized.".to_string());
        };
        match serde_json::to_string(&engine.player_ids()) {
            Ok(json) => set_out(json),
            Err(err) => set_err(format!("Serialization error: {err}")),
        }
    })
}

#[no_mangle]
pub extern "C" fn wasm_export_state() -> i32 {
    ENGINE.with(|e| {
        let borrow = e.borrow();
        let Some(engine) = borrow.as_ref() else {
            return set_err("Game engine not initialized.".to_string());
        };
        match serde_json::to_string(engine) {
            Ok(json) => set_out(json),
            Err(err) => set_err(format!("Serialization error: {err}")),
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn wasm_import_state(state_ptr: *const u8, state_len: usize) -> i32 {
    let state_str =
        String::from_utf8_lossy(slice::from_raw_parts(state_ptr, state_len)).into_owned();
    match serde_json::from_str::<GameEngine>(&state_str) {
        Ok(engine) => {
            ENGINE.with(|e| {
                *e.borrow_mut() = Some(engine);
            });
            set_out("ok".to_string())
        }
        Err(err) => set_err(format!("Failed to restore state: {err}")),
    }
}
