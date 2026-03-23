use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiceRoll {
    pub dice: String,     // e.g. "2d6+3"
    pub rolls: Vec<u32>,
    pub modifier: i32,
    pub total: i32,
    pub label: Option<String>,
}

pub fn roll(dice_str: &str) -> DiceRoll {
    // Parse formats: d20, 2d6, d8+2, 3d4-1
    let input = dice_str.trim().to_lowercase();
    let mut rng = rand::thread_rng();

    let (dice_part, modifier) = if let Some(pos) = input.rfind('+') {
        let (d, m) = input.split_at(pos);
        (d, m[1..].parse::<i32>().unwrap_or(0))
    } else if let Some(pos) = input.rfind('-') {
        let (d, m) = input.split_at(pos);
        (d, -m[1..].parse::<i32>().unwrap_or(0))
    } else {
        (input.as_str(), 0)
    };

    let (count, sides) = if let Some(d_pos) = dice_part.find('d') {
        let count = dice_part[..d_pos].parse::<u32>().unwrap_or(1).max(1);
        let sides = dice_part[d_pos + 1..].parse::<u32>().unwrap_or(6).max(1);
        (count, sides)
    } else {
        (1, dice_part.parse::<u32>().unwrap_or(6).max(1))
    };

    let rolls: Vec<u32> = (0..count).map(|_| rng.gen_range(1..=sides)).collect();
    let sum: i32 = rolls.iter().map(|&r| r as i32).sum::<i32>() + modifier;

    DiceRoll {
        dice: dice_str.to_string(),
        rolls,
        modifier,
        total: sum.max(0),
        label: None,
    }
}

pub fn roll_with_label(dice_str: &str, label: &str) -> DiceRoll {
    let mut r = roll(dice_str);
    r.label = Some(label.to_string());
    r
}

/// Returns true if total >= difficulty
pub fn check(dice_str: &str, difficulty: i32) -> (DiceRoll, bool) {
    let r = roll(dice_str);
    let success = r.total >= difficulty;
    (r, success)
}

/// Advantage: roll twice, take higher
pub fn roll_advantage(sides: u32) -> DiceRoll {
    let mut rng = rand::thread_rng();
    let a = rng.gen_range(1..=sides);
    let b = rng.gen_range(1..=sides);
    let higher = a.max(b);
    DiceRoll {
        dice: format!("d{}_adv", sides),
        rolls: vec![a, b],
        modifier: 0,
        total: higher as i32,
        label: Some("advantage".to_string()),
    }
}

/// Disadvantage: roll twice, take lower
pub fn roll_disadvantage(sides: u32) -> DiceRoll {
    let mut rng = rand::thread_rng();
    let a = rng.gen_range(1..=sides);
    let b = rng.gen_range(1..=sides);
    let lower = a.min(b);
    DiceRoll {
        dice: format!("d{}_dis", sides),
        rolls: vec![a, b],
        modifier: 0,
        total: lower as i32,
        label: Some("disadvantage".to_string()),
    }
}
