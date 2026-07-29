#!/bin/bash
set -eo pipefail

DEPLOYMENT_MANIFESTS=(kube/*deployment*)
NAMESPACE_MANIFEST="kube/namespace.yaml"
DOCKER_COMPOSE="docker/docker-compose.yaml"
NAMESPACE=$(yq -r '.metadata.name' "$NAMESPACE_MANIFEST")

KUBECTL=(kubectl)
if [ -n "${KUBECONFIG:-}" ]; then
  KUBECTL+=(--kubeconfig "$KUBECONFIG")
elif [ -f "/root/.kube/vke-komodoro" ]; then
  KUBECTL+=(--kubeconfig "/root/.kube/vke-komodoro")
elif [ -n "${KUBERNETES_SERVICE_HOST:-}" ]; then
  SERVICE_ACCOUNT_DIR="/var/run/secrets/kubernetes.io/serviceaccount"
  KUBECTL+=(
    --server="https://${KUBERNETES_SERVICE_HOST}:${KUBERNETES_SERVICE_PORT_HTTPS:-443}"
    --certificate-authority="${SERVICE_ACCOUNT_DIR}/ca.crt"
    --token="$(<"${SERVICE_ACCOUNT_DIR}/token")"
  )
fi

for DEPLOYMENT_MANIFEST in $DEPLOYMENT_MANIFESTS; do
  DEPLOYMENT=$(yq -r '.metadata.name' "$DEPLOYMENT_MANIFEST")
  echo "Restarting deployment: $DEPLOYMENT"
  "${KUBECTL[@]}" rollout restart deployment "$DEPLOYMENT" -n "$NAMESPACE"
done
