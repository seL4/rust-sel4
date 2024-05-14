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
    bash-completion \
    man \
    sudo \
    && rm -rf /var/lib/apt/lists/*

ARG UID
ARG GID

RUN set -eux; \
    if [ $UID -eq 0 ]; then \
        [ $GID -eq 0 ]; \
    else \
        ! getent passwd $UID; \
        if ! getent group $GID; then \
            groupadd --gid $GID x; \
        fi; \
        useradd --uid $UID --gid $GID --groups sudo --create-home x; \
    fi;

RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

USER $UID

ARG TOOLCHAIN

RUN set -eux; \
    if [ $UID -ne 0 ]; then \
        curl -sSf https://sh.rustup.rs | \
            bash -s -- -y --no-modify-path --default-toolchain $TOOLCHAIN; \
    fi;

ENV PATH=/home/x/.cargo/bin:/root/.cargo/bin:$PATH

RUN cargo install --locked kani-verifier@0.51.0 && cargo kani setup

WORKDIR /work/hacking/kani
