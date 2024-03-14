#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

FROM debian:bookworm

RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    python3-pip \
    sudo \
    man \
    procps \
    vim \
    bash-completion \
    && rm -rf /var/lib/apt/lists/*

RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

ARG UID
ARG GID

RUN set -eux; \
    if ! [ $UID = 0 -a $GID = 0 ]; then \
        ! getent passwd $UID; \
        if ! getent group $GID; then \
            groupadd -g $GID x; \
        fi; \
        useradd -u $UID -g $GID -G sudo -m -p x x; \
    fi

ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo

RUN set -eux; \
    dirs="$RUSTUP_HOME $CARGO_HOME"; \
    mkdir -p -m 0755 $dirs; \
    chown $UID:$GID $dirs

USER $UID

# Optimize by matching rust-toolchain.toml
ENV DEFAULT_TOOLCHAIN=nightly-2024-03-09

RUN curl -sSf -L https://sh.rustup.rs | \
        bash -s -- -y --no-modify-path --default-toolchain $DEFAULT_TOOLCHAIN

ENV PATH=$CARGO_HOME/bin:$PATH

RUN cargo install --locked kani-verifier && cargo kani setup

WORKDIR /work
