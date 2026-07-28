#!/bin/bash
set -eo pipefail

source /etc/vultr/registry
echo $VULTR_REGISTRY_PASS | docker login "$VULTR_REGISTRY_HOST" --username="$VULTR_REGISTRY_USER" --password-stdin
TAG="latest"
DOCKER_COMPOSE="docker/docker-compose.yaml"

readarray -t IMAGES < <(
  yq -r '.services[].image' "$DOCKER_COMPOSE" \
    | sort -u
)

for IMAGE in "${IMAGES[@]}"; do
  set +e
  if docker image inspect "$IMAGE" >/dev/null 2>&1; then
    echo "$IMAGE exists"
    echo "IMAGE: $IMAGE"
    IMAGE_GUID=$(curl -s -H "Authorization: Bearer ${VULTR_API_KEY}" "https://api.vultr.com/v2/registry/${VULTR_REGISTRY_ID}/repositories" | jq -r --arg img "$IMAGE" '.repositories[] | select(.name == ("komodoro/" + $img)) | .image')
    echo "deleting IMAGE_GUID: $IMAGE_GUID"
    curl -i -X DELETE -H "Authorization: Bearer ${VULTR_API_KEY}" "https://api.vultr.com/v2/registry/${VULTR_REGISTRY_ID}/repository/$IMAGE_GUID"
    set -e
    sleep 3s
    docker tag "$IMAGE" "$VULTR_REGISTRY_HOST/$IMAGE:$TAG"
    docker push "$VULTR_REGISTRY_HOST/$IMAGE:$TAG"
  else
    echo "$IMAGE missing"
  fi
done