// Game logic modules
pub mod factions;
pub mod units;
pub mod map;
pub mod combat;
pub mod diplomacy;
pub mod research;

// Core game state
#[derive(Debug, Clone)]
pub struct GameState {
    pub current_turn: u32,
    pub active_faction: String,
    pub game_speed: GameSpeed,
    pub difficulty: Difficulty,
}

#[derive(Debug, Clone)]
pub enum GameSpeed {
    Slow,
    Normal,
    Fast,
}

#[derive(Debug, Clone)]
pub enum Difficulty {
    Citizen,
    Specialist,
    Talent,
    Librarian,
    Thinker,
    Transcend,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            current_turn: 0,
            active_faction: "GAIANS".to_string(),
            game_speed: GameSpeed::Normal,
            difficulty: Difficulty::Citizen,
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn advance_turn(&mut self) {
        self.current_turn += 1;
        // TODO: Process end of turn logic
        // - Update all factions
        // - Process movement
        // - Handle combat
        // - Update diplomacy
        // - Advance research
    }
    
    pub fn get_current_year(&self) -> u32 {
        // SMAC starts in 2100
        2100 + self.current_turn
    }
}
