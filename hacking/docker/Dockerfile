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

RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers # for convenience

ARG UID
ARG GID

RUN set -eux; \
    if ! [ $UID = 0 -a $GID = 0 ]; then \
        ! getent passwd $UID; \
        # NOTE
        # This is a bit of a hack. For example, GID for "staff" on MacOS is
        # "dialout" on Debian. In an ideal world, we'd ensure that an
        # already-occupied GID corresponds to either "users" or "staff" on
        # Debian. 
        if ! getent group $GID; then \
            groupadd -g $GID x; \
        fi; \
        useradd -u $UID -g $GID -G sudo -m -p x x; \
    fi

# So that they don't depend on $HOME
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo

RUN set -eux; \
    dirs="/nix $RUSTUP_HOME $CARGO_HOME"; \
    mkdir -p -m 0755 $dirs; \
    chown $UID:$GID $dirs

USER $UID

RUN curl -sSf -L https://sh.rustup.rs | \
        bash -s -- -y --no-modify-path --default-toolchain none

ENV PATH=$CARGO_HOME/bin:$PATH

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
    nix-collect-garbage -d

# Add gcroot for store paths required by this image so that fresh images can use
# persistent /nix volumes.
RUN set -eux; \
    export USER=$(whoami); \
    . ~/.nix-profile/etc/profile.d/nix.sh; \
    nix-store -r \
        --add-root /nix/var/nix/gcroots-for-image/profile \
        $(readlink --canonicalize-existing ~/.nix-profile)

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
