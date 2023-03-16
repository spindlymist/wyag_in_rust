def write(path, contents):
    '''Writes the string `contents` to a file at `path`.'''

    with open(path, 'w') as file:
        file.write(contents)
