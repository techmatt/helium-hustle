# 017: icon style round 1

`make_icons.py` draws seven subjects (nav tiles Buildings, Commands, Research; buildings
Solar Panel, Regolith Excavator, Smelter, Data Center) as SVG in four styles, plus the picker
`icon_style_r1`. Output goes to the shared, uncommitted `art/` folder:

- `art/pickers/icon_style_r1/manifest.json`: the picker, with sheets `style_<a-d>.svg` and
  per-subject comparisons in `subjects/`.
- `art/icons/r1/<style>/<subject>_<theme>.svg`: each icon alone, 64 px, per theme.

Run from the repo root with `python experiments/results/017_icon_style_picker/make_icons.py`.
`render_check.sh` rasterizes the sheets with headless Edge into `out/017_icon_style_picker/`.

Each subject is one geometry on a 32-unit grid with its parts tagged by role (body, detail,
ground, accent, aline). A style is a way of rendering those roles, so the candidates differ
in style only:

| Style | Line | Fill | Resource color |
|---|---|---|---|
| A. Hairline | 1.5 px ink | none | the resource part as a line, in a per-theme shade |
| B. Duotone | 2 px ink | ink at 10 % | resource part filled, ink outline |
| C. Solid | none | ink silhouette, details cut out | resource part solid |
| D. Tile | 1.3 px on a rounded lunar-gray tile | tile-light body | resource part filled |

Stroke widths are set in display pixels, so they stay the same at every icon size. Pale
resource colors (Energy, Titanium) get a darker line shade on light. Nav icons use the
theme amber as their accent.
