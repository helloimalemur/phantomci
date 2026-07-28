#!/bin/bash
set -eo pipefail

DEPLOYMENT_MANIFESTS=(kube/*deployment*)
NAMESPACE_MANIFEST="kube/namespace.yaml"
DOCKER_COMPOSE="docker/docker-compose.yaml"
KUBECONFIG="/root/.kube/vke-komodoro"
NAMESPACE=$(yq -r '.metadata.name' "$NAMESPACE_MANIFEST")

for DEPLOYMENT_MANIFEST in $DEPLOYMENT_MANIFESTS; do
  DEPLOYMENT=$(yq -r '.metadata.name' "$DEPLOYMENT_MANIFEST")
  echo "Restarting deployment: $DEPLOYMENT"
  kubectl --kubeconfig "$KUBECONFIG" rollout restart deployment "$DEPLOYMENT" -n "$NAMESPACE"
done
