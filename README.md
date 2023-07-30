# clone-github-subdir

Clone any sub-directory of a Github repository to your local machine with this handy command-line tool. It is useful if you only want to clone a specific part of a repo.

## Installation

You can install clone-github-subdir from crates.io using cargo:

```cmd
cargo install clone-github-subdir
```

Or you can build it from source using git:

```cmd
git clone https://github.com/isurfer21/clone-github-subdir.git
cd clone-github-subdir
cargo build --release
```

## Usage

To use clone-github-subdir, first check if all set via short command `cgs` along with `--help` or `-h` option, e.g.,

```cmd
> cgs --help
Usage:
 cgs [options] <link>

Arguments:
 link              Github sub-directory URL

Options:
 -h, --help         Show this help message
 -v, --version      Show the program version
 -c, --curdir       Current sub-directory only
```

You can also use the following options:

- `-h, --help`: Show the help message and exit.
- `-v, --version`: Show the program version and exit.
- `-c, --curdir`: Clone only the current sub-directory, not its parent directory.

Now to clone the GitHub sub-directory, you can provide any GitHub repository's sub-directory URL as an argument, e.g.,

```cmd
cgs https://github.com/second-state/wasm-learning/tree/master/nodejs/hello
```

This will clone the `nodejs/hello` sub-directory of the `second-state/wasm-learning` repo to your current working directory.

Alternatively, to clone only the target sub-directory, you can use `--curdir` or `-c` option , e.g.,

```cmd
cgs -c https://github.com/second-state/wasm-learning/tree/master/nodejs/hello
```

This will clone only the `hello` sub-directory, not its parent directory like `nodejs/hello`.

## License

This project is licensed under the MIT license. See the [LICENSE](LICENSE) file for more details.