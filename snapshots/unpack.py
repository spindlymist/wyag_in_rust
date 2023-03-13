import argparse
import os
import shutil
import subprocess

# Parse arguments
parser = argparse.ArgumentParser()
parser.add_argument('-f', '--force', action='store_true', help="overwrite existing directories")
force = parser.parse_args().force

# Make sure we're in the right directory
snapshots_dir = os.path.dirname(__file__)
assert(os.path.basename(snapshots_dir) == "snapshots")
os.chdir(snapshots_dir)

# Unpack each 7z archive into a directory with the same name
for entry_name in os.listdir("."):
    if not os.path.isfile(entry_name): continue

    dir_name, ext = os.path.splitext(entry_name)
    if ext.lower() != ".7z": continue

    if os.path.isdir(dir_name):
        if not force: continue
        shutil.rmtree(dir_name)
        
    os.mkdir(dir_name)

    # 7-zip must be on PATH
    command = f"7z x {entry_name} -o{dir_name}"
    subprocess.run(command, shell=True)
