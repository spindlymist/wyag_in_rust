import argparse
import os
import subprocess

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

    archive_name = f"{entry_name}.7z"
    if os.path.isfile(archive_name):
        if not force: continue
        os.remove(archive_name)

    # 7-zip must be on PATH
    command = f"7z a {entry_name} ./{entry_name}/* -mtc -mtm -mta"
    subprocess.run(command, shell=True)
