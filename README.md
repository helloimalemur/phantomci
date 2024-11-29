## phantomCI
### Secure Headless Self-Hosted Runner
#### Makes zero unnecessary outbound connections, thereby increasing security.
#### Output is sent to stdout only, by default. (Webhook option in progress.)

## config/Repo.toml
```toml
[sys-compare]
path = "https://github.com/helloimalemur/sys-compare"

[elktool]
path = "https://github.com/helloimalemur/ELKTool"
target_branch = "master"
```

## ./workflow.toml 
```toml
[0] ## name must be integer and correspond to the order in which commands are run
run = "ls -lah" ## command string
[1]
run = "pwd"
[2]
run = "rm -rf testproj/"
[3]
run = "cargo new testproj"
[4]
run = "cargo add tokio --manifest-path testproj/Cargo.toml"
[5]
run = "cargo run --manifest-path testproj/Cargo.toml"
[6]
run = "rm -rf testproj/"
```
