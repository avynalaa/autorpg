use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub description: String,
    pub location_type: LocationType,
    pub visited: bool,
    pub discovered: bool,
    pub connections: Vec<String>, // adjacent location ids
    pub npcs: Vec<String>,        // NPC ids present here
    pub items: Vec<String>,       // Item ids on the ground
    pub notes: Vec<String>,
    pub tags: Vec<String>,        // e.g. ["dungeon", "safe_rest", "hostile"]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LocationType {
    Town,
    Dungeon,
    Wilderness,
    Building,
    Road,
    Cave,
    Ruins,
    Custom(String),
}

impl Location {
    pub fn new(id: &str, name: &str, location_type: LocationType) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            location_type,
            visited: false,
            discovered: true,
            connections: Vec::new(),
            npcs: Vec::new(),
            items: Vec::new(),
            notes: Vec::new(),
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldMap {
    pub locations: HashMap<String, Location>,
    pub current_location: String,
    pub previous_location: Option<String>,
}

impl WorldMap {
    pub fn add_location(&mut self, location: Location) {
        self.locations.insert(location.id.clone(), location);
    }

    pub fn move_to(&mut self, location_id: &str) -> Result<(), String> {
        if !self.locations.contains_key(location_id) {
            return Err(format!("Location '{}' not found", location_id));
        }
        let prev = self.current_location.clone();
        self.previous_location = if prev.is_empty() { None } else { Some(prev) };
        self.current_location = location_id.to_string();
        if let Some(loc) = self.locations.get_mut(location_id) {
            loc.visited = true;
        }
        Ok(())
    }

    pub fn current(&self) -> Option<&Location> {
        self.locations.get(&self.current_location)
    }

    pub fn current_mut(&mut self) -> Option<&mut Location> {
        self.locations.get_mut(&self.current_location)
    }

    pub fn exits(&self) -> Vec<String> {
        self.current()
            .map(|l| l.connections.clone())
            .unwrap_or_default()
    }

    /// Compact context for AI
    pub fn to_context_string(&self) -> String {
        let loc = self.current();
        match loc {
            Some(l) => {
                let exits: Vec<String> = l.connections.iter()
                    .filter_map(|id| self.locations.get(id))
                    .map(|l| l.name.clone())
                    .collect();
                format!("[LOC:{}|{:?}|exits:{}]",
                    l.name,
                    l.location_type,
                    if exits.is_empty() { "none".to_string() } else { exits.join(",") })
            }
            None => "[LOC:unknown]".to_string(),
        }
    }
}
