{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    fenix,
    flake-utils,
    nixpkgs,
    treefmt-nix,
    ...
  }:
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

        devShells.default = pkgs.mkShell {
          nativeBuildInputs =
          with pkgs; [
              # cudaPackages.cuda_nvcc
              # cudaPackages.cudatoolkit
              pkg-config
              stdenv.cc.cc.lib
          ] ++ 
            (with fenix.packages.${system}.latest;
            [
              cargo
              rust-analyzer
              rustc
            ]);

          buildInputs = with pkgs; [
            poetry
            python312
            python312Packages.virtualenv
          ];

          packages =
          with pkgs; [
            gdb
            python312Packages.debugpy
            rusty-man
            yapf
          ] ++ 
            (with fenix.packages.${system}.latest;
            [
              bacon
              cargo-fuzz
              cargo-info
              cargo-release
              clippy
              rustfmt
            ]);

          shellHook = 
          let
          ldLibs = nixpkgs.lib.strings.makeLibraryPath [
            pkgs.stdenv.cc.cc.lib
          ];
          in
          ''
            export LD_LIBRARY_PATH=${ldLibs}
            # export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.libGL}/lib
            # export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.glib.out}/lib
            # export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/run/opengl-driver/lib

            export CUDA_ROOT="${pkgs.cudaPackages.cudatoolkit}"
          '';
        };
      }
    );
}
