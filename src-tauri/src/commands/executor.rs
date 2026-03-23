/// Applies parsed AI commands to GameState
use crate::engine::{
    game_state::{GameState, EntrySource, PlayerChoice, ChoiceStyle},
    inventory::{Item, ItemCategory},
    quest::{Quest, QuestObjective, QuestStatus},
    combat::Combatant,
    memory::MemoryCategory,
    map::{Location, LocationType},
};
use crate::ai::parser::ParsedCommand;

pub fn apply_command(state: &mut GameState, cmd: &ParsedCommand) -> Option<String> {
    let args = &cmd.args;
    match cmd.verb.as_str() {
        // ── Inventory ──────────────────────────────────────────────────
        "add_item" => {
            let id = args.get(0)?.clone();
            let name = args.get(1).cloned().unwrap_or_else(|| id.clone()).replace('_', " ");
            let cat_str = args.get(2).cloned().unwrap_or_default();

            if name.trim().len() < 2 {
                return Some(format!("Rejected item '{}': name too short", name));
            }

            // Category must be explicitly recognized — no silent fallback to Misc.
            // If the AI can't assign a meaningful category, it's not a real item.
            let category = match cat_str.to_lowercase().as_str() {
                "weapon"     => ItemCategory::Weapon,
                "armor"      => ItemCategory::Armor,
                "consumable" => ItemCategory::Consumable,
                "quest"      => ItemCategory::Quest,
                "tool"       => ItemCategory::Tool,
                "key"        => ItemCategory::Key,
                "misc"       => ItemCategory::Misc,
                _ => return Some(format!(
                    "Rejected item '{}': category '{}' is missing or unrecognized (must be Weapon/Armor/Consumable/Quest/Misc/Tool/Key)",
                    name, cat_str
                )),
            };

            let item = Item::new(&id, &name, category);
            match state.inventory.add_item(item) {
                Ok(_) => Some(format!("Added {} to inventory", name)),
                Err(e) => Some(format!("Inventory: {}", e)),
            }
        }

        "remove_item" => {
            let id = args.get(0)?;
            if state.inventory.remove_item_by_id(id) {
                Some(format!("Removed {} from inventory", id))
            } else {
                Some(format!("Item {} not found", id))
            }
        }

        // ── Character HP/MP/Stats ──────────────────────────────────────
        "heal" => {
            let amount: i32 = args.get(0)?.parse().ok()?;
            state.player.heal(amount);
            Some(format!("Healed {} HP (now {}/{})", amount, state.player.hp, state.player.hp_max))
        }

        "damage" => {
            let amount: i32 = args.get(0)?.parse().ok()?;
            state.player.damage(amount);
            if !state.player.is_alive() {
                state.add_story_entry(EntrySource::System, "You have fallen...");
            }
            Some(format!("Took {} damage (now {}/{})", amount, state.player.hp, state.player.hp_max))
        }

        "set_hp" => {
            let val: i32 = args.get(0)?.parse().ok()?;
            state.player.hp = val.clamp(0, state.player.hp_max);
            Some(format!("HP set to {}", state.player.hp))
        }

        "set_mp" => {
            let val: i32 = args.get(0)?.parse().ok()?;
            state.player.mp = val.clamp(0, state.player.mp_max);
            Some(format!("MP set to {}", state.player.mp))
        }

        // ── Currency ───────────────────────────────────────────────────
        // add_currency <denomination> <amount>
        // denomination: copper/cp, silver/sp, gold/gp, platinum/pp
        "add_currency" => {
            let denom = args.get(0)?;
            let amount: u32 = args.get(1)?.parse().ok()?;
            match state.player.purse.add(denom, amount) {
                Ok(()) => Some(format!("Gained {}{}  (purse: {})", amount, denom, state.player.purse.to_context_string())),
                Err(e) => Some(format!("Currency error: {}", e)),
            }
        }

        "remove_currency" => {
            let denom = args.get(0)?;
            let amount: u32 = args.get(1)?.parse().ok()?;
            match state.player.purse.remove(denom, amount) {
                Ok(()) => Some(format!("Lost {}{}  (purse: {})", amount, denom, state.player.purse.to_context_string())),
                Err(e) => Some(format!("Currency: {}", e)),
            }
        }

        // Legacy aliases kept for backwards compatibility with old saves/prompts
        "add_gold" => {
            let amount: u32 = args.get(0)?.parse::<f32>().ok()? as u32;
            let _ = state.player.purse.add("gold", amount);
            Some(format!("Gained {}gp  (purse: {})", amount, state.player.purse.to_context_string()))
        }

        "remove_gold" => {
            let amount: u32 = args.get(0)?.parse::<f32>().ok()? as u32;
            match state.player.purse.remove("gold", amount) {
                Ok(()) => Some(format!("Lost {}gp  (purse: {})", amount, state.player.purse.to_context_string())),
                Err(e) => Some(format!("Currency: {}", e)),
            }
        }

        "set_stat" => {
            let name = args.get(0)?.clone();
            let val: i32 = args.get(1)?.parse().ok()?;
            state.player.custom_stats.insert(name.clone(), val);
            Some(format!("Stat {} = {}", name, val))
        }

        "add_xp" => {
            let amount: u32 = args.get(0)?.parse().ok()?;
            let leveled = state.player.add_experience(amount);
            if leveled {
                state.add_story_entry(EntrySource::System,
                    &format!("Level up! You are now level {}!", state.player.level));
            }
            Some(format!("Gained {} XP", amount))
        }

        // ── Quests ─────────────────────────────────────────────────────
        "add_quest" => {
            let id = args.get(0)?.clone();
            let title = args[1..].join(" ").replace('_', " ");
            let mut q = Quest::new(&id, &title);
            q.status = QuestStatus::Active;
            state.quests.add_quest(q);
            Some(format!("Quest added: {}", title))
        }

        "complete_quest" => {
            let id = args.get(0)?;
            state.quests.set_status(id, QuestStatus::Completed);
            Some(format!("Quest completed: {}", id))
        }

        "fail_quest" => {
            let id = args.get(0)?;
            state.quests.set_status(id, QuestStatus::Failed);
            Some(format!("Quest failed: {}", id))
        }

        "complete_obj" => {
            let quest_id = args.get(0)?;
            let obj_id = args.get(1)?;
            state.quests.complete_objective(quest_id, obj_id);
            Some(format!("Objective complete: {}/{}", quest_id, obj_id))
        }

        "add_obj" => {
            let quest_id = args.get(0)?.clone();
            let obj_id = args.get(1)?.clone();
            let desc = args[2..].join(" ");
            if let Some(quest) = state.quests.get_mut(&quest_id) {
                quest.objectives.push(QuestObjective {
                    id: obj_id,
                    description: desc.clone(),
                    completed: false,
                    optional: false,
                });
                Some(format!("Objective added: {}", desc))
            } else {
                Some(format!("Quest {} not found", quest_id))
            }
        }

        // ── NPC Relations ──────────────────────────────────────────────
        "npc_relation" => {
            let id = args.get(0)?.clone();
            let name = args.get(1)?.replace('_', " ");
            let delta: i32 = args.get(2)?.parse().ok()?;
            let reason = args.get(3).map(|r| r.replace('_', " "));
            state.relations.adjust(&id, &name, delta, reason.as_deref());
            Some(format!("Relation {} {}{}", name, if delta >= 0 { "+" } else { "" }, delta))
        }

        // ── Memory / Time ──────────────────────────────────────────────
        "record_event" => {
            let importance: u8 = args.get(0)?.parse().ok()?;
            let cat_str = args.get(1).cloned().unwrap_or_default();
            let desc = args[2..].join(" ");
            let category = match cat_str.to_lowercase().as_str() {
                "plot"      => MemoryCategory::Plot,
                "npc"       => MemoryCategory::Npc,
                "combat"    => MemoryCategory::Combat,
                "discovery" => MemoryCategory::Discovery,
                "choice"    => MemoryCategory::Choice,
                _           => MemoryCategory::World,
            };
            state.memory.record(category, &desc, importance, Vec::new());
            Some(format!("Recorded: {}", desc))
        }

        "advance_time" => {
            let minutes: u32 = args.get(0)?.parse().ok()?;
            state.memory.advance_time(minutes);
            Some(format!("Time advanced by {} minutes", minutes))
        }

        // ── Scene / World ──────────────────────────────────────────────
        "set_scene" => {
            state.scene = args.join(" ").replace('_', " ");
            Some(format!("Scene: {}", state.scene))
        }

        "set_weather" => {
            state.weather = args.join(" ");
            Some(format!("Weather: {}", state.weather))
        }

        "move_to" => {
            let loc_id = args.get(0)?.clone();
            // Auto-create location if it doesn't exist
            if state.world.locations.get(&loc_id).is_none() {
                let name = loc_id.replace('_', " ");
                state.world.add_location(Location::new(&loc_id, &name, LocationType::Wilderness));
                // Connect from current location if any
                let prev = state.world.current_location.clone();
                if !prev.is_empty() {
                    if let Some(prev_loc) = state.world.locations.get_mut(&prev) {
                        if !prev_loc.connections.contains(&loc_id) {
                            prev_loc.connections.push(loc_id.clone());
                        }
                    }
                    if let Some(new_loc) = state.world.locations.get_mut(&loc_id) {
                        if !new_loc.connections.contains(&prev) {
                            new_loc.connections.push(prev.clone());
                        }
                    }
                }
            }
            match state.world.move_to(&loc_id) {
                Ok(_) => {
                    state.scene = state.world.current()
                        .map(|l| l.name.clone())
                        .unwrap_or_default();
                    Some(format!("Moved to {}", state.scene))
                }
                Err(e) => Some(format!("Move failed: {}", e)),
            }
        }

        "add_location" => {
            let id = args.get(0)?.clone();
            let name = args.get(1).cloned().unwrap_or_else(|| id.clone()).replace('_', " ");
            let loc_type = match args.get(2).map(|s| s.to_lowercase()).as_deref() {
                Some("town")      => LocationType::Town,
                Some("dungeon")   => LocationType::Dungeon,
                Some("cave")      => LocationType::Cave,
                Some("ruins")     => LocationType::Ruins,
                Some("road")      => LocationType::Road,
                Some("building")  => LocationType::Building,
                _                 => LocationType::Wilderness,
            };
            state.world.add_location(Location::new(&id, &name, loc_type));
            Some(format!("Location added: {}", name))
        }

        "set_flag" => {
            let key = args.get(0)?.clone();
            let val_str = args.get(1).cloned().unwrap_or_else(|| "true".to_string());
            let val = if let Ok(b) = val_str.parse::<bool>() {
                serde_json::Value::Bool(b)
            } else if let Ok(n) = val_str.parse::<i64>() {
                serde_json::Value::Number(n.into())
            } else {
                serde_json::Value::String(val_str)
            };
            state.set_flag(&key, val);
            Some(format!("Flag {} set", key))
        }

        // ── Choices ────────────────────────────────────────────────────
        "set_choice" => {
            let id = args.get(0)?.clone();
            let style_str = args.get(1).cloned().unwrap_or_default();
            let raw_text = args[2..].join(" ");

            // Guard: AI sometimes repeats the id at the start of the text field.
            // Strip it if the text begins with the id (case-insensitive, with or without underscores).
            let id_variants = [
                id.clone(),
                id.replace('_', " "),
                id.replace('_', ""),
            ];
            let text = {
                let mut t = raw_text.clone();
                for variant in &id_variants {
                    let prefix = variant.to_lowercase();
                    if t.to_lowercase().starts_with(&prefix) {
                        t = t[variant.len()..].trim().to_string();
                        break;
                    }
                }
                t
            };
            let style = match style_str.to_lowercase().as_str() {
                "danger"  => ChoiceStyle::Danger,
                "social"  => ChoiceStyle::Social,
                "special" => ChoiceStyle::Special,
                _         => ChoiceStyle::Normal,
            };
            state.choices.push(PlayerChoice { id, text, style, tags: Vec::new() });
            None // silent
        }

        "clear_choices" => {
            state.clear_choices();
            None
        }

        // ── Combat ─────────────────────────────────────────────────────
        "start_combat" => {
            let enemy_id = args.get(0)?.clone();
            let name = args.get(1).cloned().unwrap_or_else(|| "Enemy".to_string()).replace('_', " ");
            let hp: i32 = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(20);
            let atk: i32 = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(3);
            let def: i32 = args.get(4).and_then(|v| v.parse().ok()).unwrap_or(10);

            let player_combatant = crate::engine::combat::Combatant::from_character(&state.player);
            let enemy = Combatant::new_enemy(&enemy_id, &name, hp, atk, def);
            state.combat.start(vec![player_combatant, enemy]);
            Some(format!("Combat started vs {}", name))
        }

        "end_combat" => {
            state.combat = crate::engine::combat::Combat::new();
            Some("Combat ended".to_string())
        }

        _ => Some(format!("Unknown command: {}", cmd.verb)),
    }
}
