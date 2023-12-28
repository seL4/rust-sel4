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

# # Install bash completion for Nix
# RUN set -eux; \
#     d=/usr/local/share/bash-completion/completions; \
#     mkdir -p $d; \
#     cd $d; \
#     curl -sSf -L -o _nix \
#         https://raw.githubusercontent.com/hedning/nix-bash-completions/v0.6.8/_nix; \
#     commands=$(bash -c ' \
#       function complete() { shift 2; echo "$@"; }; \
#       shopt -s extglob; \
#       source _nix; \
#     '); \
#     for c in $commands; do \
#       ln -s _nix $c; \
#     done

RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers # for convenience

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

    # NOTE would prefer this, but "staff" on MacOS is "dialout" on debian
    # if getent group $GID; then \
    #     group_name="$(getent group $GID | cut -d: -f1)"; \
    #     [ "$group_name" = "users" -o "$group_name" = "staff" ]; \
    # else \
    #     groupadd -g $GID x; \
    # fi; \

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

# # Install bash completion for Nix
RUN set -eux; \
    export USER=$(whoami); \
    . ~/.nix-profile/etc/profile.d/nix.sh; \
    nix-channel --add https://nixos.org/channels/nixos-23.11 nixpkgs; \
    nix-channel --update; \
    nix-env -i nix-bash-completions; \
    nix-channel --remove nixpkgs; \
    nix-collect-garbage -d; \
    nix-store --optimize

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

WORKDIR /work
