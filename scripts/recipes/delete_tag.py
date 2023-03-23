def setup(fs, wyag, git):
    wyag("init")
    git('config user.name "User Name"')
    git("config user.email user@example.com")
    git("config core.looseCompression 6")

    fs.write("a.txt", "a")
    fs.write("b.txt", "b")
    fs.write("c/d/e.txt", "c/d/e")
    fs.write("f/g.txt", "f/g")

    wyag("add .")
    wyag('commit -m "initial commit"')
    wyag('tag -a test_tag -m "this is the message"')

def run_wyag(wyag):
    wyag('tag -d test_tag')

def run_git(git):
    git('tag -d test_tag')
