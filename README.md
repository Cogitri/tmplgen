# tmplgen

A simple Void Linux template generator.

## Usage
```
USAGE:
    tmplgen [FLAGS] [OPTIONS] <PKGNAME>

FLAGS:
    -d, --debug      Print debug info
    -f, --force      Overwrite template, if it already exists
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Be more verbose

OPTIONS:
    -t, --tmpltype <crate/gem>    Explicitly sets what kind of template we want to generate

ARGS:
    <PKGNAME>    Sets for which package the template should be generated
```
