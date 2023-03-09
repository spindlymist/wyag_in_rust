import os
import shutil
import argparse

# Parse arguments
parser = argparse.ArgumentParser()
parser.add_argument('-f', '--force', action='store_true', help="overwrite existing archives")
force = parser.parse_args().force

# Make sure we're in the right directory
snapshots_dir = os.path.dirname(__file__)
assert(os.path.basename(snapshots_dir) == "snapshots")
os.chdir(snapshots_dir)

# Pack each directory into a zip with the same name
for entry_name in os.listdir("."):
    if not os.path.isdir(entry_name): continue

    zip_name = f"{entry_name}.zip"
    if os.path.isfile(zip_name):
        if not force: continue
        os.remove(zip_name)

    shutil.make_archive(entry_name, 'zip', entry_name)
