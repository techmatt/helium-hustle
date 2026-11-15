// The color theme: Earthrise light (the default for now) or dark, remembered per browser.
// Themes differ only in the color tokens style.css sets under `:root[data-theme=...]`;
// index.html sets the attribute before the first paint, so the page never flashes.

export type Theme = 'light' | 'dark';

const KEY = 'hh.theme';
const listeners = new Set<() => void>();

export function theme(): Theme {
  return document.documentElement.dataset.theme === 'dark' ? 'dark' : 'light';
}

export function setTheme(t: Theme): void {
  document.documentElement.dataset.theme = t;
  try {
    localStorage.setItem(KEY, t);
  } catch {}
  listeners.forEach((f) => f());
}

/** Calls `f` on each theme change; returns the unsubscribe. */
export function onTheme(f: () => void): () => void {
  listeners.add(f);
  return () => listeners.delete(f);
}
