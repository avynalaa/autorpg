use serde::{Deserialize, Serialize};
use super::dice;
use super::character::Character;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CombatState {
    OutOfCombat,
    Initiative,  // determining order
    PlayerTurn,
    EnemyTurn,
    Victory,
    Defeat,
    Fled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combatant {
    pub id: String,
    pub name: String,
    pub hp: i32,
    pub hp_max: i32,
    pub attack: i32,
    pub defense: i32,    // armor class equivalent
    pub initiative: i32,
    pub is_player: bool,
    pub is_alive: bool,
    pub tags: Vec<String>, // e.g. ["undead", "boss", "flying"]
}

impl Combatant {
    pub fn from_character(c: &Character) -> Self {
        let attack = c.ability_scores.str_mod() + c.proficiency_bonus();
        let defense = 10 + c.ability_scores.dex_mod();
        Self {
            id: c.id.clone(),
            name: c.name.clone(),
            hp: c.hp,
            hp_max: c.hp_max,
            attack,
            defense,
            initiative: 0,
            is_player: true,
            is_alive: c.is_alive(),
            tags: Vec::new(),
        }
    }

    pub fn new_enemy(id: &str, name: &str, hp: i32, attack: i32, defense: i32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            hp,
            hp_max: hp,
            attack,
            defense,
            initiative: 0,
            is_player: false,
            is_alive: true,
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatAction {
    pub actor_id: String,
    pub action_type: String, // "attack", "spell", "item", "flee", "defend"
    pub target_id: Option<String>,
    pub item_id: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatResult {
    pub hit: bool,
    pub damage: i32,
    pub roll: i32,
    pub vs_defense: i32,
    pub description: String,
    pub critical: bool,
    pub miss: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatLog {
    pub entries: Vec<String>,
}

impl CombatLog {
    pub fn new() -> Self { Self { entries: Vec::new() } }
    pub fn log(&mut self, msg: &str) { self.entries.push(msg.to_string()); }
    pub fn recent(&self, n: usize) -> Vec<&String> {
        let start = self.entries.len().saturating_sub(n);
        self.entries[start..].iter().collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combat {
    pub state: CombatState,
    pub combatants: Vec<Combatant>,
    pub turn_order: Vec<String>, // combatant ids in initiative order
    pub current_turn_idx: usize,
    pub round: u32,
    pub log: CombatLog,
}

impl Combat {
    pub fn new() -> Self {
        Self {
            state: CombatState::OutOfCombat,
            combatants: Vec::new(),
            turn_order: Vec::new(),
            current_turn_idx: 0,
            round: 0,
            log: CombatLog::new(),
        }
    }

    pub fn start(&mut self, combatants: Vec<Combatant>) {
        self.combatants = combatants;
        self.round = 1;
        self.state = CombatState::Initiative;
        self.roll_initiative();
    }

    pub fn roll_initiative(&mut self) {
        for c in &mut self.combatants {
            let roll = dice::roll("d20");
            c.initiative = roll.total;
        }
        // Sort by initiative descending
        let mut order: Vec<(String, i32)> = self.combatants.iter()
            .map(|c| (c.id.clone(), c.initiative))
            .collect();
        order.sort_by(|a, b| b.1.cmp(&a.1));
        self.turn_order = order.into_iter().map(|(id, _)| id).collect();

        self.current_turn_idx = 0;
        self.log.log(&format!("=== Round {} ===", self.round));

        for id in &self.turn_order {
            if let Some(c) = self.combatants.iter().find(|c| &c.id == id) {
                self.log.log(&format!("{} initiative: {}", c.name, c.initiative));
            }
        }

        self.advance_to_next_turn();
    }

    pub fn advance_to_next_turn(&mut self) {
        // Skip dead combatants
        let len = self.turn_order.len();
        for _ in 0..len {
            if let Some(id) = self.turn_order.get(self.current_turn_idx) {
                if let Some(c) = self.combatants.iter().find(|c| &c.id == id) {
                    if c.is_alive {
                        if c.is_player {
                            self.state = CombatState::PlayerTurn;
                        } else {
                            self.state = CombatState::EnemyTurn;
                        }
                        return;
                    }
                }
            }
            self.current_turn_idx = (self.current_turn_idx + 1) % len;
        }
    }

    pub fn resolve_attack(&mut self, attacker_id: &str, target_id: &str) -> CombatResult {
        let attacker = self.combatants.iter().find(|c| c.id == attacker_id).cloned();
        let target = self.combatants.iter().find(|c| c.id == target_id).cloned();

        let (Some(attacker), Some(target_snapshot)) = (attacker, target) else {
            return CombatResult {
                hit: false, damage: 0, roll: 0,
                vs_defense: 0, description: "The attack goes nowhere.".to_string(),
                critical: false, miss: true,
            };
        };

        let attack_roll = dice::roll("d20");
        let total_attack = attack_roll.total + attacker.attack;
        let target_def = target_snapshot.defense;
        let critical = attack_roll.rolls.first() == Some(&20);
        let fumble = attack_roll.rolls.first() == Some(&1);

        if fumble {
            let msg = format!("{} overcommits and leaves an opening instead of landing the blow.", attacker.name);
            self.log.log(&msg);
            return CombatResult { hit: false, damage: 0, roll: 1, vs_defense: target_def, description: msg, critical: false, miss: true };
        }

        if critical || total_attack >= target_def {
            let damage_dice = if critical { "2d6" } else { "1d6" };
            let dmg_roll = dice::roll(damage_dice);
            let damage = (dmg_roll.total + attacker.attack / 2).max(1);
            let target_name = target_snapshot.name.clone();

            let mut defeated = false;
            if let Some(target) = self.combatants.iter_mut().find(|c| c.id == target_id) {
                target.hp -= damage;
                if target.hp <= 0 {
                    target.hp = 0;
                    target.is_alive = false;
                    defeated = true;
                }
            }

            let msg = if critical {
                if defeated {
                    format!("{} drives a crushing blow into {}, dropping them where they stand.", attacker.name, target_name)
                } else {
                    format!("{} lands a brutal strike on {}, tearing through their guard for {} damage.", attacker.name, target_name, damage)
                }
            } else if defeated {
                format!("{} strikes true and drops {} with a final hit.", attacker.name, target_name)
            } else {
                format!("{} hits {} for {} damage.", attacker.name, target_name, damage)
            };
            self.log.log(&msg);

            CombatResult { hit: true, damage, roll: total_attack, vs_defense: target_def, description: msg, critical, miss: false }
        } else {
            let msg = format!("{} lashes out, but {} slips clear of the attack.", attacker.name, target_snapshot.name);
            self.log.log(&msg);
            CombatResult { hit: false, damage: 0, roll: total_attack, vs_defense: target_def, description: msg, critical: false, miss: true }
        }
    }

    pub fn end_turn(&mut self) {
        let len = self.turn_order.len();
        self.current_turn_idx = (self.current_turn_idx + 1) % len;

        // Check if new round
        if self.current_turn_idx == 0 {
            self.round += 1;
            self.log.log(&format!("=== Round {} ===", self.round));
        }

        // Check victory/defeat
        let players_alive = self.combatants.iter().any(|c| c.is_player && c.is_alive);
        let enemies_alive = self.combatants.iter().any(|c| !c.is_player && c.is_alive);

        if !players_alive {
            self.state = CombatState::Defeat;
        } else if !enemies_alive {
            self.state = CombatState::Victory;
        } else {
            self.advance_to_next_turn();
        }
    }

    /// Simple AI: attack a random living player
    pub fn enemy_auto_turn(&mut self, enemy_id: &str) -> Option<CombatResult> {
        let player_ids: Vec<String> = self.combatants.iter()
            .filter(|c| c.is_player && c.is_alive)
            .map(|c| c.id.clone())
            .collect();

        if player_ids.is_empty() { return None; }

        let target_id = player_ids[0].clone();
        let result = self.resolve_attack(enemy_id, &target_id);
        self.end_turn();
        Some(result)
    }

    /// Compact context for AI narrator
    pub fn to_context_string(&self) -> String {
        if self.state == CombatState::OutOfCombat { return String::new(); }
        let combatants: Vec<String> = self.combatants.iter()
            .map(|c| format!("{}:{}/{}", c.name, c.hp, c.hp_max))
            .collect();
        format!("[COMBAT:Round{}|{}|{}]",
            self.round,
            format!("{:?}", self.state),
            combatants.join(","))
    }
}
