{
  description = "Description for the project";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    process-compose.url = "github:Platonic-Systems/process-compose-flake";
    services.url = "github:juspay/services-flake";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.process-compose.flakeModule
      ];
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];
      perSystem = {
        config,
        self',
        inputs',
        lib,
        pkgs,
        system,
        ...
      }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [(import inputs.rust-overlay)];
        };

        devShells.default = pkgs.clangStdenv.mkDerivation {
          name = "postgrestools";
          nativeBuildInputs = with pkgs; [
            config.packages.toolchain
            cmake
            postgresql
          ];
          buildInputs = with pkgs; [rust-jemalloc-sys];
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };

        packages.toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        process-compose.devenv = {
          imports = [
            inputs.services.processComposeModules.default
          ];

          cli.options.no-server = false;

          services.postgres.pg1 = {
            enable = true;
            # package = config.packages.postgresql;
            # extensions = exts:
            #   with exts;
            #   with config.packages; [
            #     pg_cron
            #     pgddl
            #     pgtap
            #     plpgsql_check
            #   ];
            initialDatabases = lib.mkForce [];
            settings = {
              log_statement = "all";
              logging_collector = false;
              # shared_preload_libraries = "pg_cron,plpgsql_check";
            };
          };
        };
      };
    };
}
