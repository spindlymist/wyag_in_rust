from os import mkdir

def setup(fs, wyag, git):
    wyag("init")
    fs.write("a.txt", "a")
    fs.write("b.txt", "b")
    fs.write("c/d/e.txt", "c/d/e")
    fs.write("f/g.txt", "f/g")

def run_wyag(wyag):
    wyag("add c/d/e.txt")

def run_git(git):
    git("config core.looseCompression 6")
    git("add c/d/e.txt")
