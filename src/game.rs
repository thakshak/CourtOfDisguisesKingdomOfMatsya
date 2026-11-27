use serde::{Deserialize, Serialize};
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hasher};

pub type PlayerId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Faction {
    Dharma,
    Infiltrators,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Kanka,
    Brihannala,
    Ballava,
    Syrandhri,
    Granthika,
    Tantipala,
    KingSusharma,
    ChiefSpy,
    Kichaka,
    Karna,
}

impl Role {
    pub fn faction(self) -> Faction {
        match self {
            Role::Kanka
            | Role::Brihannala
            | Role::Ballava
            | Role::Syrandhri
            | Role::Granthika
            | Role::Tantipala => Faction::Dharma,
            Role::KingSusharma | Role::ChiefSpy | Role::Kichaka | Role::Karna => {
                Faction::Infiltrators
            }
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Role::Kanka => "Kanka",
            Role::Brihannala => "Brihannala",
            Role::Ballava => "Ballava",
            Role::Syrandhri => "Syrandhri",
            Role::Granthika => "Granthika",
            Role::Tantipala => "Tantipala",
            Role::KingSusharma => "King Susharma",
            Role::ChiefSpy => "Chief Spy",
            Role::Kichaka => "Kichaka",
            Role::Karna => "Karna",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Role::Kanka => "Dharma Advisor",
            Role::Brihannala => "Dharma Master",
            Role::Ballava => "Dharma Courtier",
            Role::Syrandhri => "Dharma Courtier",
            Role::Granthika => "Dharma Courtier",
            Role::Tantipala => "Dharma Courtier",
            Role::KingSusharma => "Infiltrator Commander",
            Role::ChiefSpy => "Infiltrator Envoy",
            Role::Kichaka => "Infiltrator General",
            Role::Karna => "Lone Infiltrator",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleCard {
    Defend,
    Sabotage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleOutcomeRecord {
    pub winner: Faction,
    pub defend_count: usize,
    pub sabotage_count: usize,
    pub revealed_cards: Vec<BattleCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleInfo {
    pub index: usize,
    pub number: u8,
    pub name: String,
    pub team_size: usize,
    pub sabotages_required: usize,
    pub outcome: Option<BattleOutcomeRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicPhase {
    Lobby,
    CouncilProposal,
    CourtVote,
    BallotReveal,
    SecretBattle,
    BattleOutcome,
    FinalUnmasking,
    Victory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VictoryReason {
    DharmaThreeBattlesAndHidden,
    InfiltratorsThreeSabotages,
    InfiltratorsFiveDeadlocks,
    InfiltratorsUnmaskedKanka,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicPlayer {
    pub id: PlayerId,
    pub display_name: String,
    pub is_host: bool,
    pub connected: bool,
    pub has_acknowledged_disguise: bool,
    pub has_voted: bool,
    pub has_acted_in_battle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotEntry {
    pub player_id: PlayerId,
    pub display_name: String,
    pub approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotRevealView {
    pub proposed_team: Vec<PlayerId>,
    pub entries: Vec<BallotEntry>,
    pub approved: bool,
    pub yes_count: usize,
    pub no_count: usize,
    pub deadlock_count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleProgressView {
    pub submitted_count: usize,
    pub total_count: usize,
    pub waiting_for_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionTarget {
    pub player_id: PlayerId,
    pub display_name: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateRoleView {
    pub role: Role,
    pub display_name: String,
    pub title: String,
    pub faction: Faction,
    pub vision_summary: String,
    pub vision_targets: Vec<VisionTarget>,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateBattleActionView {
    pub can_sabotage: bool,
    pub has_submitted: bool,
    pub submitted_card: Option<BattleCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateUnmaskingView {
    pub is_susharma: bool,
    pub targets: Vec<PublicPlayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevealedPlayerRole {
    pub player_id: PlayerId,
    pub display_name: String,
    pub role: Role,
    pub role_name: String,
    pub title: String,
    pub faction: Faction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictoryView {
    pub winner: Faction,
    pub reason: VictoryReason,
    pub headline: String,
    pub summary: String,
    pub revealed_roles: Vec<RevealedPlayerRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: u64,
    pub sender_id: PlayerId,
    pub sender_name: String,
    pub text: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerView {
    pub state_version: u64,
    pub is_spectator: bool,
    pub self_id: PlayerId,
    pub room_code: String,
    pub phase: PublicPhase,
    pub players: Vec<PublicPlayer>,
    pub battles: Vec<BattleInfo>,
    pub current_battle_index: usize,
    pub deadlock_count: u8,
    pub leader_id: Option<PlayerId>,
    pub proposed_team: Vec<PlayerId>,
    pub waiting_for_names: Vec<String>,
    pub ballot_results: Option<BallotRevealView>,
    pub battle_progress: Option<BattleProgressView>,
    pub latest_battle_outcome: Option<BattleOutcomeRecord>,
    pub private_role: Option<PrivateRoleView>,
    pub private_battle_action: Option<PrivateBattleActionView>,
    pub private_unmasking: Option<PrivateUnmaskingView>,
    pub victory: Option<VictoryView>,
    pub chat_messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlayerRecord {
    id: PlayerId,
    display_name: String,
    is_host: bool,
    connected: bool,
    role: Option<Role>,
    acknowledged_disguise: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn from_entropy() -> Self {
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        for _ in 0..4 {
            let mut hasher = RandomState::new().build_hasher();
            hasher.write_u64(seed);
            seed ^= hasher.finish().rotate_left(13);
        }
        if seed == 0 {
            seed = 0x1234_5678_9ABC_DEF0;
        }
        Self { state: seed }
    }

    pub fn from_seed(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn gen_range(&mut self, upper: usize) -> usize {
        if upper <= 1 {
            return 0;
        }
        (self.next_u64() as usize) % upper
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.gen_range(i + 1);
            slice.swap(i, j);
        }
    }
}

pub const BATTLE_NAMES: [&str; 5] = [
    "Southern Cattle Raid",
    "Defense of the Royal Stables",
    "Rescue of King Virata",
    "Northern Frontier Ambush",
    "Shami Tree Awakening",
];

pub fn roles_for_player_count(count: usize) -> Result<Vec<Role>, String> {
    match count {
        5 => Ok(vec![
            Role::Kanka,
            Role::Brihannala,
            Role::Ballava,
            Role::KingSusharma,
            Role::ChiefSpy,
        ]),
        6 => Ok(vec![
            Role::Kanka,
            Role::Brihannala,
            Role::Ballava,
            Role::Syrandhri,
            Role::KingSusharma,
            Role::ChiefSpy,
        ]),
        7 => Ok(vec![
            Role::Kanka,
            Role::Brihannala,
            Role::Ballava,
            Role::Syrandhri,
            Role::KingSusharma,
            Role::ChiefSpy,
            Role::Kichaka,
        ]),
        8 => Ok(vec![
            Role::Kanka,
            Role::Brihannala,
            Role::Ballava,
            Role::Syrandhri,
            Role::Granthika,
            Role::KingSusharma,
            Role::ChiefSpy,
            Role::Kichaka,
        ]),
        9 => Ok(vec![
            Role::Kanka,
            Role::Brihannala,
            Role::Ballava,
            Role::Syrandhri,
            Role::Granthika,
            Role::Tantipala,
            Role::KingSusharma,
            Role::ChiefSpy,
            Role::Kichaka,
        ]),
        10 => Ok(vec![
            Role::Kanka,
            Role::Brihannala,
            Role::Ballava,
            Role::Syrandhri,
            Role::Granthika,
            Role::Tantipala,
            Role::KingSusharma,
            Role::ChiefSpy,
            Role::Kichaka,
            Role::Karna,
        ]),
        _ => Err(format!(
            "Battle of Matsya requires 5 to 10 players (got {count})"
        )),
    }
}

pub fn battle_team_sizes(player_count: usize) -> Result<[usize; 5], String> {
    match player_count {
        5 => Ok([2, 3, 2, 3, 3]),
        6 => Ok([2, 3, 4, 3, 4]),
        7 => Ok([2, 3, 3, 4, 4]),
        8..=10 => Ok([3, 4, 4, 5, 5]),
        _ => Err(format!(
            "Battle of Matsya requires 5 to 10 players (got {player_count})"
        )),
    }
}

pub fn sabotages_required_for_battle(player_count: usize, battle_index: usize) -> usize {
    if battle_index == 3 && (7..=10).contains(&player_count) {
        2
    } else {
        1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEngine {
    pub room_code: String,
    state_version: u64,
    rng: SimpleRng,
    phase: PublicPhase,
    players: Vec<PlayerRecord>,
    battles: Vec<BattleInfo>,
    current_battle_index: usize,
    deadlock_count: u8,
    leader_index: usize,
    proposed_team: Vec<PlayerId>,
    votes: HashMap<PlayerId, bool>,
    ballot_results: Option<BallotRevealView>,
    battle_submissions: HashMap<PlayerId, BattleCard>,
    latest_battle_outcome: Option<BattleOutcomeRecord>,
    victory: Option<VictoryView>,
    chat_messages: Vec<ChatMessage>,
    next_chat_id: u64,
}

impl GameEngine {
    pub fn new(room_code: impl Into<String>) -> Self {
        Self::new_with_rng(room_code, SimpleRng::from_entropy())
    }

    pub fn new_with_seed(room_code: impl Into<String>, seed: u64) -> Self {
        Self::new_with_rng(room_code, SimpleRng::from_seed(seed))
    }

    fn new_with_rng(room_code: impl Into<String>, rng: SimpleRng) -> Self {
        Self {
            room_code: room_code.into(),
            state_version: 1,
            rng,
            phase: PublicPhase::Lobby,
            players: Vec::new(),
            battles: Vec::new(),
            current_battle_index: 0,
            deadlock_count: 0,
            leader_index: 0,
            proposed_team: Vec::new(),
            votes: HashMap::new(),
            ballot_results: None,
            battle_submissions: HashMap::new(),
            latest_battle_outcome: None,
            victory: None,
            chat_messages: Vec::new(),
            next_chat_id: 1,
        }
    }

    pub fn phase(&self) -> PublicPhase {
        self.phase
    }

    pub fn player_ids(&self) -> Vec<PlayerId> {
        self.players.iter().map(|p| p.id.clone()).collect()
    }

    pub fn player_role(&self, player_id: &str) -> Option<Role> {
        self.players
            .iter()
            .find(|p| p.id == player_id)
            .and_then(|p| p.role)
    }

    pub fn add_or_reconnect_player(
        &mut self,
        player_id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Result<(), String> {
        let player_id = player_id.into();
        let clean_name = sanitize_display_name(&display_name.into());
        if let Some(existing) = self.players.iter_mut().find(|p| p.id == player_id) {
            existing.connected = true;
            if !clean_name.is_empty() && self.phase == PublicPhase::Lobby {
                existing.display_name = clean_name;
            }
            return Ok(());
        }

        if self.phase != PublicPhase::Lobby {
            return Err("Game has already begun; new players cannot join.".to_string());
        }
        if self.players.len() >= 10 {
            return Err("Matsya Court is full (maximum 10 players).".to_string());
        }
        if clean_name.is_empty() {
            return Err("Display name cannot be empty.".to_string());
        }

        let is_host = self.players.is_empty();
        self.players.push(PlayerRecord {
            id: player_id,
            display_name: clean_name,
            is_host,
            connected: true,
            role: None,
            acknowledged_disguise: false,
        });
        Ok(())
    }

    pub fn update_display_name(
        &mut self,
        player_id: &str,
        display_name: &str,
    ) -> Result<(), String> {
        if self.phase != PublicPhase::Lobby {
            return Err("Cannot change display name after council begins.".to_string());
        }
        let clean = sanitize_display_name(display_name);
        if clean.is_empty() {
            return Err("Display name cannot be empty.".to_string());
        }
        let player = self
            .players
            .iter_mut()
            .find(|p| p.id == player_id)
            .ok_or_else(|| "Player not found.".to_string())?;
        player.display_name = clean;
        Ok(())
    }

    pub fn set_connected(&mut self, player_id: &str, connected: bool) {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            player.connected = connected;
        }
    }

    pub fn bump_version(&mut self) {
        self.state_version = self.state_version.wrapping_add(1);
    }

    pub fn kick_lobby_player(
        &mut self,
        requester_id: &str,
        target_id: &str,
    ) -> Result<(), String> {
        if self.phase != PublicPhase::Lobby {
            return Err("Players can only be removed before the council begins.".to_string());
        }
        let is_host = self
            .players
            .iter()
            .find(|p| p.id == requester_id)
            .map(|p| p.is_host)
            .unwrap_or(false);
        if !is_host {
            return Err("Only the host can remove players from the lobby.".to_string());
        }
        if requester_id == target_id {
            return Err("Host cannot remove themselves.".to_string());
        }
        self.remove_lobby_player(target_id)
    }

    pub fn remove_lobby_player(&mut self, player_id: &str) -> Result<(), String> {
        if self.phase != PublicPhase::Lobby {
            return Err("Cannot leave during an active game.".to_string());
        }
        let idx = self
            .players
            .iter()
            .position(|p| p.id == player_id)
            .ok_or_else(|| "Player not found.".to_string())?;
        let was_host = self.players[idx].is_host;
        self.players.remove(idx);
        if was_host && !self.players.is_empty() {
            self.players[0].is_host = true;
        }
        Ok(())
    }

    pub fn add_chat_message(
        &mut self,
        sender_id: &str,
        text: &str,
        timestamp_ms: u64,
    ) -> Result<(), String> {
        let sender_name = self
            .players
            .iter()
            .find(|p| p.id == sender_id)
            .map(|p| p.display_name.clone())
            .ok_or_else(|| "Player not in court.".to_string())?;
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err("Message cannot be empty.".to_string());
        }
        let capped: String = trimmed.chars().take(280).collect();
        self.chat_messages.push(ChatMessage {
            id: self.next_chat_id,
            sender_id: sender_id.to_string(),
            sender_name,
            text: capped,
            timestamp_ms,
        });
        self.next_chat_id += 1;
        if self.chat_messages.len() > 100 {
            let overflow = self.chat_messages.len() - 100;
            self.chat_messages.drain(0..overflow);
        }
        Ok(())
    }

    pub fn start_game(&mut self, requester_id: &str) -> Result<(), String> {
        if self.phase != PublicPhase::Lobby {
            return Err("Council has already begun.".to_string());
        }
        let is_host = self
            .players
            .iter()
            .find(|p| p.id == requester_id)
            .map(|p| p.is_host)
            .unwrap_or(false);
        if !is_host {
            return Err("Only the host can begin the council.".to_string());
        }
        let count = self.players.len();
        let mut roles = roles_for_player_count(count)?;
        self.rng.shuffle(&mut roles);

        for (player, role) in self.players.iter_mut().zip(roles.into_iter()) {
            player.role = Some(role);
            player.acknowledged_disguise = false;
        }

        let team_sizes = battle_team_sizes(count)?;
        self.battles = (0..5)
            .map(|idx| BattleInfo {
                index: idx,
                number: (idx + 1) as u8,
                name: BATTLE_NAMES[idx].to_string(),
                team_size: team_sizes[idx],
                sabotages_required: sabotages_required_for_battle(count, idx),
                outcome: None,
            })
            .collect();

        self.current_battle_index = 0;
        self.deadlock_count = 0;
        self.leader_index = self.rng.gen_range(count);
        self.proposed_team.clear();
        self.votes.clear();
        self.ballot_results = None;
        self.battle_submissions.clear();
        self.latest_battle_outcome = None;
        self.victory = None;
        self.phase = PublicPhase::CouncilProposal;
        Ok(())
    }

    pub fn acknowledge_disguise(&mut self, player_id: &str) -> Result<(), String> {
        if self.phase == PublicPhase::Lobby {
            return Err("Disguises have not been dealt yet.".to_string());
        }
        let player = self
            .players
            .iter_mut()
            .find(|p| p.id == player_id)
            .ok_or_else(|| "Player not found.".to_string())?;
        player.acknowledged_disguise = true;
        Ok(())
    }

    pub fn propose_team(
        &mut self,
        leader_id: &str,
        team_ids: Vec<PlayerId>,
    ) -> Result<(), String> {
        if self.phase != PublicPhase::CouncilProposal {
            return Err("Not currently in Council Proposal phase.".to_string());
        }
        let expected_leader = self
            .players
            .get(self.leader_index)
            .map(|p| p.id.as_str())
            .ok_or_else(|| "No active leader.".to_string())?;
        if expected_leader != leader_id {
            return Err("Only the active Council Leader holding the Royal Signet can propose a team.".to_string());
        }
        let required_size = self.battles[self.current_battle_index].team_size;
        if team_ids.len() != required_size {
            return Err(format!(
                "Battle {} requires exactly {} warriors (selected {}).",
                self.current_battle_index + 1,
                required_size,
                team_ids.len()
            ));
        }

        let valid_ids: HashSet<&str> = self.players.iter().map(|p| p.id.as_str()).collect();
        let mut seen = HashSet::new();
        for id in &team_ids {
            if !valid_ids.contains(id.as_str()) {
                return Err(format!("Unknown player in proposed team: {id}"));
            }
            if !seen.insert(id.as_str()) {
                return Err("Duplicate player in proposed team.".to_string());
            }
        }

        self.proposed_team = team_ids;
        self.votes.clear();
        self.ballot_results = None;
        self.phase = PublicPhase::CourtVote;
        Ok(())
    }

    pub fn submit_vote(&mut self, player_id: &str, approve: bool) -> Result<(), String> {
        if self.phase != PublicPhase::CourtVote {
            return Err("Not currently in Court Vote phase.".to_string());
        }
        if !self.players.iter().any(|p| p.id == player_id) {
            return Err("Player not found.".to_string());
        }
        if self.votes.contains_key(player_id) {
            return Err("You have already cast your vote.".to_string());
        }
        self.votes.insert(player_id.to_string(), approve);

        if self.votes.len() == self.players.len() {
            let yes_count = self.votes.values().filter(|&&v| v).count();
            let no_count = self.players.len() - yes_count;
            let approved = yes_count > no_count;

            if approved {
                self.deadlock_count = 0;
            } else {
                self.deadlock_count += 1;
            }

            let entries = self
                .players
                .iter()
                .map(|p| BallotEntry {
                    player_id: p.id.clone(),
                    display_name: p.display_name.clone(),
                    approve: *self.votes.get(&p.id).unwrap_or(&false),
                })
                .collect();

            self.ballot_results = Some(BallotRevealView {
                proposed_team: self.proposed_team.clone(),
                entries,
                approved,
                yes_count,
                no_count,
                deadlock_count: self.deadlock_count,
            });

            if !approved && self.deadlock_count >= 5 {
                self.finish_game(
                    Faction::Infiltrators,
                    VictoryReason::InfiltratorsFiveDeadlocks,
                );
            } else {
                self.phase = PublicPhase::BallotReveal;
            }
        }

        Ok(())
    }

    pub fn advance_from_ballot_reveal(&mut self, requester_id: &str) -> Result<(), String> {
        if self.phase != PublicPhase::BallotReveal {
            return Err("Not in Ballot Reveal phase.".to_string());
        }
        self.ensure_leader_or_host(requester_id)?;
        let approved = self
            .ballot_results
            .as_ref()
            .map(|b| b.approved)
            .unwrap_or(false);

        if approved {
            self.battle_submissions.clear();
            self.phase = PublicPhase::SecretBattle;
        } else {
            self.leader_index = (self.leader_index + 1) % self.players.len();
            self.proposed_team.clear();
            self.votes.clear();
            self.ballot_results = None;
            self.phase = PublicPhase::CouncilProposal;
        }
        Ok(())
    }

    pub fn submit_battle_action(
        &mut self,
        player_id: &str,
        card: BattleCard,
    ) -> Result<(), String> {
        if self.phase != PublicPhase::SecretBattle {
            return Err("Not currently in Secret Battle phase.".to_string());
        }
        if !self.proposed_team.iter().any(|id| id == player_id) {
            return Err("Only selected Battle Team members may act.".to_string());
        }
        if self.battle_submissions.contains_key(player_id) {
            return Err("You have already submitted your battle action.".to_string());
        }
        let role = self
            .player_role(player_id)
            .ok_or_else(|| "Player role missing.".to_string())?;
        if role.faction() == Faction::Dharma && card == BattleCard::Sabotage {
            return Err("Dharma Defenders cannot play Sabotage cards.".to_string());
        }

        self.battle_submissions.insert(player_id.to_string(), card);

        if self.battle_submissions.len() == self.proposed_team.len() {
            let mut cards: Vec<BattleCard> = self.battle_submissions.values().copied().collect();
            self.rng.shuffle(&mut cards);
            let sabotage_count = cards
                .iter()
                .filter(|&&c| c == BattleCard::Sabotage)
                .count();
            let defend_count = cards.len() - sabotage_count;
            let required = self.battles[self.current_battle_index].sabotages_required;
            let winner = if sabotage_count >= required {
                Faction::Infiltrators
            } else {
                Faction::Dharma
            };

            let outcome = BattleOutcomeRecord {
                winner,
                defend_count,
                sabotage_count,
                revealed_cards: cards,
            };
            self.battles[self.current_battle_index].outcome = Some(outcome.clone());
            self.latest_battle_outcome = Some(outcome);
            self.phase = PublicPhase::BattleOutcome;
        }

        Ok(())
    }

    pub fn advance_from_battle_outcome(&mut self, requester_id: &str) -> Result<(), String> {
        if self.phase != PublicPhase::BattleOutcome {
            return Err("Not in Battle Outcome phase.".to_string());
        }
        self.ensure_leader_or_host(requester_id)?;

        let dharma_wins = self
            .battles
            .iter()
            .filter(|b| {
                b.outcome
                    .as_ref()
                    .map(|o| o.winner == Faction::Dharma)
                    .unwrap_or(false)
            })
            .count();
        let infiltrator_wins = self
            .battles
            .iter()
            .filter(|b| {
                b.outcome
                    .as_ref()
                    .map(|o| o.winner == Faction::Infiltrators)
                    .unwrap_or(false)
            })
            .count();

        if infiltrator_wins >= 3 {
            self.finish_game(
                Faction::Infiltrators,
                VictoryReason::InfiltratorsThreeSabotages,
            );
        } else if dharma_wins >= 3 {
            self.phase = PublicPhase::FinalUnmasking;
        } else {
            self.current_battle_index += 1;
            self.leader_index = (self.leader_index + 1) % self.players.len();
            self.proposed_team.clear();
            self.votes.clear();
            self.ballot_results = None;
            self.battle_submissions.clear();
            self.latest_battle_outcome = None;
            self.phase = PublicPhase::CouncilProposal;
        }

        Ok(())
    }

    pub fn submit_unmasking(
        &mut self,
        requester_id: &str,
        target_id: &str,
    ) -> Result<(), String> {
        if self.phase != PublicPhase::FinalUnmasking {
            return Err("Not in Final Unmasking phase.".to_string());
        }
        let requester_role = self
            .player_role(requester_id)
            .ok_or_else(|| "Player not found.".to_string())?;
        if requester_role != Role::KingSusharma {
            return Err("Only King Susharma may attempt the Final Unmasking.".to_string());
        }

        let target_role = self
            .player_role(target_id)
            .ok_or_else(|| "Target player not found.".to_string())?;
        if target_role.faction() != Faction::Dharma {
            return Err("King Susharma must choose a Dharma Defender to unmask.".to_string());
        }

        if target_role == Role::Kanka {
            self.finish_game(
                Faction::Infiltrators,
                VictoryReason::InfiltratorsUnmaskedKanka,
            );
        } else {
            self.finish_game(Faction::Dharma, VictoryReason::DharmaThreeBattlesAndHidden);
        }

        Ok(())
    }

    pub fn play_again(&mut self, requester_id: &str) -> Result<(), String> {
        if self.phase != PublicPhase::Victory {
            return Err("Can only Play Again after the game concludes.".to_string());
        }
        self.reset_game(requester_id)
    }

    pub fn reset_game(&mut self, requester_id: &str) -> Result<(), String> {
        let is_host = self
            .players
            .iter()
            .find(|p| p.id == requester_id)
            .map(|p| p.is_host)
            .unwrap_or(false);
        if !is_host {
            return Err("Only the host can reset the court for a new game.".to_string());
        }

        self.phase = PublicPhase::Lobby;
        self.battles.clear();
        self.current_battle_index = 0;
        self.deadlock_count = 0;
        self.leader_index = 0;
        self.proposed_team.clear();
        self.votes.clear();
        self.ballot_results = None;
        self.battle_submissions.clear();
        self.latest_battle_outcome = None;
        self.victory = None;
        for p in &mut self.players {
            p.role = None;
            p.acknowledged_disguise = false;
        }
        Ok(())
    }

    fn ensure_leader_or_host(&self, requester_id: &str) -> Result<(), String> {
        let is_leader = self
            .players
            .get(self.leader_index)
            .map(|p| p.id == requester_id)
            .unwrap_or(false);
        let is_host = self
            .players
            .iter()
            .find(|p| p.id == requester_id)
            .map(|p| p.is_host)
            .unwrap_or(false);
        if !is_leader && !is_host {
            return Err("Only the Council Leader or Host can advance this phase.".to_string());
        }
        Ok(())
    }

    fn finish_game(&mut self, winner: Faction, reason: VictoryReason) {
        let headline = match winner {
            Faction::Dharma => "DHARMA DEFENDS MATSYA".to_string(),
            Faction::Infiltrators => "INFILTRATORS CONQUER MATSYA".to_string(),
        };
        let summary = match reason {
            VictoryReason::DharmaThreeBattlesAndHidden => {
                "Dharma won 3 battles and Kanka's true identity remained hidden!".to_string()
            }
            VictoryReason::InfiltratorsThreeSabotages => {
                "Infiltrators sabotaged 3 out of 5 battles for Trigarta and Hastinapura!".to_string()
            }
            VictoryReason::InfiltratorsFiveDeadlocks => {
                "Five consecutive council proposals were rejected, plunging Matsya into chaos!"
                    .to_string()
            }
            VictoryReason::InfiltratorsUnmaskedKanka => {
                "King Susharma unmasked Kanka at the final hour, stealing victory!".to_string()
            }
        };

        let revealed_roles = self
            .players
            .iter()
            .filter_map(|p| {
                p.role.map(|role| RevealedPlayerRole {
                    player_id: p.id.clone(),
                    display_name: p.display_name.clone(),
                    role,
                    role_name: role.display_name().to_string(),
                    title: role.title().to_string(),
                    faction: role.faction(),
                })
            })
            .collect();

        self.victory = Some(VictoryView {
            winner,
            reason,
            headline,
            summary,
            revealed_roles,
        });
        self.phase = PublicPhase::Victory;
    }

    pub fn view_for_player(&self, viewer_id: &str) -> Option<PlayerView> {
        let viewer = self.players.iter().find(|p| p.id == viewer_id)?;

        let players: Vec<PublicPlayer> = self
            .players
            .iter()
            .map(|p| PublicPlayer {
                id: p.id.clone(),
                display_name: p.display_name.clone(),
                is_host: p.is_host,
                connected: p.connected,
                has_acknowledged_disguise: p.acknowledged_disguise,
                has_voted: self.votes.contains_key(&p.id),
                has_acted_in_battle: self.battle_submissions.contains_key(&p.id),
            })
            .collect();

        let leader_id = if self.phase != PublicPhase::Lobby {
            self.players.get(self.leader_index).map(|p| p.id.clone())
        } else {
            None
        };

        let waiting_for_names = match self.phase {
            PublicPhase::CourtVote => self
                .players
                .iter()
                .filter(|p| !self.votes.contains_key(&p.id))
                .map(|p| p.display_name.clone())
                .collect(),
            PublicPhase::SecretBattle => self
                .proposed_team
                .iter()
                .filter(|id| !self.battle_submissions.contains_key(*id))
                .filter_map(|id| self.players.iter().find(|p| &p.id == id))
                .map(|p| p.display_name.clone())
                .collect(),
            _ => Vec::new(),
        };

        let battle_progress = if self.phase == PublicPhase::SecretBattle {
            Some(BattleProgressView {
                submitted_count: self.battle_submissions.len(),
                total_count: self.proposed_team.len(),
                waiting_for_names: waiting_for_names.clone(),
            })
        } else {
            None
        };

        let private_role = viewer.role.map(|role| self.build_private_role(viewer, role));

        let private_battle_action = if self.phase == PublicPhase::SecretBattle
            && self.proposed_team.iter().any(|id| id == viewer_id)
        {
            let can_sabotage = viewer
                .role
                .map(|r| r.faction() == Faction::Infiltrators)
                .unwrap_or(false);
            let submitted_card = self.battle_submissions.get(viewer_id).copied();
            Some(PrivateBattleActionView {
                can_sabotage,
                has_submitted: submitted_card.is_some(),
                submitted_card,
            })
        } else {
            None
        };

        let private_unmasking = if self.phase == PublicPhase::FinalUnmasking {
            let is_susharma = viewer.role == Some(Role::KingSusharma);
            let targets = if is_susharma {
                self.players
                    .iter()
                    .filter(|p| {
                        p.role
                            .map(|r| r.faction() == Faction::Dharma)
                            .unwrap_or(false)
                    })
                    .map(|p| PublicPlayer {
                        id: p.id.clone(),
                        display_name: p.display_name.clone(),
                        is_host: p.is_host,
                        connected: p.connected,
                        has_acknowledged_disguise: p.acknowledged_disguise,
                        has_voted: false,
                        has_acted_in_battle: false,
                    })
                    .collect()
            } else {
                Vec::new()
            };
            Some(PrivateUnmaskingView {
                is_susharma,
                targets,
            })
        } else {
            None
        };

        Some(PlayerView {
            state_version: self.state_version,
            is_spectator: false,
            self_id: viewer.id.clone(),
            room_code: self.room_code.clone(),
            phase: self.phase,
            players,
            battles: self.battles.clone(),
            current_battle_index: self.current_battle_index,
            deadlock_count: self.deadlock_count,
            leader_id,
            proposed_team: self.proposed_team.clone(),
            waiting_for_names,
            ballot_results: self.ballot_results.clone(),
            battle_progress,
            latest_battle_outcome: self.latest_battle_outcome.clone(),
            private_role,
            private_battle_action,
            private_unmasking,
            victory: self.victory.clone(),
            chat_messages: self.chat_messages.clone(),
        })
    }

    pub fn view_for_spectator(&self) -> PlayerView {
        let players: Vec<PublicPlayer> = self
            .players
            .iter()
            .map(|p| PublicPlayer {
                id: p.id.clone(),
                display_name: p.display_name.clone(),
                is_host: p.is_host,
                connected: p.connected,
                has_acknowledged_disguise: p.acknowledged_disguise,
                has_voted: self.votes.contains_key(&p.id),
                has_acted_in_battle: self.battle_submissions.contains_key(&p.id),
            })
            .collect();

        let leader_id = if self.phase != PublicPhase::Lobby {
            self.players.get(self.leader_index).map(|p| p.id.clone())
        } else {
            None
        };

        let waiting_for_names = match self.phase {
            PublicPhase::CourtVote => self
                .players
                .iter()
                .filter(|p| !self.votes.contains_key(&p.id))
                .map(|p| p.display_name.clone())
                .collect(),
            PublicPhase::SecretBattle => self
                .proposed_team
                .iter()
                .filter(|id| !self.battle_submissions.contains_key(*id))
                .filter_map(|id| self.players.iter().find(|p| &p.id == id))
                .map(|p| p.display_name.clone())
                .collect(),
            _ => Vec::new(),
        };

        let battle_progress = if self.phase == PublicPhase::SecretBattle {
            Some(BattleProgressView {
                submitted_count: self.battle_submissions.len(),
                total_count: self.proposed_team.len(),
                waiting_for_names: waiting_for_names.clone(),
            })
        } else {
            None
        };

        PlayerView {
            state_version: self.state_version,
            is_spectator: true,
            self_id: "spectator".to_string(),
            room_code: self.room_code.clone(),
            phase: self.phase,
            players,
            battles: self.battles.clone(),
            current_battle_index: self.current_battle_index,
            deadlock_count: self.deadlock_count,
            leader_id,
            proposed_team: self.proposed_team.clone(),
            waiting_for_names,
            ballot_results: self.ballot_results.clone(),
            battle_progress,
            latest_battle_outcome: self.latest_battle_outcome.clone(),
            private_role: None,
            private_battle_action: None,
            private_unmasking: None,
            victory: self.victory.clone(),
            chat_messages: self.chat_messages.clone(),
        }
    }

    fn build_private_role(&self, viewer: &PlayerRecord, role: Role) -> PrivateRoleView {
        let mut vision_targets: Vec<VisionTarget> = Vec::new();
        let vision_summary = match role {
            Role::Kanka => {
                for p in &self.players {
                    if let Some(other_role) = p.role {
                        if other_role.faction() == Faction::Infiltrators
                            && other_role != Role::Kichaka
                        {
                            vision_targets.push(VisionTarget {
                                player_id: p.id.clone(),
                                display_name: p.display_name.clone(),
                                label: "Infiltrator".to_string(),
                            });
                        }
                    }
                }
                let names = join_names(&vision_targets);
                format!(
                    "You are KANKA (Dharma). Infiltrators detected (excluding Kichaka): {names}."
                )
            }
            Role::Brihannala => {
                for p in &self.players {
                    if let Some(other_role) = p.role {
                        if other_role == Role::Kanka || other_role == Role::ChiefSpy {
                            vision_targets.push(VisionTarget {
                                player_id: p.id.clone(),
                                display_name: p.display_name.clone(),
                                label: "Kanka or Chief Spy".to_string(),
                            });
                        }
                    }
                }
                // Sort alphabetically so seat order never leaks which is Kanka vs Chief Spy.
                vision_targets.sort_by(|a, b| a.display_name.cmp(&b.display_name));
                let names = join_names(&vision_targets);
                format!(
                    "You are BRIHANNALA (Dharma). One of these is Kanka and one is Chief Spy: {names}."
                )
            }
            Role::Ballava | Role::Syrandhri | Role::Granthika | Role::Tantipala => {
                format!(
                    "You are {} (Dharma). You have no special vision; deduce allies through votes and battle outcomes.",
                    role.display_name().to_uppercase()
                )
            }
            Role::KingSusharma | Role::ChiefSpy | Role::Kichaka => {
                for p in &self.players {
                    if p.id == viewer.id {
                        continue;
                    }
                    if let Some(other_role) = p.role {
                        if other_role.faction() == Faction::Infiltrators
                            && other_role != Role::Karna
                        {
                            vision_targets.push(VisionTarget {
                                player_id: p.id.clone(),
                                display_name: p.display_name.clone(),
                                label: "Fellow Infiltrator".to_string(),
                            });
                        }
                    }
                }
                let names = join_names(&vision_targets);
                let extra = match role {
                    Role::KingSusharma => {
                        " If Dharma wins 3 battles, you get the final Unmasking guess."
                    }
                    Role::ChiefSpy => " You appear as Kanka to Brihannala.",
                    Role::Kichaka => " You are completely hidden from Kanka's vision.",
                    _ => "",
                };
                format!(
                    "You are {} (Infiltrator). Fellow Infiltrators (excluding Karna): {names}.{extra}",
                    role.display_name().to_uppercase()
                )
            }
            Role::Karna => {
                "You are KARNA (Lone Infiltrator). You do NOT know the other Infiltrators, and they do NOT know you.".to_string()
            }
        };

        PrivateRoleView {
            role,
            display_name: role.display_name().to_string(),
            title: role.title().to_string(),
            faction: role.faction(),
            vision_summary,
            vision_targets,
            acknowledged: viewer.acknowledged_disguise,
        }
    }
}

fn join_names(targets: &[VisionTarget]) -> String {
    if targets.is_empty() {
        "None".to_string()
    } else {
        targets
            .iter()
            .map(|t| t.display_name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn sanitize_display_name(raw: &str) -> String {
    raw.trim().chars().take(24).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_game(player_count: usize, seed: u64) -> GameEngine {
        let mut engine = GameEngine::new_with_seed("MATSYA", seed);
        for i in 1..=player_count {
            engine
                .add_or_reconnect_player(format!("p{i}"), format!("Player{i}"))
                .unwrap();
        }
        engine.start_game("p1").unwrap();
        engine
    }

    #[test]
    fn role_counts_and_factions_match_spec() {
        let expected = [(5, 3, 2), (6, 4, 2), (7, 4, 3), (8, 5, 3), (9, 6, 3), (10, 6, 4)];
        for (count, dharma_exp, inf_exp) in expected {
            let roles = roles_for_player_count(count).unwrap();
            assert_eq!(roles.len(), count);
            let dharma = roles.iter().filter(|r| r.faction() == Faction::Dharma).count();
            let inf = roles
                .iter()
                .filter(|r| r.faction() == Faction::Infiltrators)
                .count();
            assert_eq!(dharma, dharma_exp, "dharma mismatch for {count}");
            assert_eq!(inf, inf_exp, "infiltrator mismatch for {count}");
        }
        assert!(roles_for_player_count(4).is_err());
        assert!(roles_for_player_count(11).is_err());
    }

    #[test]
    fn vision_rules_and_privacy_isolation_for_10_players() {
        let engine = setup_game(10, 42);
        let find_by_role = |target: Role| -> String {
            engine
                .player_ids()
                .into_iter()
                .find(|id| engine.player_role(id) == Some(target))
                .unwrap()
        };

        let kanka = find_by_role(Role::Kanka);
        let brihannala = find_by_role(Role::Brihannala);
        let susharma = find_by_role(Role::KingSusharma);
        let chief_spy = find_by_role(Role::ChiefSpy);
        let kichaka = find_by_role(Role::Kichaka);
        let karna = find_by_role(Role::Karna);
        let ballava = find_by_role(Role::Ballava);

        // Kanka sees Susharma, Chief Spy, Karna — NOT Kichaka
        let kanka_view = engine.view_for_player(&kanka).unwrap();
        let kanka_targets: HashSet<String> = kanka_view
            .private_role
            .unwrap()
            .vision_targets
            .into_iter()
            .map(|t| t.player_id)
            .collect();
        assert!(kanka_targets.contains(&susharma));
        assert!(kanka_targets.contains(&chief_spy));
        assert!(kanka_targets.contains(&karna));
        assert!(!kanka_targets.contains(&kichaka));

        // Brihannala sees Kanka and Chief Spy
        let bri_targets: HashSet<String> = engine
            .view_for_player(&brihannala)
            .unwrap()
            .private_role
            .unwrap()
            .vision_targets
            .into_iter()
            .map(|t| t.player_id)
            .collect();
        assert_eq!(bri_targets.len(), 2);
        assert!(bri_targets.contains(&kanka));
        assert!(bri_targets.contains(&chief_spy));

        // Susharma sees Chief Spy and Kichaka — NOT Karna
        let susharma_targets: HashSet<String> = engine
            .view_for_player(&susharma)
            .unwrap()
            .private_role
            .unwrap()
            .vision_targets
            .into_iter()
            .map(|t| t.player_id)
            .collect();
        assert!(susharma_targets.contains(&chief_spy));
        assert!(susharma_targets.contains(&kichaka));
        assert!(!susharma_targets.contains(&karna));

        // Karna sees nobody
        let karna_targets = engine
            .view_for_player(&karna)
            .unwrap()
            .private_role
            .unwrap()
            .vision_targets;
        assert!(karna_targets.is_empty());

        // Ballava sees nobody
        let ballava_targets = engine
            .view_for_player(&ballava)
            .unwrap()
            .private_role
            .unwrap()
            .vision_targets;
        assert!(ballava_targets.is_empty());

        // No victory view yet — roles are strictly private
        assert!(kanka_view.victory.is_none());
    }

    #[test]
    fn five_consecutive_deadlocks_give_infiltrators_victory() {
        let mut engine = setup_game(5, 7);
        for round in 1..=5 {
            let leader = engine.view_for_player("p1").unwrap().leader_id.unwrap();
            engine
                .propose_team(&leader, vec!["p1".to_string(), "p2".to_string()])
                .unwrap();
            // 2 yes, 3 no => tie/majority NO
            engine.submit_vote("p1", true).unwrap();
            engine.submit_vote("p2", true).unwrap();
            engine.submit_vote("p3", false).unwrap();
            engine.submit_vote("p4", false).unwrap();
            engine.submit_vote("p5", false).unwrap();

            if round < 5 {
                assert_eq!(engine.phase(), PublicPhase::BallotReveal);
                engine.advance_from_ballot_reveal("p1").unwrap();
                assert_eq!(engine.phase(), PublicPhase::CouncilProposal);
            } else {
                assert_eq!(engine.phase(), PublicPhase::Victory);
                let v = engine.view_for_player("p1").unwrap().victory.unwrap();
                assert_eq!(v.winner, Faction::Infiltrators);
                assert_eq!(v.reason, VictoryReason::InfiltratorsFiveDeadlocks);
            }
        }
    }

    #[test]
    fn battle_4_in_7_player_game_requires_two_sabotages() {
        let mut engine = setup_game(7, 99);
        let dharma_players: Vec<String> = engine
            .player_ids()
            .into_iter()
            .filter(|id| engine.player_role(id).unwrap().faction() == Faction::Dharma)
            .collect();
        let inf_players: Vec<String> = engine
            .player_ids()
            .into_iter()
            .filter(|id| engine.player_role(id).unwrap().faction() == Faction::Infiltrators)
            .collect();

        // Win Battles 1 and 2 with Dharma, lose Battle 3 with 1 sabotage
        let teams = [
            vec![dharma_players[0].clone(), dharma_players[1].clone()], // Battle 1 (size 2)
            vec![
                dharma_players[0].clone(),
                dharma_players[1].clone(),
                dharma_players[2].clone(),
            ], // Battle 2 (size 3)
            vec![
                dharma_players[0].clone(),
                dharma_players[1].clone(),
                inf_players[0].clone(),
            ], // Battle 3 (size 3)
        ];

        for (idx, team) in teams.iter().enumerate() {
            let leader = engine.view_for_player("p1").unwrap().leader_id.unwrap();
            engine.propose_team(&leader, team.clone()).unwrap();
            for p in engine.player_ids() {
                engine.submit_vote(&p, true).unwrap();
            }
            engine.advance_from_ballot_reveal("p1").unwrap();
            for member in team {
                let card = if idx == 2 && member == &inf_players[0] {
                    BattleCard::Sabotage
                } else {
                    BattleCard::Defend
                };
                engine.submit_battle_action(member, card).unwrap();
            }
            engine.advance_from_battle_outcome("p1").unwrap();
        }

        // Now Battle 4 (size 4, requires 2 sabotages) — put 1 infiltrator who sabotages: Dharma still wins!
        let leader = engine.view_for_player("p1").unwrap().leader_id.unwrap();
        let team4 = vec![
            dharma_players[0].clone(),
            dharma_players[1].clone(),
            dharma_players[2].clone(),
            inf_players[0].clone(),
        ];
        engine.propose_team(&leader, team4.clone()).unwrap();
        for p in engine.player_ids() {
            engine.submit_vote(&p, true).unwrap();
        }
        engine.advance_from_ballot_reveal("p1").unwrap();

        // Verify Dharma cannot sabotage
        assert!(engine
            .submit_battle_action(&dharma_players[0], BattleCard::Sabotage)
            .is_err());

        for member in &team4 {
            let card = if member == &inf_players[0] {
                BattleCard::Sabotage
            } else {
                BattleCard::Defend
            };
            engine.submit_battle_action(member, card).unwrap();
        }
        let outcome = engine
            .view_for_player("p1")
            .unwrap()
            .latest_battle_outcome
            .unwrap();
        assert_eq!(outcome.sabotage_count, 1);
        assert_eq!(outcome.winner, Faction::Dharma);

        // Advancing should enter FinalUnmasking (3 Dharma wins!)
        engine.advance_from_battle_outcome("p1").unwrap();
        assert_eq!(engine.phase(), PublicPhase::FinalUnmasking);

        let susharma = engine
            .player_ids()
            .into_iter()
            .find(|id| engine.player_role(id) == Some(Role::KingSusharma))
            .unwrap();
        let kanka = engine
            .player_ids()
            .into_iter()
            .find(|id| engine.player_role(id) == Some(Role::Kanka))
            .unwrap();

        // Non-Susharma sees empty targets
        let other_view = engine.view_for_player(&kanka).unwrap();
        let unmask = other_view.private_unmasking.unwrap();
        assert!(!unmask.is_susharma);
        assert!(unmask.targets.is_empty());

        // Susharma unmasks Kanka -> Infiltrators steal victory
        engine.submit_unmasking(&susharma, &kanka).unwrap();
        assert_eq!(engine.phase(), PublicPhase::Victory);
        let v = engine.view_for_player("p1").unwrap().victory.unwrap();
        assert_eq!(v.winner, Faction::Infiltrators);
        assert_eq!(v.reason, VictoryReason::InfiltratorsUnmaskedKanka);
    }
}
