from os import mkdir

def setup(fs, wyag, git):
    wyag("init")
    fs.write("a.txt", "a")
    fs.write("b.txt", "b")
    fs.write("c/d/e.txt", "c/d/e")
    fs.write("f/g.txt", "f/g")
    mkdir("h")

def run_wyag(wyag):
    wyag("add .")

def run_git(git):
    git("config core.looseCompression 6")
    git("add .")
