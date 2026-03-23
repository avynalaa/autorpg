use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationTier {
    Hostile,     // < -50
    Unfriendly,  // -50 to -1
    Neutral,     // 0 to 24
    Friendly,    // 25 to 59
    Trusted,     // 60 to 89
    Allied,      // >= 90
}

impl RelationTier {
    pub fn from_score(score: i32) -> Self {
        match score {
            s if s < -50 => Self::Hostile,
            s if s < 0   => Self::Unfriendly,
            s if s < 25  => Self::Neutral,
            s if s < 60  => Self::Friendly,
            s if s < 90  => Self::Trusted,
            _            => Self::Allied,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcRelation {
    pub npc_id: String,
    pub npc_name: String,
    pub score: i32,        // -100 to 100
    pub tier: RelationTier,
    pub notes: Vec<String>, // memorable events
    pub last_met: Option<String>, // location/time string
}

impl NpcRelation {
    pub fn new(npc_id: &str, npc_name: &str) -> Self {
        Self {
            npc_id: npc_id.to_string(),
            npc_name: npc_name.to_string(),
            score: 0,
            tier: RelationTier::Neutral,
            notes: Vec::new(),
            last_met: None,
        }
    }

    pub fn adjust(&mut self, delta: i32, reason: Option<&str>) {
        self.score = (self.score + delta).clamp(-100, 100);
        self.tier = RelationTier::from_score(self.score);
        if let Some(r) = reason {
            self.notes.push(r.to_string());
            // Keep last 10 notes
            if self.notes.len() > 10 {
                self.notes.remove(0);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelationMap {
    pub relations: HashMap<String, NpcRelation>,
}

impl RelationMap {
    pub fn get_or_create(&mut self, npc_id: &str, npc_name: &str) -> &mut NpcRelation {
        self.relations
            .entry(npc_id.to_string())
            .or_insert_with(|| NpcRelation::new(npc_id, npc_name))
    }

    pub fn adjust(&mut self, npc_id: &str, npc_name: &str, delta: i32, reason: Option<&str>) {
        let r = self.get_or_create(npc_id, npc_name);
        r.adjust(delta, reason);
    }

    pub fn tier_of(&self, npc_id: &str) -> RelationTier {
        self.relations.get(npc_id)
            .map(|r| r.tier.clone())
            .unwrap_or(RelationTier::Neutral)
    }

    /// Compact context for AI — only non-neutral relations
    pub fn to_context_string(&self) -> String {
        let notable: Vec<String> = self.relations.values()
            .filter(|r| r.score.abs() >= 10)
            .map(|r| format!("{}:{:?}({})", r.npc_id, r.tier, r.score))
            .collect();
        if notable.is_empty() {
            "[REL:none]".to_string()
        } else {
            format!("[REL:{}]", notable.join(","))
        }
    }
}
