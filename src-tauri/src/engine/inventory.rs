use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Consumable,
    Quest,
    Misc,
    Tool,
    Key,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EquipSlot {
    Head,
    Chest,
    Legs,
    Feet,
    Hands,
    MainHand,
    OffHand,
    Ring,
    Neck,
    Back,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub category: ItemCategory,
    pub description: String,
    pub weight: f32,
    pub value: u32, // in gold pieces
    pub quantity: u32,
    pub stackable: bool,
    pub equip_slot: Option<EquipSlot>,
    /// Stat modifiers when equipped: e.g. {"attack": 5, "defense": 2}
    pub stat_mods: std::collections::HashMap<String, i32>,
    pub tags: Vec<String>, // e.g. ["magical", "cursed", "unique"]
}

impl Item {
    pub fn new(id: &str, name: &str, category: ItemCategory) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            category,
            description: String::new(),
            weight: 1.0,
            value: 0,
            quantity: 1,
            stackable: false,
            equip_slot: None,
            stat_mods: std::collections::HashMap::new(),
            tags: Vec::new(),
        }
    }
}

/// A single inventory slot. Locked = cannot be overwritten by AI command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub index: usize,
    pub item: Option<Item>,
    pub locked: bool, // true when occupied — AI cannot overwrite
    pub reserved: bool, // reserved for a specific item type
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<InventorySlot>,
    pub capacity: usize,
    pub max_weight: f32,
    /// Equipment slots (separate from backpack)
    pub equipped: std::collections::HashMap<String, Item>,
}

impl Inventory {
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: (0..capacity).map(|i| InventorySlot {
                index: i,
                item: None,
                locked: false,
                reserved: false,
            }).collect(),
            capacity,
            max_weight: 50.0,
            equipped: std::collections::HashMap::new(),
        }
    }

    pub fn current_weight(&self) -> f32 {
        self.slots.iter()
            .filter_map(|s| s.item.as_ref())
            .map(|i| i.weight * i.quantity as f32)
            .sum()
    }

    /// Find first empty, unlocked slot
    pub fn first_free_slot(&self) -> Option<usize> {
        self.slots.iter()
            .find(|s| s.item.is_none() && !s.locked)
            .map(|s| s.index)
    }

    /// Add item — returns slot index or error
    pub fn add_item(&mut self, item: Item) -> Result<usize, String> {
        // Try stacking first
        if item.stackable {
            if let Some(slot) = self.slots.iter_mut()
                .find(|s| s.item.as_ref().map(|i| i.id == item.id).unwrap_or(false))
            {
                if let Some(existing) = &mut slot.item {
                    existing.quantity += item.quantity;
                    return Ok(slot.index);
                }
            }
        }

        if self.current_weight() + item.weight * item.quantity as f32 > self.max_weight {
            return Err("Inventory too heavy".to_string());
        }

        match self.first_free_slot() {
            Some(idx) => {
                self.slots[idx].item = Some(item);
                self.slots[idx].locked = true; // lock the slot
                Ok(idx)
            }
            None => Err("Inventory full".to_string()),
        }
    }

    /// Add item to specific slot — fails if slot is locked or occupied
    pub fn add_item_to_slot(&mut self, slot_idx: usize, item: Item) -> Result<(), String> {
        let slot = self.slots.get_mut(slot_idx).ok_or("Invalid slot")?;
        if slot.locked {
            return Err(format!("Slot {} is locked", slot_idx));
        }
        if slot.item.is_some() {
            return Err(format!("Slot {} is occupied", slot_idx));
        }
        slot.item = Some(item);
        slot.locked = true;
        Ok(())
    }

    /// Remove item by slot index
    pub fn remove_item(&mut self, slot_idx: usize) -> Option<Item> {
        let slot = self.slots.get_mut(slot_idx)?;
        let item = slot.item.take();
        slot.locked = false; // unlock when empty
        item
    }

    /// Remove one quantity of item by id; returns true if removed
    pub fn remove_item_by_id(&mut self, item_id: &str) -> bool {
        for slot in &mut self.slots {
            if let Some(item) = &mut slot.item {
                if item.id == item_id {
                    if item.quantity > 1 {
                        item.quantity -= 1;
                        return true;
                    } else {
                        slot.item = None;
                        slot.locked = false;
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Find item by id
    pub fn find_item(&self, item_id: &str) -> Option<&Item> {
        self.slots.iter()
            .find(|s| s.item.as_ref().map(|i| i.id == item_id).unwrap_or(false))
            .and_then(|s| s.item.as_ref())
    }

    /// Equip item from inventory slot
    pub fn equip_item(&mut self, slot_idx: usize) -> Result<String, String> {
        let item = self.slots.get(slot_idx)
            .and_then(|s| s.item.clone())
            .ok_or("No item in slot")?;

        let equip_slot_key = item.equip_slot.as_ref()
            .map(|s| format!("{:?}", s))
            .ok_or("Item cannot be equipped")?;

        // Unequip existing item in that slot back to inventory
        if let Some(old_item) = self.equipped.remove(&equip_slot_key) {
            self.add_item(old_item)?;
        }

        self.remove_item(slot_idx);
        self.equipped.insert(equip_slot_key.clone(), item);
        Ok(equip_slot_key)
    }

    /// Compact context for AI
    pub fn to_context_string(&self) -> String {
        let items: Vec<String> = self.slots.iter()
            .filter_map(|s| s.item.as_ref())
            .map(|i| {
                if i.quantity > 1 {
                    format!("{}×{}", i.name, i.quantity)
                } else {
                    i.name.clone()
                }
            })
            .collect();

        let equipped: Vec<String> = self.equipped.values()
            .map(|i| format!("{}(E)", i.name))
            .collect();

        let all: Vec<String> = [items, equipped].concat();
        format!("[INV:{}|{:.1}kg/{:.1}kg]", all.join(","), self.current_weight(), self.max_weight)
    }
}
