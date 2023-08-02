set -e

if [ ! -f /nix/.installed ]; then
    curl -L https://nixos.org/nix/install | \
        sh -s -- --yes --no-channel-add --no-modify-profile
    touch /nix/.installed
fi
