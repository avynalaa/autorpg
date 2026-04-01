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

impl CharacterClass {
    pub fn display_name(&self) -> String {
        match self {
            CharacterClass::Warrior => "Warrior".to_string(),
            CharacterClass::Mage    => "Mage".to_string(),
            CharacterClass::Rogue   => "Rogue".to_string(),
            CharacterClass::Cleric  => "Cleric".to_string(),
            CharacterClass::Ranger  => "Ranger".to_string(),
            CharacterClass::Bard    => "Bard".to_string(),
            CharacterClass::Custom(s) => s.clone(),
        }
    }

    pub fn preset_scores(&self) -> AbilityScores {
        match self {
            CharacterClass::Warrior => AbilityScores { strength: 16, dexterity: 12, constitution: 14, intelligence: 8,  wisdom: 10, charisma: 10 },
            CharacterClass::Mage    => AbilityScores { strength: 8,  dexterity: 12, constitution: 10, intelligence: 16, wisdom: 14, charisma: 10 },
            CharacterClass::Rogue   => AbilityScores { strength: 10, dexterity: 16, constitution: 12, intelligence: 12, wisdom: 10, charisma: 12 },
            CharacterClass::Cleric  => AbilityScores { strength: 12, dexterity: 10, constitution: 12, intelligence: 10, wisdom: 16, charisma: 12 },
            CharacterClass::Ranger  => AbilityScores { strength: 12, dexterity: 16, constitution: 12, intelligence: 10, wisdom: 14, charisma: 8  },
            CharacterClass::Bard    => AbilityScores { strength: 8,  dexterity: 14, constitution: 10, intelligence: 12, wisdom: 10, charisma: 16 },
            CharacterClass::Custom(_) => AbilityScores::default(),
        }
    }

    pub fn base_hp(&self) -> i32 {
        match self {
            CharacterClass::Warrior => 12,
            CharacterClass::Mage    => 6,
            CharacterClass::Rogue   => 8,
            CharacterClass::Cleric  => 8,
            CharacterClass::Ranger  => 9,
            CharacterClass::Bard    => 7,
            CharacterClass::Custom(_) => 10,
        }
    }

    pub fn base_mp(&self, scores: &AbilityScores) -> i32 {
        match self {
            CharacterClass::Mage   => (12 + scores.int_mod()).max(0),
            CharacterClass::Cleric => (10 + scores.wis_mod()).max(0),
            CharacterClass::Bard   => (8  + scores.cha_mod()).max(0),
            _ => 0,
        }
    }
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

impl CharacterRace {
    pub fn display_name(&self) -> String {
        match self {
            CharacterRace::Human    => "Human".to_string(),
            CharacterRace::Elf      => "Elf".to_string(),
            CharacterRace::Dwarf    => "Dwarf".to_string(),
            CharacterRace::Halfling => "Halfling".to_string(),
            CharacterRace::Orc      => "Orc".to_string(),
            CharacterRace::Custom(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub rank: i32,
    pub ability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEffect {
    pub id: String,
    pub name: String,
    pub duration: Option<i32>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub name: String,           // first name (display name)
    pub last_name: Option<String>,
    pub age: Option<u32>,
    pub gender: String,
    pub sex: String,
    pub appearance: String,
    pub personality: String,
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
    pub portrait: Option<String>,
    pub backstory: String,
    pub notes: String,

    pub custom_stats: HashMap<String, i32>,
}

impl Character {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            last_name: None,
            age: None,
            gender: String::new(),
            sex: String::new(),
            appearance: String::new(),
            personality: String::new(),
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

    /// Full name (first + last if present)
    pub fn full_name(&self) -> String {
        match &self.last_name {
            Some(ln) if !ln.is_empty() => format!("{} {}", self.name, ln),
            _ => self.name.clone(),
        }
    }

    /// Compact context string for AI
    pub fn to_context_string(&self) -> String {
        let scores = &self.ability_scores;
        let name_str = match &self.last_name {
            Some(ln) if !ln.is_empty() => format!("{} {}", self.name, ln),
            _ => self.name.clone(),
        };
        let mut s = format!(
            "[PC:{name}|{race}_{class}|LVL:{lvl}|HP:{hp}/{hpmax}|MP:{mp}/{mpmax}|PURSE:{purse}|STR:{str}|DEX:{dex}|CON:{con}|INT:{int}|WIS:{wis}|CHA:{cha}]",
            name  = name_str,
            race  = self.race.display_name(),
            class = self.class.display_name(),
            lvl   = self.level,
            hp    = self.hp,
            hpmax = self.hp_max,
            mp    = self.mp,
            mpmax = self.mp_max,
            purse = self.purse.to_context_string(),
            str   = scores.strength,
            dex   = scores.dexterity,
            con   = scores.constitution,
            int   = scores.intelligence,
            wis   = scores.wisdom,
            cha   = scores.charisma,
        );

        // Persona block — only if any field is set
        let has_persona = self.age.is_some()
            || !self.gender.is_empty()
            || !self.sex.is_empty()
            || !self.appearance.is_empty()
            || !self.personality.is_empty();

        if has_persona {
            let mut parts: Vec<String> = Vec::new();
            if let Some(age) = self.age { parts.push(format!("age={}", age)); }
            if !self.gender.is_empty()      { parts.push(format!("gender={}", self.gender)); }
            if !self.sex.is_empty()         { parts.push(format!("sex={}", self.sex)); }
            if !self.appearance.is_empty()  { parts.push(format!("appearance={}", self.appearance)); }
            if !self.personality.is_empty() { parts.push(format!("personality={}", self.personality)); }
            s.push_str(&format!("[PERSONA:{}]", parts.join("|")));
        }

        if !self.backstory.is_empty() {
            s.push_str(&format!("[BACKSTORY:{}]", self.backstory));
        }

        s
    }

    pub fn check_level_up(&mut self) -> bool {
        if self.experience >= self.experience_to_next {
            self.level += 1;
            self.experience -= self.experience_to_next;
            self.experience_to_next = (self.experience_to_next as f32 * 1.5) as u32;
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
