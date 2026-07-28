#!/bin/bash
set -eo pipefail
DOCKER_COMPOSE="docker/docker-compose.yaml"
## build and upload image
#docker-compose --env-file /etc/vultr/registry -f $DOCKER_COMPOSE build
docker-compose -f $DOCKER_COMPOSE build
