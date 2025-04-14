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

      toolchain = fenix.packages.${system}.complete;

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

      devShells.default = craneLib.devShell {
        checks = self.checks.${system};

        RUST_SRC_PATH = "${toolchain.rust-src}/lib/rustlib/src/rust/library";

        packages = with pkgs; [
          cargo-leptos
          leptosfmt
          cargo-generate
          lldb
          lld
          devShellTools
          jujutsu
          dart-sass
        ];
      };

      formatter = pkgs.alejandra;
    });
}
