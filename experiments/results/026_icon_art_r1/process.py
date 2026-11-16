"""Cuts 026's horizon progress draws into strips, into the shared art/small_icons/horizon/.

Each stage (art/small_icons/raw/progress-<stage>-earthlight.png) is cut with 025's strip
rule (../025_nav_horizon_art_r1/process.py: 2.5:1, 690 x 276) at the band of 025's bare
`earthlight` draw, so the base stays put across stages; the bare strip is 025's own,
copied. Alpha under FAINT is zeroed: a draw can come back with a faint painted haze in
its "transparent" sky. Run from the repo root:
python experiments/results/026_icon_art_r1/process.py
"""

import importlib.util
import pathlib
import shutil
import sys

from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
RAW = REPO / "art" / "small_icons" / "raw"
OUT = REPO / "art" / "small_icons" / "horizon"
BARE = REPO / "art" / "nav_horizon" / "raw" / "horizon-earthlight.png"
STAGES = ["pad", "liftoff", "climb", "hop"]
FAINT = 40

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("p025", HERE.parent / "025_nav_horizon_art_r1" / "process.py")
p025 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(p025)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(REPO / "art" / "nav_horizon" / "horizon" / "horizon-earthlight.png", OUT / "bare.png")
    top = p025.band_top(Image.open(BARE).convert("RGBA"), "horizon-earthlight")
    for s in STAGES:
        src = RAW / f"progress-{s}-earthlight.png"
        if not src.exists():
            raise SystemExit(f"process: {src} missing: run gen.mjs {s}")
        im = Image.open(src).convert("RGBA")
        a = im.getchannel("A").point(lambda v: 0 if v < FAINT else v)
        im.putalpha(a)
        p025.strip(im, top).save(OUT / f"{s}.png", optimize=True)
    print("process: done")


if __name__ == "__main__":
    main()
