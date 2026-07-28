#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROJECT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
cd "$PROJECT_DIR"

# Keep compatibility with the old deployment host, while allowing Kubernetes
# to provide credentials through /root/.docker/config.json or environment.
REGISTRY_FILE="${PHANTOMCI_REGISTRY_FILE:-/etc/vultr/registry}"
if [[ -f "$REGISTRY_FILE" ]]; then
  # shellcheck disable=SC1090
  source "$REGISTRY_FILE"
fi

IMAGE="${PHANTOMCI_IMAGE:-phantomci:latest}"
REGISTRY_HOST="${PHANTOMCI_REGISTRY_HOST:-${VULTR_REGISTRY_HOST:-}}"

if [[ -n "${PHANTOMCI_PUSH_IMAGE:-}" ]]; then
  PUSH_IMAGE="$PHANTOMCI_PUSH_IMAGE"
elif [[ -n "$REGISTRY_HOST" ]]; then
  PUSH_IMAGE="${REGISTRY_HOST%/}/$IMAGE"
else
  PUSH_IMAGE="$IMAGE"
fi

REGISTRY_USER="${PHANTOMCI_REGISTRY_USER:-${VULTR_REGISTRY_USER:-}}"
REGISTRY_PASSWORD="${PHANTOMCI_REGISTRY_PASSWORD:-${VULTR_REGISTRY_PASS:-}}"
if [[ -n "$REGISTRY_USER" && -n "$REGISTRY_PASSWORD" && -n "$REGISTRY_HOST" ]]; then
  printf '%s\n' "$REGISTRY_PASSWORD" |
    docker login "$REGISTRY_HOST" \
      --username="$REGISTRY_USER" \
      --password-stdin
fi

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo "Docker image '$IMAGE' is missing; run build_docker_images.sh first" >&2
  exit 1
fi

if [[ "$PUSH_IMAGE" != "$IMAGE" ]]; then
  docker tag "$IMAGE" "$PUSH_IMAGE"
fi

docker push "$PUSH_IMAGE"