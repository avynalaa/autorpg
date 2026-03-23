import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { GameState, AiResponse, DiceRoll, Quest, SaveMetadata, Settings } from '../types';

// ── Core state store ──────────────────────────────────────────────────────
export const gameState = writable<GameState | null>(null);
export const isLoading = writable(false);
export const lastError = writable<string | null>(null);
export const lastDiceRoll = writable<DiceRoll | null>(null);

// ── Derived helpers ───────────────────────────────────────────────────────
export const player = derived(gameState, $gs => $gs?.player ?? null);
export const inventory = derived(gameState, $gs => $gs?.inventory ?? null);
export const combat = derived(gameState, $gs => $gs?.combat ?? null);
export const inCombat = derived(gameState, $gs => $gs?.combat?.state !== 'OutOfCombat');
export const quests = derived(gameState, $gs => $gs?.quests?.quests ?? []);
export const activeQuests = derived(quests, $q => $q.filter(q => q.status === 'Active'));
export const plannedQuests = derived(quests, $q => $q.filter(q => q.status === 'Active' || q.status === 'OnHold' || q.status === 'NotStarted'));
export const completedQuestCount = derived(quests, $q => $q.filter(q => q.status === 'Completed').length);
export const failedQuestCount = derived(quests, $q => $q.filter(q => q.status === 'Failed').length);
export const totalObjectiveCount = derived(quests, ($q) =>
  $q.reduce((total: number, quest: Quest) => total + quest.objectives.length, 0)
);
export const completedObjectiveCount = derived(quests, ($q) =>
  $q.reduce((total: number, quest: Quest) => total + quest.objectives.filter((objective) => objective.completed).length, 0)
);
export const storyLog = derived(gameState, $gs => $gs?.story_log ?? []);
export const choices = derived(gameState, $gs => $gs?.choices ?? []);
export const relations = derived(gameState, $gs =>
  Object.values($gs?.relations?.relations ?? {})
);
export const currentLocation = derived(gameState, $gs => {
  const world = $gs?.world;
  if (!world) return null;
  return world.locations[world.current_location] ?? null;
});

// ── Actions ───────────────────────────────────────────────────────────────

export async function loadGameState() {
  const gs = await invoke<GameState>('get_game_state');
  gameState.set(gs);
}

export async function sendToAI(input: string): Promise<AiResponse> {
  isLoading.set(true);
  lastError.set(null);
  try {
    const result = await invoke<AiResponse>('send_to_ai', { playerInput: input });
    gameState.set(result.game_state);
    return result;
  } catch (e) {
    const msg = String(e);
    lastError.set(msg);
    throw e;
  } finally {
    isLoading.set(false);
  }
}

export async function rollDice(notation: string, label?: string): Promise<DiceRoll> {
  const roll = label
    ? await invoke<DiceRoll>('roll_dice_labeled', { notation, label })
    : await invoke<DiceRoll>('roll_dice', { notation });
  lastDiceRoll.set(roll);
  return roll;
}

export async function createCharacter(data: {
  name: string;
  race: string;
  class: string;
  backstory: string;
}) {
  const gs = await invoke<GameState>('create_character', { data });
  gameState.set(gs);
}

export async function resetGame() {
  await invoke('reset_game');
  await loadGameState();
}

export async function setApiKey(key: string) {
  await invoke('set_api_key', { key });
}

export async function setModel(model: string) {
  await invoke('set_model', { model });
}

export async function setEndpoint(url: string) {
  await invoke('set_endpoint', { url });
}

export async function setApiFormat(format: string) {
  await invoke('set_api_format', { format });
}

export async function getSettings(): Promise<Settings> {
  return invoke('get_settings');
}

export async function fetchModels(): Promise<string[]> {
  return invoke('fetch_models');
}

export async function getSystemPrompt(): Promise<string> {
  return invoke<string>('get_system_prompt');
}

export async function setSystemPrompt(prompt: string) {
  await invoke('set_system_prompt', { prompt });
}

export async function resetSystemPrompt() {
  await invoke('reset_system_prompt');
}

export async function equipItem(slotIdx: number) {
  const gs = await invoke<GameState>('equip_item', { slotIdx });
  gameState.set(gs);
}

export async function removeInventoryItem(slotIdx: number) {
  const gs = await invoke<GameState>('remove_inventory_item', { slotIdx });
  gameState.set(gs);
}

export async function playerAttack(targetId: string) {
  const gs = await invoke<GameState>('player_attack', { targetId });
  gameState.set(gs);
}

export async function forceStartCombat(enemyName: string, hp: number, atk: number, def: number) {
  const gs = await invoke<GameState>('force_start_combat', { enemyName, hp, atk, def });
  gameState.set(gs);
}

export async function forceNpcCombat(npcId: string) {
  const gs = await invoke<GameState>('force_npc_combat', { npcId });
  gameState.set(gs);
}

export async function setHistoryMode(mode: string) {
  await invoke('set_history_mode', { mode });
}

export async function setHistoryLimit(limit: number) {
  await invoke('set_history_limit', { limit });
}

export async function setMaxTokens(tokens: number) {
  await invoke('set_max_tokens', { tokens });
}

export async function regenerateLast(): Promise<AiResponse> {
  isLoading.set(true);
  lastError.set(null);
  try {
    const result = await invoke<AiResponse>('regenerate_last');
    gameState.set(result.game_state);
    return result;
  } catch (e) {
    const msg = String(e);
    lastError.set(msg);
    throw e;
  } finally {
    isLoading.set(false);
  }
}

export async function goToMenu(): Promise<void> {
  const gs = await invoke<GameState>('go_to_menu');
  gameState.set(gs);
}

export async function listSaves(): Promise<Array<SaveMetadata | null>> {
  return invoke('list_saves_cmd');
}

export async function saveGame(slot: number, name: string): Promise<void> {
  await invoke('save_game_cmd', { slot, name });
}

export async function loadGame(slot: number): Promise<GameState> {
  const result = await invoke<{ game_state: GameState }>('load_game_cmd', { slot });
  gameState.set(result.game_state);
  return result.game_state;
}

export async function deleteSave(slot: number): Promise<void> {
  await invoke('delete_save_cmd', { slot });
}
