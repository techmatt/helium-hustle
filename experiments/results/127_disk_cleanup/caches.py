"""Size the shared tool caches (every project's, not this one's alone), never following links."""
import os, sys, time
sys.path.insert(0, os.path.dirname(__file__))

def walk(p):
    t = 0
    try:
        it = os.scandir(p)
    except OSError:
        return 0
    with it:
        for e in it:
            try:
                if e.is_symlink() or e.is_junction():
                    continue
                t += walk(e.path) if e.is_dir(follow_symlinks=False) else e.stat(follow_symlinks=False).st_size
            except OSError:
                pass
    return t

H = os.path.expanduser("~")
L = os.environ.get("LOCALAPPDATA", "")
for p in [os.path.join(H, ".cargo", "registry"), os.path.join(H, ".cargo", "git"),
          os.path.join(H, ".cargo", "bin"), os.path.join(H, ".rustup", "toolchains"),
          os.path.join(L, "npm-cache"), os.path.join(L, "ms-playwright"),
          os.path.join(L, ".wasm-pack"), os.path.join(L, "wasm-pack")]:
    s = time.time()
    print(f"{p:60s} {walk(p)/1e6:10.1f} MB  ({time.time()-s:.0f} s)", flush=True)
print("done", flush=True)
