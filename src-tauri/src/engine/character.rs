use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::currency::Purse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityScores {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

impl AbilityScores {
    pub fn modifier(score: i32) -> i32 {
        (score - 10) / 2
    }

    pub fn str_mod(&self) -> i32 { Self::modifier(self.strength) }
    pub fn dex_mod(&self) -> i32 { Self::modifier(self.dexterity) }
    pub fn con_mod(&self) -> i32 { Self::modifier(self.constitution) }
    pub fn int_mod(&self) -> i32 { Self::modifier(self.intelligence) }
    pub fn wis_mod(&self) -> i32 { Self::modifier(self.wisdom) }
    pub fn cha_mod(&self) -> i32 { Self::modifier(self.charisma) }
}

impl Default for AbilityScores {
    fn default() -> Self {
        Self { strength: 10, dexterity: 10, constitution: 10, intelligence: 10, wisdom: 10, charisma: 10 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CharacterClass {
    Warrior,
    Mage,
    Rogue,
    Cleric,
    Ranger,
    Bard,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CharacterRace {
    Human,
    Elf,
    Dwarf,
    Halfling,
    Orc,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub rank: i32,       // 0-5
    pub ability: String, // which ability drives it
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEffect {
    pub id: String,
    pub name: String,
    pub duration: Option<i32>, // None = permanent until removed
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub race: CharacterRace,
    pub class: CharacterClass,
    pub level: u32,
    pub experience: u32,
    pub experience_to_next: u32,

    pub hp: i32,
    pub hp_max: i32,
    pub mp: i32,
    pub mp_max: i32,
    pub stamina: i32,
    pub stamina_max: i32,

    pub ability_scores: AbilityScores,
    pub skills: Vec<Skill>,
    pub status_effects: Vec<StatusEffect>,

    pub purse: Purse,
    pub portrait: Option<String>, // path or base64
    pub backstory: String,
    pub notes: String, // AI-readable short notes

    // Extra named attributes (e.g. "hunger", "sanity")
    pub custom_stats: HashMap<String, i32>,
}

impl Character {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            race: CharacterRace::Human,
            class: CharacterClass::Warrior,
            level: 1,
            experience: 0,
            experience_to_next: 100,
            hp: 10,
            hp_max: 10,
            mp: 0,
            mp_max: 0,
            stamina: 10,
            stamina_max: 10,
            ability_scores: AbilityScores::default(),
            skills: Vec::new(),
            status_effects: Vec::new(),
            purse: Purse::default(),
            portrait: None,
            backstory: String::new(),
            notes: String::new(),
            custom_stats: HashMap::new(),
        }
    }

    /// Compact context string for AI (minimal tokens)
    pub fn to_context_string(&self) -> String {
        let scores = &self.ability_scores;
        format!(
            "[PC:{name}|{race:?}_{class:?}|LVL:{lvl}|HP:{hp}/{hpmax}|MP:{mp}/{mpmax}|PURSE:{purse}|STR:{str}|DEX:{dex}|CON:{con}|INT:{int}|WIS:{wis}|CHA:{cha}]",
            name = self.name,
            race = self.race,
            class = self.class,
            lvl = self.level,
            hp = self.hp,
            hpmax = self.hp_max,
            mp = self.mp,
            mpmax = self.mp_max,
            purse = self.purse.to_context_string(),
            str = scores.strength,
            dex = scores.dexterity,
            con = scores.constitution,
            int = scores.intelligence,
            wis = scores.wisdom,
            cha = scores.charisma,
        )
    }

    /// Attempt to level up if experience threshold met
    pub fn check_level_up(&mut self) -> bool {
        if self.experience >= self.experience_to_next {
            self.level += 1;
            self.experience -= self.experience_to_next;
            self.experience_to_next = (self.experience_to_next as f32 * 1.5) as u32;
            // HP/MP increase on level up
            let con_mod = self.ability_scores.con_mod();
            self.hp_max += 4 + con_mod;
            self.hp = self.hp_max;
            true
        } else {
            false
        }
    }

    pub fn add_experience(&mut self, amount: u32) -> bool {
        self.experience += amount;
        self.check_level_up()
    }

    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.hp_max);
    }

    pub fn damage(&mut self, amount: i32) {
        self.hp = (self.hp - amount).max(0);
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn proficiency_bonus(&self) -> i32 {
        1 + (self.level as i32 / 4).max(1)
    }
}
