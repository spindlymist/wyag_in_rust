import os
import importlib.util
import logging
import shutil
import stat
import subprocess
import sys

from commands import exec_redirect_to_file, exec_redirect_to_log
import pack
import unpack
import recipe_helpers

logger = logging.getLogger(__name__)

def has_callable(obj, attr):
    '''Returns `True` if `obj` has a callable attribute `attr`.'''

    return hasattr(obj, attr) and callable(getattr(obj, attr))

def import_recipe(name):
    '''Imports and validates the recipe from recipes/`name`.py.'''
    
    recipe_path = os.path.join(os.path.dirname(__file__), "recipes", f"{name}.py")
    logging.debug(f"importing recipe from `{recipe_path}`")

    spec = importlib.util.spec_from_file_location(f"recipes.{name}", recipe_path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)

    if has_callable(module, 'setup') \
        and has_callable(module, 'run_wyag') \
        and has_callable(module, 'run_git'):
        return module
    else:
        logging.warn(f"invalid recipe: `{recipe_path}`")
        return None

def delete_readonly(function, path, excinfo):
    os.chmod(path, stat.S_IWRITE)
    os.remove(path)

def remove_path(path, force):
    '''Removes the file or directory at `path`.
    Returns `True` if it was removed or never existed.

    Unless `force` is `True`, the user will be prompted to confirm before deleting.'''

    path = os.path.abspath(path)

    if os.path.isdir(path):
        if force or input(f"delete directory `{path}` (y/n)? ").lower() == 'y':
            shutil.rmtree(path, onerror=delete_readonly)
            return True
    elif os.path.isfile(path):
        if force or input(f"delete file `{path}` (y/n)? ").lower() == 'y':
            os.remove(path)
            return True
    else:
        return True

    return False

def clone_directory(original_path, archive_path, clone_paths):
    '''Clones the directory at `original_path` to each directory in the list of `clone_paths`,
    preserving the created, modified, and accessed times of its contents. In the process,
    the directory at `original_path` will be packed into an archive at `archive_path`.'''

    logging.info(f"cloning `{original_path}` to {clone_paths}")

    pack.pack_directory(original_path, archive_path)

    for clone_path in clone_paths:
        unpack.unpack_archive(archive_path, clone_path)
    
    return True

def generate_all_snapshots(force):
    '''Generates a snapshot from each recipe in the recipes directory.
    If `force` is `True`, existing snapshots will be regenerated.'''

    logging.info(f"generating all snapshots")

    recipes_dir = os.path.join(os.path.dirname(__file__), "recipes")
    for entry_name in os.listdir(recipes_dir):
        entry_path = os.path.join(recipes_dir, entry_name)
        name, ext = os.path.splitext(entry_name)
        if os.path.isfile(entry_path) and ext == ".py":
            generate_snapshot(name, force)

def generate_snapshots(names, force):
    '''Generates a snapshot from each recipe specified in `names`.
    If `force` is `True`, existing snapshots will be regenerated.'''
    for name in names:
        generate_snapshot(name, force)

def generate_snapshot(name, force):
    '''Generates a snapshot from the recipe called `name`.
    If `force` is `True` and there exists a snapshot by that name, it will be overwritten.'''

    # construct paths
    current_dir = os.getcwd()

    before_name = f"before_{name}"
    before_archive = f"{before_name}.7z"
    before_dir = os.path.abspath(before_name)

    after_name = f"after_{name}"
    after_archive = f"{after_name}.7z"
    after_dir = os.path.abspath(after_name)

    git_after_dir = os.path.join(os.path.dirname(__file__), "__scratch", f"git_{after_name}")
 
    # check if snapshot exists
    if not force and os.path.isfile(before_archive) and os.path.isfile(after_archive):
        logging.info(f"skipping snapshot `{name}` (force={force})")
        return

    print("{:=^80s}".format(f" {name} "))    
    logging.info(f"generating snapshot `{name}`...")

    # import recipe
    recipe = import_recipe(name)
    if recipe is None:
        return
    
    logging.debug("done importing recipe, running setup...")

    # set up initial state
    call_wyag = lambda command: exec_redirect_to_log(f"cargo r -- {command}", logger, logging.DEBUG)
    call_git = lambda command: exec_redirect_to_log(f"git {command}", logger, logging.DEBUG)

    if not remove_path(before_dir, force): return
    os.mkdir(before_dir)
    os.chdir(before_dir)
    recipe.setup(recipe_helpers, call_wyag, call_git)
    os.chdir(current_dir)

    logging.debug("done with setup, duplicating...")

    # duplicate initial state
    if not remove_path(before_archive, force) \
        or not remove_path(after_dir, force) \
        or not remove_path(git_after_dir, force):
        return
    clone_directory(before_dir, f"{before_name}.7z", [after_dir, git_after_dir])

    logging.debug("done duplicating setup, running wyag commands...")

    # run wyag commands
    os.chdir(after_dir)
    recipe.run_wyag(call_wyag)
    os.chdir(current_dir)

    logging.debug(f"done with wyag commands, running git commands...")

    # run git commands
    os.chdir(git_after_dir)
    recipe.run_git(call_git)
    os.chdir(current_dir)

    logging.debug(f"done with git commands, comparing results...")

    # compare
    os.chdir(after_dir)
    wyag_ls_files_output = os.path.join(os.path.dirname(__file__), "__scratch", "wyag_ls-files.txt")
    exec_redirect_to_file("git ls-files -s --debug", wyag_ls_files_output)

    os.chdir(git_after_dir)
    git_ls_files_output = os.path.join(os.path.dirname(__file__), "__scratch", "git_ls-files.txt")
    exec_redirect_to_file("git ls-files -s --debug", git_ls_files_output)

    os.chdir(current_dir)

    print("{:-^80s}".format(" begin directory diff "))
    subprocess.run(f"git diff --no-index {git_after_dir} {after_dir}")
    print("{:-^80s}".format(" end directory diff "))

    if input("accept differences (y/n)? ").lower() != 'y':
        logging.info(f"differences were rejected for snapshot `{name}`")
        return

    print("{:-^80s}".format(" begin ls-files diff "))
    subprocess.run(f"git diff --no-index {git_ls_files_output} {wyag_ls_files_output}")
    print("{:-^80s}".format(" end ls-files diff "))

    if input("accept differences (y/n)? ").lower() != 'y':
        logging.info(f"differences were rejected for snapshot `{name}`")
        return
    
    # save snapshot and clean up
    if not remove_path(after_archive, force): return
    pack.pack_directory(after_dir, f"{after_name}.7z")
    os.remove(wyag_ls_files_output)
    os.remove(git_ls_files_output)
    shutil.rmtree(git_after_dir, onerror=delete_readonly)

    logging.info(f"done generating snapshot `{name}`")
