import os
import shutil

def write(path, contents):
    '''Writes the string `contents` to a file at `path`. Nonexistent directories will be created.'''

    parent_dir = os.path.dirname(path)
    if len(parent_dir) > 0 and not os.path.isdir(parent_dir):
        os.makedirs(parent_dir)

    with open(path, 'w') as file:
        file.write(contents)

def remove(path):
    '''Removes the file or directory at `path`.'''

    if os.path.isdir(path):
        shutil.rmtree(path)
    elif os.path.isfile(path):
        os.remove(path)
