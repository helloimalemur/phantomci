# AGENTS.md: phantomci

A self-hosted CI runner: it polls the repos in its `Repo.toml` and runs each repo's per-branch
workflow (`workflow/<branch>.toml`, e.g. `deploy.toml`) when that branch gets a new commit.

This repository may be public: keep infrastructure details (hosts, clusters, other repos,
credentials) out of it. How the production runner is deployed and operated is documented in the
private `komodoro_infra` repo (`k8s/phantomci`, its `AGENTS.md`) and in the private
infrastructure atlas.

Operational notes that apply anywhere:
- A runner restart interrupts every repo's builds in progress.
- A fresh runner re-clones every repo and builds nothing until a new commit lands on a watched branch
  (push an empty commit to re-trigger).
