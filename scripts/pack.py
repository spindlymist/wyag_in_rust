import logging
import os

from commands import exec_redirect_to_log

logger = logging.getLogger(__name__)

def pack_subdirectories(dir_path, force=False, ignore_prefix=('.', '_')):
    '''Packs each subdirectory in `dir_path` into an archive with the same name.

    Ignores directories starting with `ignore_prefix` (string or tuple).
    If `force` is `True`, existing directories will be overwritten.
    '''

    logger.info(f"packing subdirectories of `{dir_path}`")

    current_dir = os.getcwd()
    os.chdir(dir_path)

    for entry_name in os.listdir("."):
        if entry_name.startswith(ignore_prefix):
            logger.debug(f"ignored `{entry_name}` (prefix)")
            continue
        if not os.path.isdir(entry_name):
            logger.debug(f"ignored `{entry_name}` (not directory)")
            continue

        archive_name = f"{entry_name}.7z"
        if os.path.isfile(archive_name):
            if not force:
                logger.info(f"skipped packing `{entry_name}` (archive exists, force={force})")
                continue
            logger.info(f"removing archive `{archive_name}` (force={force})")
            os.remove(archive_name)
        
        pack_directory(entry_name, archive_name)
    
    os.chdir(current_dir)

def pack_directory(dir_path, archive_path):
    '''Packs the directory at `dir_path` into an archive at `archive_path`.'''

    logger.info(f"packing `{dir_path}` to `{archive_path}`")

    archive_path = os.path.abspath(archive_path)

    current_dir = os.getcwd()
    os.chdir(dir_path)

    command = f"7z a {archive_path} . -mtc -mtm -mta"
    exec_redirect_to_log(command, logger, logging.DEBUG)

    os.chdir(current_dir)
