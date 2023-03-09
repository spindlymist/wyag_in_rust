import os
import shutil
import argparse

# Parse arguments
parser = argparse.ArgumentParser()
parser.add_argument('-f', '--force', action='store_true', help="overwrite existing directories")
force = parser.parse_args().force

# Make sure we're in the right directory
snapshots_dir = os.path.dirname(__file__)
assert(os.path.basename(snapshots_dir) == "snapshots")
os.chdir(snapshots_dir)

# Pack each directory into a zip with the same name
for entry_name in os.listdir("."):
    if not os.path.isfile(entry_name): continue

    dir_name, ext = os.path.splitext(entry_name)
    if ext.lower() != ".zip": continue

    if os.path.isdir(dir_name):
        if not force: continue
        shutil.rmtree(dir_name)

    shutil.unpack_archive(entry_name, dir_name, 'zip')
