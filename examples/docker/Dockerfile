FROM rust:bookworm

# Use the key at runtime by default, disable host key checking for automation
ENV GIT_SSH_COMMAND="ssh -i /root/.ssh/id_rsa -o StrictHostKeyChecking=no"

# Ensure needed tools exist at runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends git openssh-client ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Prepare SSH directory and generate a fallback key (can be overridden below)
RUN mkdir -p /root/.ssh \
    && chmod 700 /root/.ssh \
    && ssh-keygen -t rsa -b 4096 -f /root/.ssh/id_rsa -P ""

WORKDIR /phantom_ci
COPY . .

# Optional: placeholder keys copied into /root/keys (used only if non-empty)
COPY docker/key /root/keys/id_rsa
COPY docker/key.pub /root/keys/id_rsa.pub

# Use provided keys only if they are non-empty (fix inverted logic)
RUN if (( $(stat -c%s /root/keys/id_rsa) > 5 )); then cp /root/keys/id_rsa /root/.ssh/id_rsa; fi \
 && if (( $(stat -c%s /root/keys/id_rsa.pub) > 5 )); then cp /root/keys/id_rsa.pub /root/.ssh/id_rsa.pub; fi \
 && chmod 600 /root/.ssh/id_rsa \
 && [ -f /root/.ssh/id_rsa.pub ] && chmod 644 /root/.ssh/id_rsa.pub || true \
 && chown -R root:root /root/.ssh

# Install the binary into PATH
RUN cargo install --path ./

CMD ["phantom_ci"]