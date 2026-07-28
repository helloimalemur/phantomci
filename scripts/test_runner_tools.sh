#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
DOCKERFILE="$SCRIPT_DIR/../docker/Dockerfile"

grep -Eq '^FROM rust([:[:space:]]|$)' "$DOCKERFILE"

for package in \
  build-essential \
  docker-compose \
  docker.io \
  git \
  kubernetes-client \
  libssl-dev \
  openssh-client \
  pkg-config; do
  grep -Eq "(^|[[:space:]])${package}([[:space:]\\\\]|$)" "$DOCKERFILE"
done

for command in cargo docker docker-compose git scp ssh; do
  grep -Eq "${command}" "$DOCKERFILE"
done

printf '%s\n' 'Runner tool declarations passed.'