// Entry: load the engine, load the save (or start a game), start the clock, render.

import '@fontsource/rajdhani/600.css';
import '@fontsource/rajdhani/700.css';
import '@fontsource/ibm-plex-sans/400.css';
import '@fontsource/ibm-plex-sans/400-italic.css';
import '@fontsource/ibm-plex-sans/600.css';
import '@fontsource/ibm-plex-mono/400.css';
import '@fontsource/ibm-plex-mono/600.css';
import { render } from 'preact';
import { loadEngine } from './engine';
import { start } from './store';
import { App } from './ui/App';

async function main(): Promise<void> {
  await loadEngine();
  start();
  // The boot line goes: left under the app, a page scrolled into view (a focused control
  // near the bottom) showed "Loading…" below the status bar and cut the title.
  document.getElementById('boot')?.remove();
  render(<App />, document.getElementById('app')!);
}

main().catch((e) => {
  const p = document.getElementById('boot') ?? document.body.appendChild(document.createElement('p'));
  p.textContent = `Engine failed to load: ${e}`;
});
