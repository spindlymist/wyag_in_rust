# Write Yourself a Git in Rust

A simplified clone of Git in Rust.

This project is educational and is not intended as a Rust-based replacement for [Git](https://git-scm.com/) or [libgit2](https://libgit2.org/). (For that, check out [gitoxide](https://github.com/Byron/gitoxide).) Do not try it out with any repository that has not been backed up.

This project initially followed [the Python-based guide written by Thibault Polge](https://wyag.thb.lt/) (hence the name) but has since diverged considerably.

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

All git commands not listed in the previous section are unavailable. Notably, `merge`, `rebase`, `revert`, and `reset` have not been implemented. Furthermore, most commands only support a subset of the options available in git.

While the `checkout` command is not implemented, `switch` and `restore` cover the majority of its use cases. In fact, these commands were created with the intent of splitting up the overloaded `checkout` command: see [commit f496b06](https://github.com/git/git/commit/f496b064fc1135e0dded7f93d85d72eb0b302c22) in the Git repo.

Additionally:

- This program has only been tested on Windows. Notably, treatment of file stats and permissions has been simplified. Also, its behavior with symlinks is undefined and likely incorrect.
- `.gitignore` is not supported.
- Packfiles are not supported.
- Remotes are not supported.
- Commands that take a pathspec in git only accept a path.
- Commands that take a tree-ish in git accept either only a commit identifier or only a tree hash.
- The `switch` command will not switch branches if there are any uncommitted changes in the index or working directory. (Git allows this as long as the operation is nondestructive.)
- Index extensions are not supported. Any extension data present is erased when the index is updated.
- Most config options are not supported. Global config is not supported at all.
- The `log` command outputs a representation of the commit graph in the graph description language [DOT](https://en.wikipedia.org/wiki/DOT_(graph_description_language)). It can be visualized with [Graphviz](https://graphviz.org/) ([try it here](https://dreampuf.github.io/GraphvizOnline/)).

## Tests

Unit tests are present in many modules where appropriate, but because Git primarily operates on the file system, the testing strategy relies heavily on integration tests.

### Snapshots

In addition to the usual obstacles involved when testing code that interacts with the file system, there is the added dimension that file timestamps (e.g. modification time) are meaningful (in particular, they are stored in the index file). To overcome this, the testing apparatus is based around "snapshots" stored in `.7z` archives, which are able to save and restore the timestamps associated with their contents. These archives are stored in the [`/snapshots`](/snapshots) directory.

Most integration tests follow this procedure:

1. Extract a "before" snapshot to a temporary directory.
2. Execute a command on the temp directory.
3. Extract an "after" snapshot and compare it to the temp directory.

### Recipes

To generate and vet the correctness of these snapshots, a Python script [`/scripts/snapshots.py`](scripts/snapshots.py) has been provided. Each snapshot is based on a "recipe" located in the [`/scripts/recipes`](scripts/recipes) directory. A recipe is a Python script that supplies the steps to generate the "before" snapshot (via the `setup` function), the wyag command(s) being tested (`run_wyag` function), and the equivalent Git commands (`run_git` function) which serve as the ground truth.

When the recipe is executed with the `snapshots.py generate` command, two identical copies of the "before" snapshot are created. The wyag commands are executed on one copy and the git commands are executed on the other. Then, a diff of the two directories and the contents of their index files is produced. The user is asked to confirm that any differences between the two are acceptable. (Differences often arise from unimplemented features rather than errors.) If accepted, two archives are created in the [`/snapshots`](/snapshots) directory: `before_<recipe name>.7z` and `after_<recipe name>.7z`. By convention, a recipe should use the exact name of the test that it supports.

To use the script, `git` and `7z` must be present on your `$PATH`. To learn more, run this command from the project's root directory:

```shell
python scripts/snapshots.py --help
```

### Coverage

Integration tests have been written for almost all commands that modify the file system, including:

- `add`
- `branch`
- `commit`
- `hash-object`
- `init`
- `rm`
- `tag`

Most other commands simply read from the file system and output information to the console. Notably, however, the `switch` and `restore` commands (which do modify the file system) lack appropriate coverage due to challenges with timestamps that will require a rework of the testing apparatus.

## Future Work

Due to the educational nature of this project, I have limited plans to continue its development. However, I plan to implement the most critical missing feature, the `merge` command, which is the final piece required to support a multi-branch workflow.

If I were to continue development beyond that, I would:
- Address the most pressing limitations described above.
- Take better advantage of Rust's type system by making more use of traits and reducing reliance on primitive types.
- Develop a unified system for traversing, comparing, modifying, and converting between the three file trees (`WorkDir`, `Index`, and `Tree`). Their differences pose a challenge to creating a coherent, performant abstraction, but I believe it is possible.
- Improve algorithmic clarity. Many algorithms that use the file system could be made clearer (and perhaps even more performant) by separating file system interactions from processing at the cost of consuming more memory.
- Refactor the `object` module. It is rare to abstract over multiple types of objects, so the `GitObject` type is not extremely useful. I would rather use a trait and replace the methods of `GitObject` with a set of functions that are generic over that trait.
- Limit the direct dependence of modules on the file system. This would allow for more extensive unit testing.
- Improve the testing apparatus and test coverage. In particular, I would fold the functionality of the Python scripts into the [`common` test module](/tests/common) module. Then, I would incorporate the recipes into the tests they support, reducing the maintenance burden. Finally, I would automate comparisons to the ground truth as much as possible, supported by unit tests to ensure the integrity of the system.
- Improve error reporting. In particular, adding context ([`anyhow::Context`](https://docs.rs/anyhow/latest/anyhow/trait.Context.html)) or bespoke errors in the many places where I/O errors can occur would improve the legibility of error messages.
