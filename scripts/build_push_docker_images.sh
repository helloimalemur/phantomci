#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

"$SCRIPT_DIR/build_docker_images.sh"
"$SCRIPT_DIR/push_docker_images.sh"