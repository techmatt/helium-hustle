"""027: copies the picked painted art into web/src/assets/ as WebP (run from the repo root).

- nav/<id>.webp: 025's painterly nav tile (`paint`), 96 px (2x the 34 px tile icon,
  with room for a 48 px one).
- horizon/<stage>.webp: 026's earthlight strips bare, pad and climb, full size (690 x 276).
Reads the shared art/ folder, never writes it.
"""
from pathlib import Path

from PIL import Image

ART = Path('art')
OUT = Path('web/src/assets')

NAV = ['buildings', 'commands', 'launch_pads', 'research', 'projects', 'ideologies',
       'adversaries', 'stats', 'story', 'retirement', 'options']
STAGES = ['bare', 'pad', 'climb']

(OUT / 'nav').mkdir(parents=True, exist_ok=True)
for nav_id in NAV:
    im = Image.open(ART / 'nav_horizon' / 'icons' / f'{nav_id}_paint.png').convert('RGBA')
    im.resize((96, 96), Image.LANCZOS).save(OUT / 'nav' / f'{nav_id}.webp', quality=90, method=6)

(OUT / 'horizon').mkdir(parents=True, exist_ok=True)
for stage in STAGES:
    im = Image.open(ART / 'small_icons' / 'horizon' / f'{stage}.png').convert('RGBA')
    im.save(OUT / 'horizon' / f'{stage}.webp', quality=88, method=6)

print(f'wrote {len(NAV)} nav tiles, {len(STAGES)} horizon strips')
