#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

FROM debian:bookworm

RUN apt-get update && apt-get install -y \
    xz-utils \
    curl git make \
    sudo man vim rsync procps \
    bash-completion \
    && rm -rf /var/lib/apt/lists/*

ARG UID
ARG GID

RUN groupadd -f -g $GID x && useradd -u $UID -g $GID -G sudo -m -p x x
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers # for convenience

RUN mkdir -m 0755 /nix && chown x:x /nix

USER x

WORKDIR /home/x

# Convenient shell completion and aliases
RUN mkdir -p .bash_completion.d && \
    curl -sSL -o .bash_completion.d/nix \
        https://raw.githubusercontent.com/hedning/nix-bash-completions/e6db3081fe1f221470a26e345a96855e5f09ddec/_nix

COPY bashrc_extra .bashrc_extra
RUN echo ". ~/.bashrc_extra" >> .bashrc

WORKDIR /tmp

ARG STATEFUL

COPY setup.sh .

RUN if [ "$STATEFUL" = "1" ]; then bash setup.sh; fi

COPY nix.conf /etc/nix/

ENV NIX_BUILD_SHELL=bash

WORKDIR /work
