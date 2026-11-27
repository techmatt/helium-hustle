// Card art: one painted cut-out per building (`src/assets/buildings/<id>.webp`, made by
// experiments/results/020_building_card_art/) and per project (`src/assets/projects/<id>.webp`,
// experiments/results/100_wire_project_art_and_research_glyphs/), each 192 px square, 2x the
// card's 96 px box, transparent around the subject. Originals are in the shared art/ folder.
// Presentation only: a card with no file simply shows no picture.

import { byName } from './assets';

const stems = (files: Record<string, string>) => byName(files, '.webp');

const BUILDINGS = stems(import.meta.glob<string>('../assets/buildings/*.webp', { eager: true, query: '?url', import: 'default' }));
const PROJECTS = stems(import.meta.glob<string>('../assets/projects/*.webp', { eager: true, query: '?url', import: 'default' }));

/** The URL of a building's card picture, if it has one. */
export function buildingArt(id: string): string | undefined {
  return BUILDINGS[id];
}

/** The URL of a project's card picture, if it has one. */
export function projectArt(id: string): string | undefined {
  return PROJECTS[id];
}

// Ideology emblems: one painted panel per axis (`src/assets/ideology/<axis>.webp`, 320x400, 2x
// the Ideologies card's 160 px slot; made by experiments/results/119_no_partial_purchases/emblems.py).
const EMBLEMS = stems(import.meta.glob<string>('../assets/ideology/*.webp', { eager: true, query: '?url', import: 'default' }));

/** The URL of an ideology axis's emblem, if it has one. */
export function emblemArt(axis: string): string | undefined {
  return EMBLEMS[axis];
}
