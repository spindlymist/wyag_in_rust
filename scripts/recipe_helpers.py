import os

def write(path, contents):
    '''Writes the string `contents` to a file at `path`. Nonexistent directories will be created.'''

    parent_dir = os.path.dirname(path)
    if len(parent_dir) > 0 and not os.path.isdir(parent_dir):
        os.makedirs(parent_dir)

    with open(path, 'w') as file:
        file.write(contents)
