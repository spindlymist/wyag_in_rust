def setup(fs, wyag, git):
    wyag("init")
    git("config user.name \"User Name\"")
    git("config user.email user@example.com")
    git("config core.looseCompression 6")

    fs.write("a.txt", "a")
    fs.write("b.txt", "b")
    fs.write("c/d/e.txt", "c/d/e")
    fs.write("f/g.txt", "f/g")

    wyag("add . ")
    wyag('commit -m "initial commit"')
    wyag("branch test_branch")
    wyag("switch test_branch")

def run_wyag(wyag):
    wyag("branch -d test_branch")

def run_git(git):
    try:
        git("branch -d test_branch")
    except:
        pass
