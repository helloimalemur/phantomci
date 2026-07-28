FROM rust:bookworm

# Use the key at runtime by default, disable host key checking for automation
ENV GIT_SSH_COMMAND="ssh -i /root/.ssh/id_ed25519 -o StrictHostKeyChecking=no"

# Ensure needed tools exist at runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates docker.io git openssh-client \
    && rm -rf /var/lib/apt/lists/*

# Prepare SSH directory and generate a fallback key (can be overridden below)
RUN mkdir -p /root/.ssh \
    && chmod 700 /root/.ssh \
    && ssh-keygen -t ed25519 -f /root/.ssh/id_ed25519 -P ""

WORKDIR /phantom_ci
COPY . .

# Optional: placeholder keys copied into /root/keys (used only if non-empty)
COPY docker/key /root/keys/id_ed25519
COPY docker/key.pub /root/keys/id_ed25519.pub

# Use provided keys only if they are non-empty.
RUN if [ -s /root/keys/id_ed25519 ]; then cp /root/keys/id_ed25519 /root/.ssh/id_ed25519; fi \
 && if [ -s /root/keys/id_ed25519.pub ]; then cp /root/keys/id_ed25519.pub /root/.ssh/id_ed25519.pub; fi \
 && chmod 600 /root/.ssh/id_ed25519 \
 && [ -f /root/.ssh/id_ed25519.pub ] && chmod 644 /root/.ssh/id_ed25519.pub || true \
 && chown -R root:root /root/.ssh

# Wait for the DinD sidecar when this image is used by Compose or Kubernetes.
COPY docker/entrypoint.sh /usr/local/bin/phantomci-entrypoint
RUN chmod 755 /usr/local/bin/phantomci-entrypoint

# Install the binary into PATH
RUN cargo install --locked --path ./

ENTRYPOINT ["/usr/local/bin/phantomci-entrypoint"]
CMD ["phantom_ci"]