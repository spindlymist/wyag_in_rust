def setup(fs, wyag, git):
    wyag("init")
    git("config user.name \"User Name\"")
    git("config user.email user@example.com")
    git("config core.looseCompression 6")

    fs.write("a.txt", "a")
    fs.write("b.txt", "b")

    wyag("add .")
    wyag('commit -m "initial commit"')
    wyag("branch test_branch")
    git("switch test_branch")

    fs.write("c/d/e.txt", "c/d/e")
    fs.write("f/g.txt", "f/g")

    wyag("add .")
    wyag('commit -m "second commit to test_branch"')
    git("switch master")

    fs.remove(".git/logs")

def run_wyag(wyag):
    wyag("switch test_branch")

def run_git(git):
    git("switch test_branch")
