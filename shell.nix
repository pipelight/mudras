{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs.buildPackages; [
    openssl
    pkg-config

    # libs
    udev

    # cairo
    # dbus
    # libGL
    # libdisplay-info
    seatd
    libinput
    libxkbcommon
    # mesa
    libgbm
    pango
    wayland

    # rust vmm uses latest stable and oxalica tend to lag behind.break
    # so we temporary force use of beta.

    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
    rust-analyzer
  ];
  env = {
    # Force linking with libEGL and libwayland-client
    # so they can be discovered by `dlopen()`
    # CARGO_BUILD_RUSTFLAGS = toString (
    #   map (arg: "-C link-arg=" + arg) [
    #     "-Wl,--push-state,--no-as-needed"
    #     "-lEGL"
    #     "-lwayland-client"
    #     "-Wl,--pop-state"
    #   ]
    # );
  };
}
