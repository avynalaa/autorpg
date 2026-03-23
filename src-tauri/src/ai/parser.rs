/// Parses AI response text and extracts [CMD:...] directives.
///
/// Command format: [CMD:verb arg1 arg2 ...]
///
/// Supported commands:
///   add_item id name category
///   remove_item id
///   set_hp value
///   damage amount
///   heal amount
///   add_currency denomination amount
///   remove_currency denomination amount
///   add_gold amount       (legacy alias → gold denomination)
///   remove_gold amount    (legacy alias → gold denomination)
///   set_stat name value
///   add_xp amount
///   add_quest id title...
///   complete_quest id
///   fail_quest id
///   complete_obj quest_id obj_id
///   npc_relation id name delta reason...
///   record_event importance category description...
///   advance_time minutes
///   set_scene name...
///   set_weather description...
///   move_to location_id
///   add_location id name type
///   set_flag key value
///   set_choice id style text...
///   clear_choices
///   start_combat enemy_id name hp atk def
///   end_combat

use regex::Regex;

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub verb: String,
    pub args: Vec<String>,
}

pub struct ParseResult {
    pub clean_text: String,
    pub commands: Vec<ParsedCommand>,
}

pub fn parse_ai_response(raw: &str) -> ParseResult {
    let re = Regex::new(r"\[CMD:([^\]]+)\]").unwrap();
    let mut commands = Vec::new();

    for cap in re.captures_iter(raw) {
        let inner = cap[1].trim().to_string();
        let parts: Vec<String> = inner.splitn(6, ' ')
            .map(|s| s.to_string())
            .collect();
        if parts.is_empty() { continue; }
        commands.push(ParsedCommand {
            verb: parts[0].to_lowercase(),
            args: parts[1..].to_vec(),
        });
    }

    // Strip complete CMD tags, then also remove any truncated/incomplete ones
    // (e.g. "[CMD:set_choice 1" without a closing "]" caused by max_tokens cutoff)
    let incomplete_re = Regex::new(r"\[CMD:[^\]]*$").unwrap();
    let clean = re.replace_all(raw, "").to_string();
    let clean = incomplete_re.replace_all(&clean, "").to_string();
    let clean = clean.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    ParseResult { clean_text: clean, commands }
}
