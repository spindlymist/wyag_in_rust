def setup(fs, wyag, git):
    wyag("init")
    fs.write("hello.txt", "hello")
    fs.write("goodbye.txt", "goodbye")
    fs.write("test.txt", "test")

def run_wyag(wyag):
    wyag("add .")

def run_git(git):
    git("config core.looseCompression 6")
    git("add .")
