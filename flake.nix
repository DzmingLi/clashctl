{
  description = "clashctl - CLI & TUI for Clash RESTful API";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { nixpkgs, ... }:
  let
    forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ];
  in {
    homeManagerModules.default = import ./nix/hm-module.nix;

    packages = forAllSystems (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "clashctl";
          version = "0.3.6";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };
      }
    );
  };
}
