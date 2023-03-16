def setup(fs, wyag, git):
    wyag("init")
    git("config user.name \"User Name\"")
    git("config user.email user@example.com")

    fs.write("x.txt", "x")
    fs.write("a/b.txt", "a/b")
    fs.write("a/c.txt", "a/c")
    fs.write("a/b/c.txt", "a/b/c")
    fs.write("a/b/d.txt", "a/b/d")
    fs.write("a/b/c/d.txt", "a/b/c/d")
    fs.write("y/x.txt", "y/x")
    fs.write("y/z.txt", "y/z")
    
    wyag("add .")
    wyag('commit -m "initial commit"')

def run_wyag(wyag):
    wyag("rm a/b")

def run_git(git):
    git("rm -r a/b")
