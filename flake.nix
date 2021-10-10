{
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.mkarm64image = {
    url = "github:rosehuds/mkarm64image";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { self, nixpkgs, flake-utils, mkarm64image }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      triple = (nixpkgs.lib.systems.elaborate system).config;
    in
      {
        devShell = pkgs.pkgsCross.aarch64-multiplatform.mkShell {
          depsBuildBuild = with pkgs; [ cacert rustup gcc gdb mkarm64image.packages.${system}.mkarm64image qemu ];
          shellHook = ''
            rustup component add rust-src rust-analyzer-preview
            export PATH=$PATH:~/.rustup/toolchains/${pkgs.lib.readFile ./rust-toolchain}-${triple}/bin/
          '';
        };
      }
  );
}
