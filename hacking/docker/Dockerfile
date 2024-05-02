#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

FROM debian:bookworm

RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    xz-utils \
    make \
    git \
    # general utilities
    sudo \
    man \
    procps \
    rsync \
    file \
    less \
    vim \
    bash-completion \
    && rm -rf /var/lib/apt/lists/*

RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

ARG UID
ARG GID

RUN set -eux; \
    if [ $UID -eq 0 ]; then \
        if [ $GID -ne 0 ]; then \
            echo "error: \$UID == 0 but \$GID != 0" >&2; \
            exit 1; \
        fi; \
    else \
        if getent passwd $UID; then \
            echo "error: \$UID $UID already exists" >&2; \
            exit 1; \
        fi; \
        if ! getent group $GID; then \
            groupadd --gid $GID x; \
        fi; \
        useradd --uid $UID --gid $GID --groups sudo --create-home x; \
    fi;

RUN set -eux; \
    mkdir -p -m 0755 /nix; \
    chown $UID:$GID /nix;

USER $UID

RUN curl -sSf -L https://sh.rustup.rs | \
        bash -s -- -y --no-modify-path --default-toolchain none

# Account for both possible cases
ENV PATH=/root/x/.cargo/bin:/home/x/.cargo/bin:$PATH

RUN curl -sSf -L https://nixos.org/nix/install | \
        bash -s -- --yes --no-modify-profile --no-channel-add

# Install bash completion for Nix
RUN set -eux; \
    export USER=$(whoami); \
    . ~/.nix-profile/etc/profile.d/nix.sh; \
    nix-channel --add https://nixos.org/channels/nixos-23.11 nixpkgs; \
    nix-channel --update; \
    nix-env -i nix-bash-completions; \
    nix-channel --remove nixpkgs; \
    nix-collect-garbage -d;

# Add gcroot for store paths required by this image so that fresh images can use persistent /nix
# volumes.
RUN set -eux; \
    export USER=$(whoami); \
    . ~/.nix-profile/etc/profile.d/nix.sh; \
    nix-store -r \
        --add-root /nix/var/nix/gcroots-for-image/profile \
        $(readlink --canonicalize-existing ~/.nix-profile);

COPY nix.conf /etc/nix/

ENV NIX_BUILD_SHELL=bash

RUN ( \
        echo 'export USER=$(whoami)'; \
        echo '. ~/.nix-profile/etc/profile.d/nix.sh'; \
    ) >> ~/.bashrc

RUN ( \
        echo 'set show-mode-in-prompt on'; \
        echo 'set editing-mode vi'; \
    ) >> ~/.inputrc

VOLUME /nix

WORKDIR /work
