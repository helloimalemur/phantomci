#!/bin/sh
set -eu

if [ "${WAIT_FOR_DOCKER:-false}" = "true" ]; then
    attempts="${DOCKER_WAIT_ATTEMPTS:-60}"
    while ! docker info >/dev/null 2>&1; do
        if [ "$attempts" -le 0 ]; then
            echo "Docker daemon did not become ready" >&2
            exit 1
        fi
        attempts=$((attempts - 1))
        sleep 1
    done
fi

exec "$@"