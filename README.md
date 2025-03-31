# 🐱 `phantom_ci`
### ⚙️ Secure, Headless, Self-Hosted CI Runner
> ✅ Zero unnecessary outbound connections  
> 📤 Output to stdout by default (with optional webhooks)  
> 🔒 Built for minimal trust surfaces

---

## 🧠 Summary

**`phantom_ci`** is a fully self-hosted CI runner that detects changes in Git repositories and executes pipeline steps defined in a `workflow.toml` file.  
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

- Workflows are only run from a **locally configured branch** (`target_branch`)
- Branch execution config is stored **outside the repo**, reducing tampering risk
- CLI-based only — no API, no sockets, no network listeners
- Workflow steps are executed via `std::process::Command` with optional sandboxing

Default `target_branch` is `"master"` — configure this explicitly and enforce restrictions via Git to avoid unauthorized command execution.

---

## 📦 Example: `$REPO_ROOT/workflow/master.toml`

```toml
[0] # step index must be numeric and define execution order
run = "pwd"

[1]
run = "make build"

[2]
run = "make deploy"
```

---

## 📁 Repo Configuration

Monitored repositories are defined in:

```text
~/.config/phantom_ci/Repo.toml
```

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

To enable Discord webhook notifications, create a `.env` file at:

```text
~/.config/phantom_ci/.env
```

```env
DISCORD_WEBHOOK_URL="https://discord.com/api/webhooks/..."
```

Additional options for verbosity and payload formatting are planned.

---

## 🚀 Installation

Requires [Rust](https://www.rust-lang.org/tools/install):

```bash
cargo install phantom_ci
```

---

## ⚙️ Usage

```bash
# Run normally
phantom_ci

# Add repo via HTTPS
phantom_ci add https://github.com/your/repo

# Add repo via SSH (recommended)
phantom_ci add git@github.com:your/repo

# Install systemd service
phantom_ci configure service
```

---

## 💡 Workflow Configuration

Create a `workflow.toml` at the root of any monitored repo.  
Steps are executed in numeric key order.

---

## 💣 Development & Contribution

Contributions welcome — PRs encouraged!

```bash
cargo clippy -- -D clippy::all
cargo fmt -- --check
```
