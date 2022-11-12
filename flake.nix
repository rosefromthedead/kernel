{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-21.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.mkarm64image = {
    url = "github:rosehuds/mkarm64image";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { self, nixpkgs, flake-utils, mkarm64image }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      build-triple = (nixpkgs.lib.systems.elaborate system).config;
      channel = (nixpkgs.lib.trivial.importTOML ./rust-toolchain.toml).toolchain.channel;
    in
      {
        devShell = pkgs.pkgsCross.aarch64-multiplatform.mkShell {
          depsBuildBuild = with pkgs; [ cacert rustup gdb mkarm64image.packages.${system}.mkarm64image qemu ];
          shellHook = ''
            export PATH=$PATH:~/.rustup/toolchains/${channel}-${build-triple}/bin/
          '';
        };
      }
  );
}
