// Building card art: one painted cut-out per building, `src/assets/buildings/<id>.webp`
// (192 px square, 2x the card's 96 px box, transparent around the building). Made by
// experiments/results/020_building_card_art/ from originals in the shared art/ folder.
// Presentation only: a building with no file simply shows no picture.

const files = import.meta.glob<string>('../assets/buildings/*.webp', { eager: true, query: '?url', import: 'default' });

const byId: Record<string, string> = {};
for (const [file, url] of Object.entries(files)) byId[file.slice(file.lastIndexOf('/') + 1, -'.webp'.length)] = url;

/** The URL of a building's card picture, if it has one. */
export function buildingArt(id: string): string | undefined {
  return byId[id];
}
