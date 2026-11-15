// Entry: load the engine, load the save (or start a game), start the clock, render.

import { render } from 'preact';
import { loadEngine } from './engine';
import { start } from './store';
import { App } from './ui/App';

async function main(): Promise<void> {
  await loadEngine();
  start();
  render(<App />, document.getElementById('app')!);
}

main().catch((e) => {
  const p = document.getElementById('boot')!;
  p.textContent = `Engine failed to load: ${e}`;
});
