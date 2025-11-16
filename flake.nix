{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";

    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    devenv,
    flake-utils,
    nixpkgs,
    treefmt-nix,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };

        treefmtConfig = {...}: {
          projectRootFile = "flake.nix";
          programs = {
            alejandra.enable = true;
            rustfmt.enable = true;
            yapf.enable = true;
          };
        };

        treefmtEval = treefmt-nix.lib.evalModule pkgs (treefmtConfig {inherit pkgs;});
      in {
        formatter = treefmtEval.config.build.wrapper;

        devShells.default = devenv.lib.mkShell {
          inherit inputs pkgs;

          modules = [
            ({pkgs, ...}: {
              languages = {
                nix.enable = true;
                rust = {
                  enable = true;
                  channel = "nightly";
                };
              };

              packages = with pkgs; [
                alejandra
                bacon
                cargo-fuzz
                cargo-release
                clippy
                presenterm
                rustfmt
                rusty-man
                yapf
              ];

              enterShell = ''
                export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$NIX_LD_LIBRARY_PATH
                export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/run/opengl-driver/lib

                export CUDA_ROOT="${pkgs.cudaPackages.cudatoolkit}"
              '';
            })
          ];
        };
      }
    );
}
