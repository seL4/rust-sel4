#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

FROM debian:bookworm

RUN apt-get update -q && apt-get install -y --no-install-recommends \
    bash-completion \
    build-essential \
    ca-certificates \
    curl \
    make \
    man \
    procps \
    python3-pip \
    sudo \
    vim \
    && rm -rf /var/lib/apt/lists/*

ARG UID
ARG GID

RUN groupadd -f -g $GID x && useradd -u $UID -g $GID -G sudo -m -p x x
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers # for convenience

USER x

# Optimize by matching rust-toolchain.toml
ENV DEFAULT_TOOLCHAIN=nightly-2023-08-02

RUN curl -sSf https://sh.rustup.rs | \
        bash -s -- -y --no-modify-path --default-toolchain $DEFAULT_TOOLCHAIN

ENV PATH=/home/x/.cargo/bin:$PATH

RUN cargo install --locked kani-verifier && cargo kani setup

WORKDIR /work
