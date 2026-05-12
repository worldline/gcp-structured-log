let
  system = "x86_64-linux";
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-25.05";
  pkgs = import nixpkgs { config = {}; overlays = []; };

in

pkgs.mkShellNoCC {
  packages = with pkgs; [
    pkgs.gcc
    pkgs.llvmPackages_19.libcxxClang
    pkgs.lld_19
    pkgs.pkg-config
    pkgs.rustup
  ];

  shellHook = ''
    export RUSTUP_HOME="$HOME/.rustup"
    export CARGO_HOME="$HOME/.cargo"
    export PATH="$CARGO_HOME/bin:$PATH"

    if [ ! -d "$RUSTUP_HOME" ]; then
      echo "Initializing rustup..."
      rustup-init -y
      source $RUSTUP_HOME/env
    fi

    rustup default stable
    rustup component add rust-analyzer clippy rustfmt
  '';
}
