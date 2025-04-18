{
  description = "Chicken door";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    fenix,
    flake-utils,
    advisory-db,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      inherit (pkgs) lib;

      # toolchain = fenix.packages.${system}.complete;
      toolchain = fenix.packages.${system}.stable;

      craneLib = crane.mkLib pkgs;
      src = craneLib.cleanCargoSource ./.;

      commonArgs = {
        inherit src;
        strictDeps = true;

        buildInputs = [] ++ lib.optionals pkgs.stdenv.isDarwin [pkgs.libiconv];
      };

      craneLibLLvmTools =
        craneLib.overrideToolchain
        (toolchain.withComponents [
          "cargo"
          "llvm-tools"
          "rustc"
        ]);

      devShellTools = toolchain.withComponents [
        "rustc"
        "cargo"
        "rustfmt"
        "rust-src"
        "rust-analyzer"
      ];

      mkToolchain = fenix.packages.${system}.combine;
      piTarget = fenix.packages.${system}.targets."armv7-unknown-linux-gnueabihf".stable;

      mobileTargets = mkToolchain (with toolchain; [
        cargo
        rustc
        rustfmt
        rust-src
        rust-analyzer
        piTarget.rust-std
      ]);


      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      chicken-door = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          doCheck = false;
        });
    in {
      checks = {
        inherit chicken-door;

        chicken-door-clippy = craneLib.cargoClippy (commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

        chicken-door-doc = craneLib.cargoDoc (commonArgs
          // {
            inherit cargoArtifacts;
          });

        chicken-door-fmt = craneLib.cargoFmt {
          inherit src;
        };

        chicken-door-toml-fmt = craneLib.taploFmt {
          src = pkgs.lib.sources.sourceFilesBySuffices src [".toml"];
        };

        chicken-door-audit = craneLib.cargoAudit {
          inherit src advisory-db;
        };

        chicken-door-deny = craneLib.cargoDeny {
          inherit src;
        };

        chicken-door-nextest = craneLib.cargoNextest (commonArgs
          // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
            cargoNextestPartitionsExtraArgs = "--no-tests=pass";
          });
      };

      packages =
        {
          default = chicken-door;
        }
        // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
          chicken-door-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs
            // {
              inherit cargoArtifacts;
            });
        };

      apps.default = flake-utils.lib.mkApp {
        drv = chicken-door;
      };

      devShells.default = pkgs.mkShell {
    strictDeps = true;
    nativeBuildInputs = with pkgs; [
      cargo-leptos
      cargo-cross
      rustup
      rustPlatform.bindgenHook
      dart-sass
      binaryen
    ];
    # libraries here
    buildInputs =
      [
      ];
    RUSTC_VERSION = "stable";
    # https://github.com/rust-lang/rust-bindgen#environment-variables
    shellHook = ''
      export PATH="''${CARGO_HOME:-~/.cargo}/bin":"$PATH"
      export PATH="''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-${pkgs.stdenv.hostPlatform.rust.rustcTarget}/bin":"$PATH"
    '';
  };
      # default = craneLib.devShell {
      #   checks = self.checks.${system};

      #   RUST_SRC_PATH = "${toolchain.rust-src}/lib/rustlib/src/rust/library";

      #   packages = with pkgs; [
      #     cargo-leptos
      #     rustup
      #     leptosfmt
      #     cargo-generate
      #     lldb
      #     lld
      #     # devShellTools
      #     mobileTargets
      #     jujutsu
      #     dart-sass
      #     binaryen
      #     cargo-cross
      #   ];
      # };

      formatter = pkgs.alejandra;
    });
}
# {
#   description = "A Nix-flake-based Python development environment";

#   inputs = {
#     nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    # crane.url = "github:ipetkov/crane";

    # fenix = {
    #   url = "github:nix-community/fenix";
    #   inputs.nixpkgs.follows = "nixpkgs";
    #   inputs.rust-analyzer-src.follows = "";
    # };

    # flake-utils.url = "github:numtide/flake-utils";

    # advisory-db = {
    #   url = "github:rustsec/advisory-db";
    #   flake = false;
    # };
  # };

#   outputs = { self, nixpkgs }:
#     let
#       supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
#       forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
#         pkgs = import nixpkgs { inherit system; crossSystem = { config = "armv7-unknown-linux-gnueabihf"; };};
#       });
#     in
#     {
#       devShells = forEachSupportedSystem ({ pkgs }: {
#         default = pkgs.mkShell {
#           strictDeps = true;
#           nativeBuildInputs = with pkgs; [
#             cargo-leptos
#             rustup
#             rustPlatform.bindgenHook
#             dart-sass
#           ];
#           # libraries here
#           buildInputs =
#             [
#             ];
#           RUSTC_VERSION = "stable";
#           # https://github.com/rust-lang/rust-bindgen#environment-variables
#           shellHook = ''
#             export PATH="''${CARGO_HOME:-~/.cargo}/bin":"$PATH"
#             export PATH="''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-${pkgs.stdenv.hostPlatform.rust.rustcTarget}/bin":"$PATH"
#           '';
#         };
#       });
#     };
# }
