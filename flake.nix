{
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.mkarm64image = {
    url = "github:rosehuds/mkarm64image";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { self, nixpkgs, flake-utils, mkarm64image }: flake-utils.lib.eachDefaultSystem (system:
    let pkgs = nixpkgs.legacyPackages.${system}; in
    {
      devShell = pkgs.pkgsCross.aarch64-multiplatform.mkShell {
        depsBuildBuild = with pkgs; [ cacert rustup gcc gdb mkarm64image qemu ];
        preUnpack = "rustup component add rust-src rust-analyzer";
      };
    }
  );
}
