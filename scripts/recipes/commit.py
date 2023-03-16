def setup(fs, wyag, git):
    wyag("init")
    git("config user.name \"User Name\"")
    git("config user.email user@example.com")
    fs.write("x.txt", "x")
    fs.write("a/b/c/d.txt", "a/b/c/d")
    fs.write("y/z.txt", "y/z")
    wyag("add .")
    wyag("commit -m \"initial commit\"")
    fs.write("a/b/c.txt", "a/b/c")
    fs.write("a/b/d.txt", "a/b/d")
    wyag("add .")

def run_wyag(wyag):
    wyag("commit -m \"second commit\"")

def run_git(git):
    git("config core.looseCompression 6")
    git("commit -m \"second commit\"")
