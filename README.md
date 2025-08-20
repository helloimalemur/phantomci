# 🐱 `phantom_ci`
### ⚙️ Secure, Headless, Self-Hosted CI Runner
> ✅ Zero unnecessary outbound connections  
> 📤 Output to stdout by default (with optional webhooks)  
> 🔒 Built for minimal trust surfaces

---

## 🧠 Summary

**`phantom_ci`** is a fully self-hosted CI runner that detects changes in Git repositories and executes pipeline steps defined in a per-branch TOML workflow file.  
All execution happens **locally**, as the user who runs `phantom_ci`. No external services are contacted unless explicitly configured.

This project was built with **isolation and security** in mind — specifically to prevent granting inbound or outbound access to unowned servers.

---

## 🚫 Common CI Tradeoffs vs `phantom_ci`

| Approach                                          | Tradeoff                                           |
|--------------------------------------------------|----------------------------------------------------|
| GitHub Actions / SaaS Runners                    | Inbound access from GitHub into your servers       |
| GitHub’s Self-Hosted Runners                     | Outbound access to GitHub's infra                  |
| 3rd-party Runners                                | Implicit outbound connections or exposed APIs      |
| ✅ `phantom_ci`                                   | **No inbound or outbound access required**         |

---

## 🛡️ Security Posture

- Workflows are only run from a **locally configured branch** (`target_branch`).
- Branch execution config is stored **outside the repo**, reducing tampering risk.
- CLI-based only — no API, no sockets, no network listeners.
- Workflow steps are executed via `std::process::Command`.

Default `target_branch` is `master` — configure this explicitly and enforce restrictions via Git to avoid unauthorized command execution.

---

## 📦 Workflow Location and Format

Place workflows under the repository root at:

```text
$REPO_ROOT/workflow/<branch>.toml
```

Example for branch `master`:

```toml
[0] # step index must be numeric and define execution order
run = "pwd"

[1]
run = "make build"

[2]
run = "make deploy"
```

Rules:
- Only numeric tables are supported (e.g., `[0]`, `[1]`, ...). Lower numbers run first.
- Each step supports a single key: `run` (a shell command invoked without a shell).
- Commands run with the working directory set to the checked-out repo directory.

See `examples/workflow.toml` for a more complete example.

---

## 📁 Repo Configuration

Monitored repositories are defined in a `Repo.toml` inside your user config directory.

- Linux: `~/.config/phantom_ci/Repo.toml`
- macOS: `~/Library/Application Support/com.helloimalemur.phantom_ci/Repo.toml`

```toml
[sys-compare]
path = "https://github.com/helloimalemur/sys-compare"
target_branch = "master"

[elktool]
path = "https://github.com/helloimalemur/ELKTool"
target_branch = "master"

[elktool2] # section headers must be unique
path = "git@github.com:helloimalemur/ELKTool" # SSH recommended
target_branch = "test-branch" # ensure the branch exists
```

---

## 🔔 Webhook Notifications (Optional)

Create a `.env` file in your user config directory to enable webhooks:

- Linux: `~/.config/phantom_ci/.env`
- macOS: `~/Library/Application Support/com.helloimalemur.phantom_ci/.env`

Supported variables:

```env
# Discord
DISCORD_WEBHOOK_URL="https://discord.com/api/webhooks/..."

# Slack
SLACK_WEBHOOK_URL="https://hooks.slack.com/services/..."
```

---

## 🚀 Installation

Requires [Rust](https://www.rust-lang.org/tools/install):

```bash
cargo install phantom_ci
```

---

## ⚙️ Usage

```bash
# Run normally (polls repos and executes workflows on changes)
phantom_ci

# Add a repo (path + branch are required)
phantom_ci add https://github.com/your/repo master
# or via SSH (recommended)
phantom_ci add git@github.com:your/repo main

# Install systemd service (Linux)
phantom_ci configure service

# Inspect state
phantom_ci repo              # list repos and latest job status
phantom_ci jobs              # list jobs
phantom_ci logs              # list logs
phantom_ci reset             # stop service, clear caches, and restart
```

---

## 💡 Notes on Workflows

- Place files at `$REPO_ROOT/workflow/<branch>.toml`.
- Steps run sequentially in numeric order.
- Each step exposes only `run` and does not spawn a shell; if you need shell features, invoke `bash -lc "..."` explicitly.
- Output is captured and printed to stdout. Webhooks (if configured) receive command output.

---

## 💣 Development & Contribution

Contributions welcome — PRs encouraged!

```bash
cargo clippy -- -D clippy::all
cargo fmt -- --check
```
