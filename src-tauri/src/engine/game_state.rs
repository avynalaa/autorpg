use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{
    character::Character,
    inventory::Inventory,
    combat::Combat,
    quest::QuestLog,
    relations::RelationMap,
    memory::MemoryLog,
    map::WorldMap,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GamePhase {
    MainMenu,
    CharacterCreation,
    Playing,
    Combat,
    GameOver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub phase: GamePhase,
    pub player: Character,
    pub inventory: Inventory,
    pub combat: Combat,
    pub quests: QuestLog,
    pub relations: RelationMap,
    pub memory: MemoryLog,
    pub world: WorldMap,

    /// Current scene/context title
    pub scene: String,
    /// Weather/environment
    pub weather: String,
    /// Running story log (displayed in chat panel)
    pub story_log: Vec<StoryEntry>,
    /// Pending choices presented to player
    pub choices: Vec<PlayerChoice>,
    /// World variables (flags, counters for the story)
    pub flags: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryEntry {
    pub id: u32,
    pub source: EntrySource,
    pub text: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntrySource {
    Narrator,   // AI response
    Player,     // player action/input
    System,     // dice rolls, combat results, etc.
    Internal,   // engine messages
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerChoice {
    pub id: String,
    pub text: String,
    pub style: ChoiceStyle,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChoiceStyle {
    Normal,
    Danger,
    Social,
    Skill(String),  // skill check required
    Special,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            phase: GamePhase::MainMenu,
            player: Character::new("player", "Hero"),
            inventory: Inventory::new(24),
            combat: Combat::new(),
            quests: QuestLog::default(),
            relations: RelationMap::default(),
            memory: MemoryLog::default(),
            world: WorldMap::default(),
            scene: "Unknown".to_string(),
            weather: "clear".to_string(),
            story_log: Vec::new(),
            choices: Vec::new(),
            flags: HashMap::new(),
        }
    }

    /// Build the compact context string sent to AI before every message
    /// Goal: minimal tokens while giving AI everything it needs
    pub fn build_ai_context(&self) -> String {
        let mut parts = Vec::new();

        // Scene & time
        parts.push(format!("[SCENE:{}|{}|{}]", self.scene, self.weather, self.memory.current_time.time_of_day()));

        // Time
        parts.push(self.memory.current_time.to_context_string().replace("[TIME:", "").trim_end_matches(']').to_string());
        // Actually keep full format
        parts.push(format!("[TIME:{}]", self.memory.current_time.to_context_string()));

        // Player character
        parts.push(self.player.to_context_string());

        // Inventory
        parts.push(self.inventory.to_context_string());

        // Combat (only if active)
        let combat_ctx = self.combat.to_context_string();
        if !combat_ctx.is_empty() {
            parts.push(combat_ctx);
        }

        // Quests
        parts.push(self.quests.to_context_string());

        // Relations
        parts.push(self.relations.to_context_string());

        // Location
        parts.push(self.world.to_context_string());

        // Memory/recent events
        parts.push(self.memory.to_context_string());

        parts.join("")
    }

    pub fn add_story_entry(&mut self, source: EntrySource, text: &str) {
        let id = self.story_log.len() as u32;
        let ts = self.memory.current_time.to_context_string();
        self.story_log.push(StoryEntry {
            id,
            source,
            text: text.to_string(),
            timestamp: ts,
        });
        // Cap log at 200 entries
        if self.story_log.len() > 200 {
            self.story_log.remove(0);
        }
    }

    /// Remove the last player turn and everything after it from the story log.
    /// Used by regenerate to roll back before re-sending.
    pub fn trim_last_turn(&mut self) {
        if let Some(idx) = self.story_log.iter().rposition(|e| e.source == EntrySource::Player) {
            self.story_log.truncate(idx);
        }
    }

    pub fn set_choices(&mut self, choices: Vec<PlayerChoice>) {
        self.choices = choices;
    }

    pub fn clear_choices(&mut self) {
        self.choices.clear();
    }

    pub fn set_flag(&mut self, key: &str, value: serde_json::Value) {
        self.flags.insert(key.to_string(), value);
    }

    pub fn get_flag(&self, key: &str) -> Option<&serde_json::Value> {
        self.flags.get(key)
    }

    pub fn flag_is_true(&self, key: &str) -> bool {
        self.flags.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
    }
}

impl Default for GameState {
    fn default() -> Self { Self::new() }
}
