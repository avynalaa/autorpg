export interface Purse {
  platinum: number;
  gold: number;
  silver: number;
  copper: number;
}

export interface AbilityScores {
  strength: number;
  dexterity: number;
  constitution: number;
  intelligence: number;
  wisdom: number;
  charisma: number;
}

export interface Character {
  id: string;
  name: string;
  race: string;
  class: string;
  level: number;
  experience: number;
  experience_to_next: number;
  hp: number;
  hp_max: number;
  mp: number;
  mp_max: number;
  stamina: number;
  stamina_max: number;
  ability_scores: AbilityScores;
  skills: Skill[];
  status_effects: StatusEffect[];
  purse: Purse;
  portrait: string | null;
  backstory: string;
  notes: string;
  custom_stats: Record<string, number>;
}

export interface Skill {
  name: string;
  rank: number;
  ability: string;
}

export interface StatusEffect {
  id: string;
  name: string;
  duration: number | null;
  description: string;
}

export interface Item {
  id: string;
  name: string;
  category: string;
  description: string;
  weight: number;
  value: number;
  quantity: number;
  stackable: boolean;
  equip_slot: string | null;
  stat_mods: Record<string, number>;
  tags: string[];
}

export interface InventorySlot {
  index: number;
  item: Item | null;
  locked: boolean;
  reserved: boolean;
}

export interface Inventory {
  slots: InventorySlot[];
  capacity: number;
  max_weight: number;
  equipped: Record<string, Item>;
}

export interface Quest {
  id: string;
  title: string;
  description: string;
  status: 'NotStarted' | 'Active' | 'Completed' | 'Failed' | 'OnHold';
  objectives: QuestObjective[];
  rewards: string[];
  giver: string | null;
  location: string | null;
  notes: string[];
}

export interface QuestObjective {
  id: string;
  description: string;
  completed: boolean;
  optional: boolean;
}

export interface NpcRelation {
  npc_id: string;
  npc_name: string;
  score: number;
  tier: string;
  notes: string[];
  last_met: string | null;
}

export interface StoryEntry {
  id: number;
  source: 'Narrator' | 'Player' | 'System' | 'Internal';
  text: string;
  timestamp: string;
}

export interface PlayerChoice {
  id: string;
  text: string;
  style: 'Normal' | 'Danger' | 'Social' | { Skill: string } | 'Special';
  tags: string[];
}

export interface Combatant {
  id: string;
  name: string;
  hp: number;
  hp_max: number;
  attack: number;
  defense: number;
  initiative: number;
  is_player: boolean;
  is_alive: boolean;
  tags: string[];
}

export interface Combat {
  state: 'OutOfCombat' | 'Initiative' | 'PlayerTurn' | 'EnemyTurn' | 'Victory' | 'Defeat' | 'Fled';
  combatants: Combatant[];
  turn_order: string[];
  current_turn_idx: number;
  round: number;
  log: { entries: string[] };
}

export interface Location {
  id: string;
  name: string;
  description: string;
  location_type: string;
  visited: boolean;
  connections: string[];
  npcs: string[];
  tags: string[];
}

export interface WorldMap {
  locations: Record<string, Location>;
  current_location: string;
  previous_location: string | null;
}

export interface GameTime {
  day: number;
  hour: number;
  minute: number;
  season: 'Spring' | 'Summer' | 'Autumn' | 'Winter';
  year: number;
}

export interface GameState {
  phase: 'MainMenu' | 'CharacterCreation' | 'Playing' | 'Combat' | 'GameOver';
  player: Character;
  inventory: Inventory;
  combat: Combat;
  quests: { quests: Quest[] };
  relations: { relations: Record<string, NpcRelation> };
  memory: { current_time: GameTime; events: unknown[] };
  world: WorldMap;
  scene: string;
  weather: string;
  story_log: StoryEntry[];
  choices: PlayerChoice[];
  flags: Record<string, unknown>;
}

export interface DiceRoll {
  dice: string;
  rolls: number[];
  modifier: number;
  total: number;
  label: string | null;
}

export interface AiResponse {
  narrative: string;
  commands_applied: string[];
  game_state: GameState;
}

export interface SaveMetadata {
  slot: number;
  name: string;
  saved_at: string;
  player_name: string;
  scene: string;
}

export interface Settings {
  endpoint: string;
  api_format: string;
  model: string;
  history_mode: string;
  history_limit: number;
  max_tokens: number;
}
