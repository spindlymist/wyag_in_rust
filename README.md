# Write Yourself a Git in Rust

A simplified clone of Git in Rust.

This project initially followed [a guide written by Thibault Polge](https://wyag.thb.lt/) (which uses Python) but has since diverged considerably. It was created solely for the edification of its author and is not intended for serious use. Do not try it with any repository that has not been backed up.

## Features

The basic functionality of the following git commands has been implemented:

- `add`
- `branch`
- `cat-file`
- `commit`
- `hash-object`
- `init`
- `log`
- `ls-files`
- `ls-tree`
- `restore`
- `rev-parse`
- `rm`
- `show-ref`
- `status`
- `switch`
- `tag`

This subset of commands is sufficient for a single-branch workflow. `branch` and `switch` allow the creation and use of additional branches, but the unimplemented `merge` command is needed to take full advantage of them.

## Limitations

All git commands not listed in the previous section are unavailable. Notably, `merge`, `rebase`, and `revert`, and `reset` have not been implemented. Most commands only support a subset of the options available in git. Additionally:

- `.gitignore` is not respected.
- Commands that take a pathspec in git only accept a path.
- Commands that take a tree-ish in git accept either only a commit identifier or only a tree hash.
- The `switch` command will not switch branches if there are any uncommited changes in the index or working directory. (Git allows this as long as the operation is nondestructive.)
- Index extensions are not supported. Any extension data present is erased when the index is updated.
- The `log` command outputs a representation of the commit graph in the graph description language [DOT](https://en.wikipedia.org/wiki/DOT_(graph_description_language)). It can be visualized with [Graphviz](https://graphviz.org/) ([try it here](https://dreampuf.github.io/GraphvizOnline/)).