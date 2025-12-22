{
  description = "djv.sh - personal homepage";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      crane,
    }:
    let
      forAllSystems =
        fn:
        nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed (
          system: fn nixpkgs.legacyPackages.${system} fenix.packages.${system} system
        );
    in
    {
      formatter = forAllSystems (
        pkgs: _: _:
        pkgs.nixfmt-rfc-style
      );

      devShells = forAllSystems (
        pkgs: fenixPkgs: _:
        let
          rustToolchain = fenixPkgs.combine [
            fenixPkgs.latest.cargo
            fenixPkgs.latest.rustc
            fenixPkgs.latest.rust-src
            fenixPkgs.latest.clippy
            fenixPkgs.latest.rustfmt
            fenixPkgs.targets.wasm32-unknown-unknown.latest.rust-std
          ];
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              openssl
              pkg-config
            ];

            nativeBuildInputs = with pkgs; [
              rustToolchain
              cargo-leptos
              binaryen
              dart-sass
              wasm-bindgen-cli
              just
              rust-analyzer

              nixfmt-rfc-style
              statix
              deadnix
            ];

            RUST_SRC_PATH = "${fenixPkgs.latest.rust-src}/lib/rustlib/src/rust/library";

            shellHook = ''
              echo "djv dev shell"
              echo "  cargo leptos watch  - start dev server"
              echo "  just check          - run fmt, lint, test"
            '';
          };
        }
      );

      packages = forAllSystems (
        pkgs: fenixPkgs: _:
        let
          rustToolchain = fenixPkgs.combine [
            fenixPkgs.latest.cargo
            fenixPkgs.latest.rustc
            fenixPkgs.targets.wasm32-unknown-unknown.latest.rust-std
          ];

          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter =
              path: type:
              (pkgs.lib.hasSuffix ".scss" path)
              || (pkgs.lib.hasSuffix ".ico" path)
              || (craneLib.filterCargoSources path type);
          };

        in
        {
          default = craneLib.buildPackage {
            inherit src;
            pname = "djv";
            version = "0.1.0";
            strictDeps = true;

            buildInputs = with pkgs; [
              openssl
            ];

            nativeBuildInputs = with pkgs; [
              pkg-config
              cargo-leptos
              binaryen
              dart-sass
              wasm-bindgen-cli
              makeWrapper
            ];

            # cargo-leptos handles its own multi-target build (wasm32 + native)
            # so we skip crane's buildDepsOnly and let cargo-leptos manage deps
            buildPhaseCargoCommand = ''
              cargo leptos build --release
            '';

            installPhaseCommand = ''
              mkdir -p $out/bin $out/share/djv
              cp target/release/djv $out/bin/
              cp -r target/site/* $out/share/djv/
              wrapProgram $out/bin/djv \
                --set LEPTOS_SITE_ROOT "$out/share/djv"
            '';

            doCheck = false;
          };
        }
      );

      nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          cfg = config.services.djv;
        in
        {
          options.services.djv = {
            enable = lib.mkEnableOption "djv.sh homepage service";

            package = lib.mkOption {
              type = lib.types.package;
              inherit (self.packages.${pkgs.stdenv.hostPlatform.system}) default;
              description = "The djv package to use";
            };

            socketPath = lib.mkOption {
              type = lib.types.str;
              default = "/run/djv/djv.sock";
              description = "Path to the Unix socket";
            };

            environment = lib.mkOption {
              type = lib.types.str;
              default = "production";
              description = "Deployment environment name";
            };

            opentelemetryEndpoint = lib.mkOption {
              type = lib.types.str;
              default = "http://127.0.0.1:4318";
              description = "OpenTelemetry collector endpoint";
            };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.djv = {
              description = "djv.sh homepage";
              wantedBy = [ "multi-user.target" ];
              after = [ "network.target" ];

              environment = {
                DJV_SOCKET = cfg.socketPath;
                OTEL_EXPORTER_OTLP_ENDPOINT = cfg.opentelemetryEndpoint;
                OTEL_RESOURCE_ATTRIBUTES = "deployment.environment.name=${cfg.environment}";
              };

              serviceConfig = {
                Type = "simple";
                ExecStart = "${cfg.package}/bin/djv";
                Restart = "always";
                RestartSec = 5;

                RuntimeDirectory = "djv";
                RuntimeDirectoryMode = "0755";

                DynamicUser = true;
                NoNewPrivileges = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                PrivateTmp = true;
              };
            };
          };
        };
    };
}
