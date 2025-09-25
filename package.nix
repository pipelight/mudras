{
  pkgs ? import <nixpkgs> {},
  lib,
  ...
}:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "mudras";
  version = (builtins.fromTOML (lib.readFile ./Cargo.toml)).package.version;

  src = ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
    # outputHashes = {
    #   "tappers-0.4.2" = "sha256-kx/gLngL7+fH5JmJTVTGawyNdRde59dbFdrzermy/CE=";
    # };
  };

  # disable tests
  checkType = "debug";
  doCheck = false;

  nativeBuildInputs = with pkgs; [
    installShellFiles
    pkg-config

    # llvmPackages.clang
    # clang
  ];
  buildInputs = with pkgs; [
    openssl
    pkg-config

    # libs
    udev

    # rust vmm uses latest stable and oxalica tend to lag behind.break
    # so we temporary force use of beta.
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
  ];
}
