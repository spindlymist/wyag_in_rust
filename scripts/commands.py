import subprocess

def exec_redirect_to_log(command, logger, level):
    '''Executes the provided `command`, redirecting stdout and stderr to `logger` with the specified level.'''

    logger.log(level, f"executing subprocess `{command}`:")
    logger.log(level, "{:-^80s}".format(" begin subprocess output "))

    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    with process.stdout as stdout:
        for line in iter(stdout.readline, b''):
            logger.log(level, line)

    logger.log(level, "{:-^80s}".format(" end subprocess output "))

    return_code = process.wait()
    if return_code:
        raise subprocess.CalledProcessError(return_code, command)

def exec_redirect_to_file(command, path):
    '''Executes the provided `command`, redirecting stdout and stderr to the file at `path`.'''

    with open(path, 'w') as output_file:
        subprocess.run(command, stdout=output_file, stderr=subprocess.STDOUT)
