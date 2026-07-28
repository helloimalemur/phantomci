#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROJECT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
cd "$PROJECT_DIR"

IMAGE="${PHANTOMCI_IMAGE:-phantomci:latest}"

docker build \
  --file docker/Dockerfile \
  --tag "$IMAGE" \
  .
