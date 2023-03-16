from os import remove

def setup(fs, wyag, git):
    wyag("init")
    fs.write("x.txt", "x")
    fs.write("a/b.txt", "a/b")
    fs.write("a/c.txt", "a/c")
    fs.write("a/b/c.txt", "a/b/c")
    fs.write("a/b/d.txt", "a/b/d")
    fs.write("a/b/c/d.txt", "a/b/c/d")
    fs.write("y/x.txt", "y/x")
    fs.write("y/z.txt", "y/z")
    wyag("add .")
    remove("x.txt")
    remove("a/b/c.txt")
    remove("a/b/d.txt")

def run_wyag(wyag):
    wyag("add x.txt")

def run_git(git):
    git("config core.looseCompression 6")
    git("add x.txt")
