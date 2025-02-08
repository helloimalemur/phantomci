FROM rust:bookworm
ENV GIT_SSH_COMMAND="ssh -i /root/.ssh/id_rsa -o StrictHostKeyChecking=no"
RUN ssh-keygen -t rsa -b 4096 -f /root/.ssh/id_rsa -P ""
WORKDIR /phantom_ci
COPY . .

COPY docker/key /root/keys/id_rsa
COPY docker/key.pub /root/keys/id_rsa.pub
RUN if (( $(stat -c%s /root/keys/id_rsa) > 5 )); then echo ""; else cp /root/keys/id_rsa /root/.ssh/id_rsa; fi
RUN if (( $(stat -c%s /root/keys/id_rsa.pub) > 5 )); then echo ""; else cp /root/keys/id_rsa.pub /root/.ssh/id_rsa.pub; fi

RUN chmod 500 /root/.ssh/id_rsa*
RUN cargo install --path ./
CMD ["phantom_ci"]