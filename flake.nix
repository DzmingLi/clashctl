{
  description = "clashctl - CLI & TUI for Clash RESTful API";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { nixpkgs, crane, ... }:
  let
    forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ];
  in {
    homeManagerModules.default = import ./nix/hm-module.nix;

    packages = forAllSystems (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
        cargoArtifacts = craneLib.buildDepsOnly {
          pname = "clashctl";
          version = "0.3.6";
          src = ./.;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };
      in {
        default = craneLib.buildPackage {
          pname = "clashctl";
          version = "0.3.6";
          inherit cargoArtifacts;
          src = ./.;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };
      }
    );
  };
}
