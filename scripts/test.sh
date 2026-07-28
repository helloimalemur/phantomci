#!/usr/bin/env bash
set -euo pipefail

compose=(docker compose -f docker/docker-compose.yaml)
trap '${compose[@]} down --volumes --remove-orphans' EXIT

"${compose[@]}" config --quiet
images=$("${compose[@]}" config --images)
grep -Fxq 'phantomci:latest' <<<"$images"
"${compose[@]}" up --build --detach
"${compose[@]}" exec --user root --workdir / phantom_ci docker info