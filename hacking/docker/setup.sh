#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

set -e

if [ ! -f /nix/.installed ]; then
    curl -L https://nixos.org/nix/install | \
        sh -s -- --yes --no-channel-add --no-modify-profile
    ln -s $(readlink --canonicalize $HOME/.nix-profile) /nix/var/nix/the-profile
    rm -r $HOME/.nix-profile $HOME/.nix-defexpr $HOME/.local/state/nix
    touch /nix/.installed
fi
