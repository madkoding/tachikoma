// Simple localStorage-backed UI mode. `ponytail:` no zustand in admin; a module
// with a subscribe hook is enough for a single persisted boolean.

export type UIMode = 'simple' | 'advanced';

const KEY = 'tachikoma-admin-mode';
const DEFAULT: UIMode = 'simple';

export function getMode(): UIMode {
  return (localStorage.getItem(KEY) as UIMode) || DEFAULT;
}

export function setMode(mode: UIMode) {
  localStorage.setItem(KEY, mode);
}

type Listener = (mode: UIMode) => void;
const listeners = new Set<Listener>();

export function subscribeMode(listener: Listener) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function changeMode(mode: UIMode) {
  setMode(mode);
  listeners.forEach((l) => l(mode));
}
