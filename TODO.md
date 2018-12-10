# TODO

## Crates
* Figure out native dependencies for recursive deps

## Doc
* Add a manpage

## Updating
* Think of something smart to not download distfiles twice if the template uses a different distfile than pkg_info.download_url

## Perldists
* Make generating templates faster. Right now we have to query the target package and its deps twice (to figure out if its a module or a dist)