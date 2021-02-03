# Git Remote Swap

A standalone utility for changing a whole bunch of git remotes on your local machine.

It is best to run with the `--dry-run` flag first to see what will be changed.

Useful when you a large number of repositories have been migrated to a different remote. This scours your filesystem for repos with retired remotes and points them at the new URL.

```
USAGE:
    git-remote-swap [FLAGS] [OPTIONS]

FLAGS:
        --dry-run    Logs output but does not swap any remotes
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <PATH>    The path to the configuration file [default: git-remote-swap.yaml]
    -r, --root <PATH>      The root directory where the crawl should start [default: .]
```

## Config File Format

The config file format understands YAML and expects a list of `replace`/`with` urls underneath the top level `remotes` field.

Example

```yaml
remotes:
  - replace: https://bitbucket.org/old-workspace/some-repo.git
    with: git@bitbucket.org:new-workspace/new-repo-location.git
  - replace: git@bitbucket.org:old-workspace/some-repo.git
    with: git@bitbucket.org:new-workspace/new-repo-location.git
```

# Contributing

This codebase uses [VS Code Remote Containers](https://code.visualstudio.com/docs/remote/containers) for development.