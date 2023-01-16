set -e

if [ ! -f /nix/.installed ]; then
    echo "Installing Nix..."
    bash /install-nix.sh
    touch /nix/.installed
    echo "Done"
fi
