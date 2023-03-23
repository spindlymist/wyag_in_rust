def setup(fs, wyag, git):
    wyag("init")
    fs.write("a.txt", "a")

def run_wyag(wyag):
    wyag("hash-object -w a.txt")

def run_git(git):
    git("config core.looseCompression 6")
    git("hash-object -w a.txt")
