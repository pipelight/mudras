{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs.buildPackages; [
    openssl
    pkg-config

    # libs
    udev

    libinput
    libxkbcommon

    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
    # rust-analyzer
  ];
}
