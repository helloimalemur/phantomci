#!/usr/bin/env python3
"""Scan lockfiles under a directory against OSV.dev. Usage: osv_scan.py <root> [--json]"""
import json, os, re, sys, urllib.request

SKIP = {"node_modules", "target", ".git", "vendor", "dist"}

def cargo(p):
    txt = open(p).read()
    for blk in txt.split("[[package]]")[1:]:
        n = re.search(r'^name = "([^"]+)"', blk, re.M); v = re.search(r'^version = "([^"]+)"', blk, re.M)
        if n and v and "source = " in blk:  # skip workspace crates
            yield "crates.io", n.group(1), v.group(1)

def gosum(p):
    seen = set()
    for line in open(p):
        parts = line.split()
        if len(parts) >= 2 and not parts[1].endswith("/go.mod"):
            k = (parts[0], parts[1].lstrip("v").split("+")[0])
            if k not in seen: seen.add(k); yield "Go", k[0], k[1]

def npm(p):
    d = json.load(open(p))
    for path, meta in (d.get("packages") or {}).items():
        if path and "version" in meta and not meta.get("link"):
            yield "npm", path.split("node_modules/")[-1], meta["version"]

def pip(p):
    for line in open(p):
        m = re.match(r"\s*([A-Za-z0-9_.\-]+)\s*==\s*([^\s;#]+)", line)
        if m: yield "PyPI", m.group(1), m.group(2)

PARSERS = {"Cargo.lock": cargo, "go.sum": gosum, "package-lock.json": npm}

def collect(root):
    out = []
    for dp, dns, fns in os.walk(root):
        dns[:] = [d for d in dns if d not in SKIP]
        for f in fns:
            fn = PARSERS.get(f) or (pip if re.match(r"requirements.*\.txt$", f) else None)
            if fn:
                p = os.path.join(dp, f)
                try:
                    for eco, n, v in fn(p): out.append((os.path.relpath(p, root), eco, n, v))
                except Exception as e: print(f"warn: {p}: {e}", file=sys.stderr)
    return out

def post(url, body):
    req = urllib.request.Request(url, json.dumps(body).encode(), {"Content-Type": "application/json"})
    return json.load(urllib.request.urlopen(req, timeout=60))

def get(url):
    return json.load(urllib.request.urlopen(url, timeout=60))

def main():
    root = sys.argv[1]; deps = collect(root)
    uniq = sorted({(e, n, v) for _, e, n, v in deps})
    hits = {}
    for i in range(0, len(uniq), 900):
        chunk = uniq[i:i+900]
        r = post("https://api.osv.dev/v1/querybatch",
                 {"queries": [{"package": {"ecosystem": e, "name": n}, "version": v} for e, n, v in chunk]})
        for q, res in zip(chunk, r["results"]):
            ids = [x["id"] for x in res.get("vulns", [])]
            if ids: hits[q] = ids
    cache, rows = {}, []
    for (e, n, v), ids in hits.items():
        for vid in ids:
            if vid not in cache:
                try: cache[vid] = get(f"https://api.osv.dev/v1/vulns/{vid}")
                except Exception: cache[vid] = {"id": vid}
            d = cache[vid]
            if d.get("withdrawn"): continue
            fixed = sorted({ev["fixed"] for a in d.get("affected", []) if a.get("package", {}).get("name") == n
                            for r_ in a.get("ranges", []) for ev in r_.get("events", []) if "fixed" in ev})
            sev = (d.get("database_specific") or {}).get("severity") or \
                  next((s.get("score") for s in d.get("severity", [])), "")
            files = sorted({f for f, e2, n2, v2 in deps if (e2, n2, v2) == (e, n, v)})
            rows.append({"eco": e, "pkg": n, "ver": v, "id": vid,
                         "aliases": [a for a in d.get("aliases", []) if a.startswith("CVE-")],
                         "summary": d.get("summary", "")[:140], "severity": sev,
                         "published": d.get("published", "")[:10], "fixed": fixed,
                         "unmaintained": "unmaintained" in json.dumps(d.get("database_specific", {})).lower()
                                         or "unmaintained" in d.get("summary", "").lower(),
                         "files": files})
    if "--json" in sys.argv: print(json.dumps(rows, indent=1)); return
    print(f"{len(deps)} dep entries, {len(uniq)} unique, {len(rows)} advisories")
    for r in sorted(rows, key=lambda r: (r["files"][0], r["pkg"])):
        print(f"{r['pkg']}@{r['ver']} [{r['eco']}] {r['id']} {','.join(r['aliases'])} sev={r['severity']} "
              f"fix={','.join(r['fixed']) or 'none'} pub={r['published']}\n    {r['summary']}\n    in: {', '.join(r['files'])}")

if __name__ == "__main__": main()
