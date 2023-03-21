def setup(fs, wyag, git):
    wyag("init")
    git("config user.name \"User Name\"")
    git("config user.email user@example.com")
    git("config core.looseCompression 6")

    fs.write("a.txt", "a")
    fs.write("b.txt", "b")

    wyag("add .")
    wyag('commit -m "initial commit"')
    wyag("tag starting_point")

    fs.write("c/d/e.txt", "c/d/e")
    fs.write("f/g.txt", "f/g")

    wyag("add .")
    wyag('commit -m "second commit to master"')

    fs.remove(".git/logs")

def run_wyag(wyag):
    wyag("switch --detach starting_point")

def run_git(git):
    git("switch --detach starting_point")
