use serde::{Deserialize, Serialize};

/// A single event/memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    pub id: String,
    pub timestamp: GameTime,
    pub category: MemoryCategory,
    pub description: String,
    pub tags: Vec<String>,
    pub importance: u8, // 1-5, used to filter context
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryCategory {
    Plot,       // main story events
    Npc,        // NPC interactions
    Combat,     // battle outcomes
    Discovery,  // found locations, items, secrets
    Choice,     // player choices and consequences
    World,      // world-state changes
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameTime {
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub season: Season,
    pub year: u32,
}

impl GameTime {
    pub fn advance_minutes(&mut self, minutes: u32) {
        self.minute += minutes;
        while self.minute >= 60 {
            self.minute -= 60;
            self.hour += 1;
        }
        while self.hour >= 24 {
            self.hour -= 24;
            self.day += 1;
        }
        while self.day >= 90 { // 3 seasons of 30 days each, 4 seasons = 120 days/year
            self.day -= 90;
            self.year += 1;
        }
    }

    pub fn time_of_day(&self) -> &'static str {
        match self.hour {
            0..=5   => "night",
            6..=11  => "morning",
            12..=17 => "afternoon",
            18..=20 => "evening",
            _       => "night",
        }
    }

    pub fn to_context_string(&self) -> String {
        format!("Day{}.{:02}:{:02}|{:?}|Y{}", self.day, self.hour, self.minute, self.season, self.year)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryLog {
    pub events: Vec<MemoryEvent>,
    pub current_time: GameTime,
    next_id: u32,
}

impl MemoryLog {
    pub fn record(&mut self, category: MemoryCategory, description: &str, importance: u8, tags: Vec<String>) {
        let event = MemoryEvent {
            id: format!("ev{}", self.next_id),
            timestamp: self.current_time.clone(),
            category,
            description: description.to_string(),
            tags,
            importance: importance.clamp(1, 5),
        };
        self.next_id += 1;
        self.events.push(event);

        // Cap at 500 events — remove oldest low-importance
        if self.events.len() > 500 {
            // Remove first event with importance <= 2
            if let Some(pos) = self.events.iter().position(|e| e.importance <= 2) {
                self.events.remove(pos);
            } else {
                self.events.remove(0);
            }
        }
    }

    pub fn advance_time(&mut self, minutes: u32) {
        self.current_time.advance_minutes(minutes);
    }

    /// Get recent important events for AI context
    pub fn recent_for_context(&self, max_events: usize, min_importance: u8) -> Vec<&MemoryEvent> {
        let mut filtered: Vec<&MemoryEvent> = self.events.iter()
            .filter(|e| e.importance >= min_importance)
            .collect();
        let start = filtered.len().saturating_sub(max_events);
        filtered[start..].to_vec()
    }

    /// Compact context for AI
    pub fn to_context_string(&self) -> String {
        let time = self.current_time.to_context_string();
        let recent: Vec<String> = self.recent_for_context(5, 3)
            .iter()
            .map(|e| format!("[{}]", e.description))
            .collect();
        format!("[TIME:{}][MEM:{}]", time, recent.join(""))
    }
}
