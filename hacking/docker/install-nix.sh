# update by copying from https://github.com/nspin/minimally-invasive-nix-installer/blob/master/dist/install-min-nix.fragment.sh

set -e

script_name="install-min-nix.sh"
script_url="https://github.com/nspin/minimally-invasive-nix-installer/raw/dist-h6yr5xax7m/dist/install-min-nix.sh"
script_sha256="81580600d2ec27b8368355be9423406bd43b74237cc74ac2e8c2078e0f7119a6"

curl -fL "$script_url" -o "$script_name"
echo "$script_sha256 $script_name" | sha256sum -c -
bash "$script_name"
rm "$script_name"
