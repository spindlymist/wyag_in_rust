import argparse
import os
import logging

import pack
import unpack
import generate

# Command handlers
def handle_pack(args):
    pack.pack_subdirectories(".", force=args.force)

def handle_unpack(args):
    unpack.unpack_directory(".", force=args.force)

def handle_generate(args):
    if args.all and len(args.snapshot) == 0:
        generate.generate_all_snapshots(args.force)
    elif args.all:
        print("list of snapshots is not allowed when the --all switch is present")
    else:
        generate.generate_snapshots(args.snapshot, args.force)

# Set up logging functionality
logger = logging.getLogger()
logger.setLevel(logging.DEBUG)
log_path = os.path.join(os.path.dirname(__file__), "snapshots.log")
file_handler = logging.FileHandler(log_path, mode='w')
file_handler.setLevel(logging.DEBUG)
file_formatter = logging.Formatter("%(asctime)s %(levelname)s %(name)s | %(message)s")
file_handler.setFormatter(file_formatter)
logger.addHandler(file_handler)

# Switch to snapshots directory
snapshots_dir = os.path.join(os.path.dirname(__file__), "../snapshots")
if not os.path.isdir(snapshots_dir):
    os.mkdir(snapshots_dir)
os.chdir(snapshots_dir)

# Make scratch directory
scratch_path = os.path.join(os.path.dirname(__file__), "__scratch")
if not os.path.isdir(scratch_path):
    os.mkdir(scratch_path)

# Parse command line args
parser = argparse.ArgumentParser()
subparsers = parser.add_subparsers(dest="command", required=True)

pack_parser = subparsers.add_parser("pack", help="make archives from directories")
pack_parser.add_argument("-f", "--force", action="store_true", help="overwrite existing archives")
pack_parser.set_defaults(handler=handle_pack)

unpack_parser = subparsers.add_parser("unpack", help="make directories from archives")
unpack_parser.add_argument("-f", "--force", action="store_true", help="overwrite existing directories")
unpack_parser.set_defaults(handler=handle_unpack)

generate_parser = subparsers.add_parser("generate", help="generate snapshots")
generate_parser.add_argument("snapshot", nargs='*')
generate_parser.add_argument("-a", "--all", action="store_true", help="generate all snapshots")
generate_parser.add_argument("-f", "--force", action="store_true", help="overwrite existing snapshots")
generate_parser.set_defaults(handler=handle_generate)

args = parser.parse_args()
logger.info(f"args: {args}")
args.handler(args)
