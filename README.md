# fix-cli
Command line tools for FIX protocol.

## Tools

* fixcat - prints FIX log file in more human readable form.

## Install

Download binaries from here:
https://github.com/rustamkulenov/fix-cli/releases

or build from sources:
```
$ cargo build --release
```

## Usage:

* fixcat \<filename\>
* cat fixlog.log | fixcat

## How it works

`$ cat fixlog.log`
![cat](/img/cat.png)

`$ fixcat fixlog.log`
![fixcat](/img/fixcat.png)