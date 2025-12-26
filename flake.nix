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
              sqlx-cli
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
              || (pkgs.lib.hasSuffix ".svg" path)
              || (pkgs.lib.hasSuffix ".sql" path)
              || (pkgs.lib.hasInfix ".sqlx" path)
              || (craneLib.filterCargoSources path type);
          };

          commonArgs = {
            inherit src;
            pname = "djv";
            version = "0.1.0";
            strictDeps = true;

            buildInputs = with pkgs; [
              openssl
            ];

            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
          };

          # Build deps separately using standard cargo check (not cargo-leptos)
          # This vendors dependencies for the main build
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        in
        {
          default = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;

              nativeBuildInputs = with pkgs; [
                pkg-config
                cargo-leptos
                binaryen
                dart-sass
                wasm-bindgen-cli
                makeWrapper
              ];

              # cargo-leptos handles its own multi-target build (wasm32 + native)
              buildPhaseCargoCommand = ''
                cargo leptos build --release
              '';

              # We handle installation ourselves since cargo-leptos doesn't produce
              # the standard cargo build log that crane expects
              doNotPostBuildInstallCargoBinaries = true;

              installPhaseCommand = ''
                mkdir -p $out/bin $out/share/djv
                cp target/release/djv $out/bin/
                cp -r target/site/* $out/share/djv/
                wrapProgram $out/bin/djv \
                  --set LEPTOS_SITE_ROOT "$out/share/djv"
              '';

              doCheck = false;
            }
          );
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

            listenAddress = lib.mkOption {
              type = lib.types.str;
              default = "127.0.0.1:3000";
              description = "Address and port to listen on";
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

            vcsRevision = lib.mkOption {
              type = lib.types.str;
              default = "";
              description = "Git commit hash for telemetry";
            };

            vcsRefName = lib.mkOption {
              type = lib.types.str;
              default = "";
              description = "Git branch or tag name for telemetry";
            };

            database = {
              enable = lib.mkEnableOption "PostgreSQL database for projects";

              host = lib.mkOption {
                type = lib.types.str;
                default = "/run/postgresql";
                description = "PostgreSQL host or socket path";
              };

              port = lib.mkOption {
                type = lib.types.port;
                default = 5432;
                description = "PostgreSQL port";
              };

              name = lib.mkOption {
                type = lib.types.str;
                default = "djv";
                description = "Database name";
              };

              user = lib.mkOption {
                type = lib.types.str;
                default = "djv";
                description = "Database user";
              };

              passwordFile = lib.mkOption {
                type = lib.types.nullOr lib.types.path;
                default = null;
                description = "Path to file containing database password (optional for socket auth)";
              };
            };

            sync = {
              enable = lib.mkEnableOption "background sync from code forges";

              intervalSeconds = lib.mkOption {
                type = lib.types.int;
                default = 3600;
                description = "Sync interval in seconds";
              };

              github = {
                user = lib.mkOption {
                  type = lib.types.nullOr lib.types.str;
                  default = null;
                  description = "GitHub username to sync repositories from";
                };

                tokenFile = lib.mkOption {
                  type = lib.types.nullOr lib.types.path;
                  default = null;
                  description = "Path to file containing GitHub token (optional, increases rate limits)";
                };
              };

              cratesIo = {
                user = lib.mkOption {
                  type = lib.types.nullOr lib.types.str;
                  default = null;
                  description = "crates.io username to sync crates from";
                };
              };

              contributions = {
                user = lib.mkOption {
                  type = lib.types.nullOr lib.types.str;
                  default = null;
                  description = "GitHub username to track contributions from";
                };
              };
            };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.djv = {
              description = "djv.sh homepage";
              wantedBy = [ "multi-user.target" ];
              after = [ "network.target" ] ++ lib.optionals cfg.database.enable [ "postgresql.service" ];
              requires = lib.optionals cfg.database.enable [ "postgresql.service" ];

              environment =
                let
                  # Build DATABASE_URL based on config
                  # For socket auth: postgres:///dbname?host=/run/postgresql&user=user
                  # For TCP: postgres://user:password@host:port/dbname
                  dbUrl =
                    if cfg.database.enable then
                      if lib.hasPrefix "/" cfg.database.host then
                        "postgres:///${cfg.database.name}?host=${cfg.database.host}&user=${cfg.database.user}"
                      else
                        "postgres://${cfg.database.user}@${cfg.database.host}:${toString cfg.database.port}/${cfg.database.name}"
                    else
                      null;
                in
                {
                  DJV_LISTEN = cfg.listenAddress;
                  OTEL_EXPORTER_OTLP_ENDPOINT = cfg.opentelemetryEndpoint;
                  OTEL_RESOURCE_ATTRIBUTES = "deployment.environment.name=${cfg.environment}";
                }
                // lib.optionalAttrs (cfg.vcsRevision != "") {
                  VCS_REF_HEAD_REVISION = cfg.vcsRevision;
                }
                // lib.optionalAttrs (cfg.vcsRefName != "") {
                  VCS_REF_HEAD_NAME = cfg.vcsRefName;
                }
                // lib.optionalAttrs (dbUrl != null) {
                  DATABASE_URL = dbUrl;
                }
                // lib.optionalAttrs cfg.sync.enable {
                  DJV_SYNC_ENABLED = "true";
                  DJV_SYNC_INTERVAL = toString cfg.sync.intervalSeconds;
                }
                // lib.optionalAttrs (cfg.sync.github.user != null) {
                  DJV_GITHUB_USER = cfg.sync.github.user;
                }
                // lib.optionalAttrs (cfg.sync.cratesIo.user != null) {
                  DJV_CRATES_IO_USER = cfg.sync.cratesIo.user;
                }
                // lib.optionalAttrs (cfg.sync.contributions.user != null) {
                  DJV_CONTRIBUTIONS_USER = cfg.sync.contributions.user;
                };

              serviceConfig = {
                Type = "simple";
                Restart = "always";
                RestartSec = 5;

                DynamicUser = true;

                NoNewPrivileges = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                PrivateTmp = true;
              }
              // lib.optionalAttrs (cfg.sync.github.tokenFile == null) {
                ExecStart = "${cfg.package}/bin/djv";
              }
              // lib.optionalAttrs (cfg.sync.github.tokenFile != null) {
                LoadCredential = "github-token:${cfg.sync.github.tokenFile}";
              };

              script = lib.mkIf (cfg.sync.github.tokenFile != null) ''
                export DJV_GITHUB_TOKEN="$(cat $CREDENTIALS_DIRECTORY/github-token)"
                exec ${cfg.package}/bin/djv
              '';
            };
          };
        };
    };
}
