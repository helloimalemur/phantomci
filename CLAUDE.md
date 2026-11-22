@AGENTS.md


## Dependency CVE check (cve-watch)
After any code change in this repo, and before committing, run the `cve-watch` subagent
(`.claude/agents/cve-watch.md`). If it reports `FIX REQUIRED`, surface that to the user before finishing.
Offline fallback: `python3 -I .claude/scripts/osv-scan.py .`
