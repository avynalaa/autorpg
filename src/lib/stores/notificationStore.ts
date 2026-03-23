import { writable } from 'svelte/store';

export type NotifType = 'quest_new' | 'quest_done' | 'quest_fail' | 'objective' | 'item' | 'relation';

export interface Notification {
  id: string;
  type: NotifType;
  icon: string;
  label: string;   // small tag above the title (e.g. "Quest" / "Item")
  title: string;
  body?: string;
}

export const notifications = writable<Notification[]>([]);

const DURATION = 3800;

export function pushNotification(n: Omit<Notification, 'id'>) {
  const id = crypto.randomUUID();
  notifications.update(ns => [{ ...n, id }, ...ns]);
  setTimeout(() => dismiss(id), DURATION);
}

export function dismiss(id: string) {
  notifications.update(ns => ns.filter(n => n.id !== id));
}
