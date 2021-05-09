# FLAN
A dot-file manager, inspired by Sheng Chen's `Variational Typing`.
The idea is to use variational types to produce different dot-files depending on the configuration (i.e. a laptop or a desktop), avoiding the need to duplicating code. There are some extras bells and whistles for convenience, such as being able to declare variables and directly referencing envrionment variables.

## USAGE
an example of a .gitconfig using a dimension to configure autocrlf:
```
# .gitconfig_generic
[user]
name = foo
email = foo@bar.com

[core]
autocrl = #os{false##true}#
```
then running from command line the following
```
$ cat .gitconfig_generic | flan os=0 --stdin
# .gitconfig_generic
[user]
name = foo
email = foo@bar.com

[core]
autocrl = false
```
The option `os=0` indicates that for the dimension called `os` we chose the first choice (0-indexed).
It is possible to give your choices names, by declaring your dimensions in a config file (by default `.flan` or specified with the `--config` option) as follow
```
[dimensions]
os = [ "windows", "linux"]
```
we can then call the choice by name
```
$ cat .gitconfig_generic | flan os=linux --stdin
# .gitconfig_generic
[user]
name = foo
email = foo@bar.com

[core]
autocrl = false
```

the full syntax:
```
Terms := Term*
Term  :=  #$IDENTIFIER#                      // variables
       |  #$$ENV_VAR#                        // environment variables
       | `#DIMID{` Terms (`##` Terms)* `}#`  // Dimensions
       |  Text                               // anything else

DIMID := (alpha | `_`)(alphanumeric | `_`)*
IDENTIFIER := (alphanumeric | [!%&'*+-./:<=>?@_])+
```

## CONFIG
The configuration file uses a TOML syntax and the following things can be specified:
```
[options] # default command-line flags/options
force = false        # overwrite destination files
verbosity = 5        # see: `flan::cfg::ErrorFlags::report_level`
ignore_unset = false # ignores unset variables (will be substituted by blank string)
in-prefix = "./src/"     # prefix directory for input paths
out-prefix = "./dist/"    # prefix directory for output paths

[variables]
hostname = "foo"

[dimensions]
os = 2               # dimensions "os" with unnamed choices of size 2

[paths]
"source.conf" = "dest/ination.conf"  # source -> destination file mappings
```


# TODO
* optimization (cf. Domination)
* error message in config file (currently `fatal error: <toml parser error>`)
* better inference
* on call-site/scoped dimension declaration
