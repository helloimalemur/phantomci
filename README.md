## phantomCI
### Secure Headless Self-Hosted Runner
#### Makes zero unnecessary outbound connections, thereby increasing security.
#### Output is sent to stdout only, by default. (Webhook option in progress.)

## Summary
Phantom CI is a self-hosted runner in that it will detect changes on a repository and process the repository's workflow.toml file.
The workflow file exists at the root of the repo and would contain your pipeline shell commands.

This was written with the intention of isolating deployment pipelines from allowing un-owned servers unnecessary access.

Typically, a developer as a few options;
1. allow Github/etc to connect into your servers (allowing inbound connections from unowned servers)
2. install a self-hosted runner written by Github/etc (allowing outbound connections to unowned servers)
3. use a 3rd party self-hosted runner that still makes connections to un-owned servers or has an api which may have it's own security vulnerabilities.

phantom_ci also moves the declaration of the target branch off of the workflow files to it's configuration,
preventing the branch from which the workflow will run from being tampered with.
In combination with a restricted target branch we can achieve the most secure posture possible for a self-hosted runner.

When configuring, if not included `target_branch` defaults to "master".
Please use branch restrictions on the target_branch to prevent unauthorized commands from being run (should be best practice).

To solve the obvious issue of receiving notifications when a job fails or to receive job output for debugging, all output of running commands is sent to stdout. (please do not output passwords to stdout).
A webhook option with varying levels of verbosity is also up for consideration.

## $TARGET_REPO/workflow.toml
```toml
[0] ## name must be integer and correspond to the order in which commands are run
run = "pwd" ## command string
[1]
run = "make build"
[2]
run = "make deploy"
```

### a repo will only begin to be monitored after adding it to phantom_ci's configuration file.
## ~/.config/phantomCI/Repo.toml 
```toml
[sys-compare]
path = "https://github.com/helloimalemur/sys-compare"

[elktool]
path = "https://github.com/helloimalemur/ELKTool"
target_branch = "master"
```

## Installation
##### Requires [Rust](https://www.rust-lang.org/tools/install) to be installed
```shell
cargo install phantom_ci
```

## Config
Create a file named `workflow.toml` at the root of the repo you wish to poll for changes.

## Usage
```shell
## run normally
phantom_ci

## add repo to config file
phantom_ci add https://github.com/your/repo.git

## install systemd service file
phantom_ci configure service
```
