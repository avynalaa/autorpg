import { writable } from 'svelte/store';

export type Theme = 'slate' | 'codex' | 'wire';

const STORAGE_KEY = 'autorpg-theme';

const VALID: Theme[] = ['slate', 'codex', 'wire'];

function getInitial(): Theme {
  if (typeof localStorage === 'undefined') return 'slate';
  const stored = localStorage.getItem(STORAGE_KEY) as Theme;
  return VALID.includes(stored) ? stored : 'slate';
}

export const theme = writable<Theme>(getInitial());

export function setTheme(t: Theme) {
  theme.set(t);
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, t);
  }
}

export const THEMES: { id: Theme; label: string; description: string }[] = [
  { id: 'slate', label: 'Slate',  description: 'Modern dark — clean and versatile' },
  { id: 'codex', label: 'Codex',  description: 'Literary dark — serif, warm, scholarly' },
  { id: 'wire',  label: 'Wire',   description: 'Terminal — monospace, phosphor green' },
];
