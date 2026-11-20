"""Recursive delete that never follows a link: removes every junction/symlink itself first."""
import os, stat, sys

def is_link(e):
    return e.is_symlink() or (hasattr(e, "is_junction") and e.is_junction())

def find_links(path, out):
    with os.scandir(path) as it:
        for e in it:
            if is_link(e):
                out.append(e.path)
            elif e.is_dir(follow_symlinks=False):
                find_links(e.path, out)

def rm(path, failed):
    with os.scandir(path) as it:
        for e in it:
            if is_link(e):
                raise SystemExit("link found mid-delete: " + e.path)
            if e.is_dir(follow_symlinks=False):
                rm(e.path, failed)
            else:
                try:
                    os.chmod(e.path, stat.S_IWRITE)
                    os.unlink(e.path)
                except OSError as ex:
                    failed.append((e.path, str(ex)))
    try:
        os.rmdir(path)
    except OSError as ex:
        failed.append((path, str(ex)))

target = sys.argv[1]
top = os.path.basename(os.path.dirname(target))
assert os.path.isdir(target), target
links = []
find_links(target, links)
for l in links:
    print("removing link", l)
    try:
        os.rmdir(l)
    except OSError:
        os.unlink(l)
failed = []
rm(target, failed)
print("failed:", len(failed))
for f in failed[:20]:
    print(f)
