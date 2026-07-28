# PhantomCI on Kubernetes

The deployment runs PhantomCI beside a privileged `docker:dind` sidecar. The
runner uses the sidecar through `tcp://127.0.0.1:2375`, so workflow steps can
run `docker build` and `docker push` without access to the Kubernetes node's
Docker socket.

## Build and push the runner image

Replace the registry and image name with a registry your cluster can pull from:

```bash
export IMAGE=registry.example.com/your-org/phantomci:0.2.3
docker build -f docker/Dockerfile -t "$IMAGE" .
docker push "$IMAGE"
```

Set the same value in `deployment.yaml`. If the registry is private, create an
image-pull secret and uncomment `imagePullSecrets` in that manifest.

## Create runtime secrets

The SSH key is used to clone configured repositories:

```bash
kubectl -n phantomci create secret generic phantomci-ssh \
  --from-file=id_ed25519="$HOME/.ssh/id_ed25519"
```

The optional registry secret supplies `/root/.docker/config.json` to workflow
commands, allowing non-interactive pushes:

```bash
kubectl -n phantomci create secret docker-registry phantomci-registry \
  --docker-server=registry.example.com \
  --docker-username="$REGISTRY_USER" \
  --docker-password="$REGISTRY_PASSWORD"
```

## Deploy

Update the repository and image placeholders in `configmap.yaml` and
`deployment.yaml`, then apply the examples:

```bash
kubectl apply -f examples/k8s/namespace.yaml
kubectl apply -f examples/k8s/pvc.yaml
kubectl apply -f examples/k8s/configmap.yaml
kubectl apply -f examples/k8s/deployment.yaml
```

The DinD sidecar requires a Kubernetes policy that permits privileged
containers. The deployment uses an `emptyDir` for Docker layer data; use a
dedicated persistent volume if retaining build cache across pod recreation is
important.