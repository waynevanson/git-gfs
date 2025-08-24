{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    naersk,
    fenix,
    flake-utils,
    nixpkgs,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [fenix.overlays.default];
        };

        # utility functions
        createPkgConfigPath = deps: pkgs.lib.strings.concatStringsSep ":" (builtins.map (a: "${a}/lib/pkgconfig") deps);
        createBindgenExtraClangArgs = deps: (builtins.map (a: ''-I"${a}/include"'') deps);
        createRustFlags = deps: builtins.map (a: ''-L ${a}/lib'') deps;

        rust' = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-JE+aoEa897IBKa03oVUOOnW+sbyUgXGrhkwzWFzCnnI=";
        };

        naersk' = pkgs.callPackage naersk {
          inherit pkgs;
          cargo = rust';
          rustc = rust';
        };

        codebase' = naersk'.buildPackage {
          name = "workspace";
          src = ./.;
          cargoClippyOptions = _: ["-A clippy::all"];
        };

        git-gfs = naersk'.buildPackage {
          name = "git-gfs";
          version = "0.0.0";
          src = ./.;
        };

        nativeBuildInputs = with pkgs; [
          cargo-watch
          cargo-tarpaulin
          clang
          #codebase'
          git
          git-subrepo
          llvmPackages.bintools
          openssl
          openssl.dev
          pkg-config
          rust'
          rust-analyzer-nightly
        ];
        buildInputs = with pkgs; [
          git
          openssl
          pkg-config
          #git-gfs
        ];

        environment = {
          LIBCLANG_PATH = pkgs.lib.makeLibraryPath [
            pkgs.llvmPackages_latest.libclang.lib
          ];
          RUSTFLAGS = createRustFlags [];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (nativeBuildInputs ++ buildInputs);
          BINGEN_EXTRA_CLANG_ARGS =
            createBindgenExtraClangArgs (with pkgs; [glibc.dev])
            ++ [
              ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
              ''-I"${pkgs.glib.dev}/include/glib-2.0"''
              ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
            ];
          PKG_CONFIG_PATH = createPkgConfigPath buildInputs;
          # Ensure we use the version in the flake, not what `git2` crate prefers.
          # https://github.com/rust-lang/git2-rs?tab=readme-ov-file#version-of-libgit2
          LIBGIT2_NO_VENDOR = 1;
        };

        shellHook = ''
          export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
          export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
        '';
        common = environment // {inherit nativeBuildInputs buildInputs shellHook;};


      in {
        packages.git-gfs = git-gfs;
        devShells.default = pkgs.mkShell common;
      }
    );
}
