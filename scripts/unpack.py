import logging
import os
import shutil

from commands import exec_redirect_to_log

logger = logging.getLogger(__name__)

def unpack_directory(dir_path, force=False):
    '''Unpacks all archives in `dir_path` into directories with the same name.

    If `force` is `True`, existing archives will be overwritten
    '''

    logger.info(f"unpacking directory at `{dir_path}`")

    current_dir = os.getcwd()
    os.chdir(dir_path)

    for entry_name in os.listdir("."):
        if not os.path.isfile(entry_name):
            logger.debug(f"ignored `{entry_name}` (not file)")
            continue

        dir_name, ext = os.path.splitext(entry_name)
        if ext.lower() != ".7z":
            logger.debug(f"ignored `{entry_name}` (not 7z)")
            continue

        if os.path.isdir(dir_name):
            if not force:
                logger.info(f"skipped unpacking `{entry_name}` (directory exists, force={force})")
                continue
            logger.info(f"removing directory `{dir_name}` (force={force})")
            shutil.rmtree(dir_name)

        unpack_archive(entry_name, dir_name)

    os.chdir(current_dir)

def unpack_archive(archive_path, dir_path):
    '''Unpacks the archive at `archive_path` into the directory at `dir_path`.'''

    logger.info(f"unpacking `{archive_path}` to `{dir_path}`")

    os.makedirs(dir_path)

    command = f"7z x {archive_path} -o{dir_path}"
    exec_redirect_to_log(command, logger, logging.DEBUG)
