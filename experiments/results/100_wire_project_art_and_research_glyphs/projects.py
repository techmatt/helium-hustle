"""100: ships Matt's project card art picks (picker project_art_r1) into the game.

For each pick, 097's card picture art/projects_r1/<project>-<option>_192.png (already cut as
020 ships building art: a 192 px square, 2x the card's 96 px box) is saved as
web/src/assets/projects/<project>.webp, WebP quality 88 as 020's building pictures.

Run from the repo root (Python with Pillow and WebP). Every pick and source is checked
before anything is written.
"""

import json
import pathlib

from PIL import Image

REPO = pathlib.Path(__file__).resolve().parents[3]
SRC = REPO / "art" / "projects_r1"
WEB = REPO / "web" / "src" / "assets" / "projects"
PROJECTS = REPO / "content" / "projects.json"

PICKS = {
    "blueprint_archive": "vault",
    "foundation_grant": "kit",
    "lunar_cartography": "survey",
    "microwave_power": "satellite",
    "ai_consciousness": "core",
    "research_archive": "hall",
    "chemical_energy": "tankfarm",
    "deep_core_survey": "cutaway",
    "grid_recalibration": "arc",
    "predictive_maintenance": "spares",
    "market_cornering": "analyzer",
    "speculator_dossier": "board",
    "conditional_logic": "flowchart",
    "deep_space_telescope": "observatory",
    "dyson_sphere": "shell",
}


def main():
    ids = {p["id"] for p in json.loads(PROJECTS.read_text(encoding="utf-8"))}
    plan = []
    for pid, key in PICKS.items():
        if pid not in ids:
            raise SystemExit(f"projects: no project {pid} in content/projects.json")
        src = SRC / f"{pid}-{key}_192.png"
        if not src.exists():
            raise SystemExit(f"projects: {src} missing")
        im = Image.open(src).convert("RGBA")
        if im.size != (192, 192):
            raise SystemExit(f"projects: {src} is {im.size}, not 192x192")
        plan.append((pid, im))
    missing = sorted(ids - PICKS.keys())
    if missing:
        print(f"projects: no pick for {', '.join(missing)} (their cards show no picture)")
    WEB.mkdir(parents=True, exist_ok=True)
    total = 0
    for pid, im in plan:
        out = WEB / f"{pid}.webp"
        im.save(out, "WEBP", quality=88, method=6)
        total += out.stat().st_size
    print(f"projects: {len(plan)} pictures, {total // 1024} KB")


if __name__ == "__main__":
    main()
