tmplgen(1)

# NAME

tmplgen - Void Linux template generator for language-specific package managers

# SYNOPSIS

*tmplgen* [FLAGS] [OPTIONS] <PKGNAME>

# FLAGS

*-d, --debug*
	Print debug info. Will overrule the verbose switch.

*-f, --force*
	Overwrite template if it already exists.

*-h, --help*
	Prints help information

*-n, --no-prefix*
	Don't use language-specific prefix when creating or updating packages

*-u, --update*
	Check if a new version for the package is available and if so, update _version_ and _checksum_

*-U, --UpdateAll*
	Same as -u but also update _distfiles_ and _homepage_

*-V, --version*
	Prints version information

*-v, --verbose*
	Be more verbose. Is ignored if debugging is enabled.

# OPTIONS

*-t, --tmpltype <crate/gem/perldist>*
	Explicitly sets what kind of template we want to generate

# ARGS

*<PKGNAME>*
	Name of the package to be generated

# DESCRIPTION

_tmplgen_ was created to fill the job of automatically generating xbps-src templates
from language specific package managers like _CPAN_, _crate_ and _rubygems_.

_tmplgen_ makes uses of the APIs available in each language specific package
manager to get information and fill out in a xbps-src compatible template. 
e.g. fields like _version_, _distfiles_, _homepage_, _depends_.

_tmplgen_ also creates templates recursively, walking down the full dependency tree
and creating templates for all required packages.

Other fields like _maintainer_ is automatically filled by information from 
GIT\_AUTHOR\_{NAME,EMAIL} and git config.

XBPS\_DISTDIR needs to be set for tmplgen to know where to write the templates

# NOTES

- When generating templates for the perldist tmpltype rewrite all occurrences of :: as -. e.g. Task::Kensho -> Task-Kensho

# TODO

- Download progress bar

# EXAMPLES

```
# Generate template for ffi from rubygems
$ tmplgen -t gem ffi

# Generate template for travis from rubygems
$ tmplgen -t gem travis

# Generate template for tmplgen itself
$ tmplgen -t crate tmplgen

# Generate template for Perl's Task::Kensho
$ tmplgen -t perldist Task-Kensho
```
