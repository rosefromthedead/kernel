let
  aarch64-dev-overlay = import (builtins.fetchTarball https://github.com/rosehuds/aarch64-dev-overlay/archive/a9afe06a76fd4f42d3467fa16d0df083b1ddf107.tar.gz);
  pkgs = import <nixpkgs> { overlays = [ aarch64-dev-overlay ]; };
  pkgsCross = import <nixpkgs> {
    crossSystem = {
      config = "aarch64-unknown-linux-gnu";
    };
  };
in
pkgsCross.mkShell {
  depsBuildBuild = with pkgs; [ cacert rustup gcc gdb mkarm64image qemu ];
  preUnpack = "rustup component add rust-src";
}

