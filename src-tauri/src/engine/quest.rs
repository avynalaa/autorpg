use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuestStatus {
    NotStarted,
    Active,
    Completed,
    Failed,
    OnHold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    pub id: String,
    pub description: String,
    pub completed: bool,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: QuestStatus,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<String>, // descriptions of rewards
    pub giver: Option<String>, // NPC id
    pub location: Option<String>,
    pub notes: Vec<String>, // running notes, updated by AI
}

impl Quest {
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: String::new(),
            status: QuestStatus::NotStarted,
            objectives: Vec::new(),
            rewards: Vec::new(),
            giver: None,
            location: None,
            notes: Vec::new(),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.objectives.iter().filter(|o| !o.optional).all(|o| o.completed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuestLog {
    pub quests: Vec<Quest>,
}

impl QuestLog {
    pub fn add_quest(&mut self, quest: Quest) {
        if !self.quests.iter().any(|q| q.id == quest.id) {
            self.quests.push(quest);
        }
    }

    pub fn get_mut(&mut self, quest_id: &str) -> Option<&mut Quest> {
        self.quests.iter_mut().find(|q| q.id == quest_id)
    }

    pub fn set_status(&mut self, quest_id: &str, status: QuestStatus) {
        if let Some(q) = self.get_mut(quest_id) {
            q.status = status;
        }
    }

    pub fn complete_objective(&mut self, quest_id: &str, obj_id: &str) {
        if let Some(quest) = self.get_mut(quest_id) {
            if let Some(obj) = quest.objectives.iter_mut().find(|o| o.id == obj_id) {
                obj.completed = true;
            }
            // Auto-complete quest if all objectives done
            if quest.is_complete() {
                quest.status = QuestStatus::Completed;
            }
        }
    }

    pub fn active_quests(&self) -> Vec<&Quest> {
        self.quests.iter().filter(|q| q.status == QuestStatus::Active).collect()
    }

    /// Compact context for AI
    pub fn to_context_string(&self) -> String {
        let active: Vec<String> = self.active_quests().iter()
            .map(|q| {
                let done: usize = q.objectives.iter().filter(|o| o.completed).count();
                let total = q.objectives.len();
                format!("{}({}/{})", q.id, done, total)
            })
            .collect();
        if active.is_empty() {
            "[QUESTS:none]".to_string()
        } else {
            format!("[QUESTS:{}]", active.join(","))
        }
    }
}
