FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    curl git make \
    sudo man vim rsync procps \
    bash-completion \
    && rm -rf /var/lib/apt/lists/*

ARG UID
ARG GID
ARG STATEFUL

RUN groupadd -f -g $GID x && useradd -u $UID -g $GID -G sudo -m -p x x
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers # for convenience

RUN mkdir -m 0755 /nix && chown x:x /nix

COPY install-nix.sh /
COPY setup.sh /

USER x

WORKDIR /home/x

RUN if [ "$STATEFUL" = "1" ]; then bash /setup.sh; fi

COPY nix.conf /etc/nix/

ENV PATH="/nix/env/bin:${PATH}"
ENV MANPATH="/nix/env/share/man:${MANPATH}"
ENV NIX_SSL_CERT_FILE=/nix/env/etc/ssl/certs/ca-bundle.crt
ENV NIX_BUILD_SHELL=bash

# for persistent Nix git cache
ENV XDG_CACHE_HOME=/nix/cache

# convenient shell completion and aliases

RUN mkdir -p .bash_completion.d && \
    curl -SL -o .bash_completion.d/nix \
        https://raw.githubusercontent.com/hedning/nix-bash-completions/e6db3081fe1f221470a26e345a96855e5f09ddec/_nix

COPY bashrc_extra .bashrc_extra
RUN echo ". ~/.bashrc_extra" >> .bashrc

WORKDIR /work
