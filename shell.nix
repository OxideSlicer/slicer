{
  pkgs ? import <nixpkgs> { },
}:
let
  overrides = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);

  runtimeLibs = with pkgs; [
    alsa-lib
    wayland
    libxkbcommon
    libx11
    libxcb
    libxcursor
    libxrandr
    libxi
    libxinerama
    vulkan-loader
    mesa
  ];

  buildLibs = with pkgs; [
    wayland-protocols
    alsa-lib
    udev
    fontconfig
    freetype
    openssl
    libpulseaudio
  ];

in
pkgs.callPackage (
  {
    stdenv,
    mkShell,
    rustup,
    rustPlatform,
    pkg-config,
  }:
  mkShell {
    nativeBuildInputs = [
      pkg-config          # <-- the fix
      rustup
      rustPlatform.bindgenHook
    ];

    buildInputs = runtimeLibs ++ buildLibs;

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;

    RUSTC_VERSION = overrides.toolchain.channel;
    shellHook = ''
      export PATH="''${CARGO_HOME:-~/.cargo}/bin":"$PATH"
      export PATH="''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-${stdenv.hostPlatform.rust.rustcTarget}/bin":"$PATH"
    '';
  }
) { }

