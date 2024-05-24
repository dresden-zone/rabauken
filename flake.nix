{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, naersk, fenix }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      makeTest = pkgs.callPackage "${nixpkgs}/nixos/tests/make-test-python.nix";

      toolchain = with fenix.packages.${system}; combine [
        latest.cargo
        latest.rustc
      ];

      migration-package = pkgs.callPackage ./derivation.nix {
        cargoToml = ./lib/migration/Cargo.toml;
      };

      chef = pkgs.callPackage ./derivation.nix {
        cargoToml = ./bin/chef/Cargo.toml;
      };

      maid = pkgs.callPackage ./derivation.nix {
        cargoToml = ./bin/maid/Cargo.toml;
      };
    in
    {
      checks.${system}.test-sea-orm-cli-migration =
        let
          username = "postgres";
          password = "password";
          database = "database";
          migrations_dir = ./migration;
        in
        makeTest
          {
            name = "test-sea-orm-cli-migration";
            nodes = {
              server = { lib, config, pkgs, ... }: {
                services.postgresql = {
                  enable = true;
                  ensureDatabases = [ database ];
                  ensureUsers = [{
                    name = username;
                    ensurePermissions = {
                      "DATABASE ${database}" = "ALL PRIVILEGES";
                    };
                  }];
                  initialScript = pkgs.writeScript "initScript" ''
                    ALTER USER postgres WITH PASSWORD '${password}';
                  '';
                };

                systemd.services.postgresql.postStart = lib.mkAfter ''
                  ${migration-package}/bin/migration refresh --database-url postgresql://${username}:${password}@localhost/${database}
                '';
              };
            };
            testScript = ''
              start_all()
              server.wait_for_unit("postgresql.service")
              server.execute("${pkgs.sea-orm-cli}/bin/sea-orm-cli generate entity --database-url postgresql://${username}:${password}@localhost/${database} --date-time-crate time --with-serde both --output-dir /tmp/out")
              server.copy_from_vm("/tmp/out", "")
            '';
          }
          {
            inherit pkgs;
            inherit (pkgs) system;
          };

      packages.${system} = {
        update-schema = pkgs.writeScriptBin "update-schema" ''
          nix build ${self}#checks.${system}.test-sea-orm-cli-migration
          BUILD_DIR=$(nix build ${self}#checks.${system}.test-sea-orm-cli-migration --no-link --print-out-paths)
          rm -rf ./lib/entity/src/models/*
          cp -r $BUILD_DIR/out/* ./lib/entity/src/models/
          chmod -R 644 ./lib/entity/src/models/*
          #${pkgs.git}/bin/git apply ${./lib/entity/patch/fixed_time_crate_serde_configuration.patch} --verbose
          ${pkgs.cargo}/bin/cargo fmt
        '';

        run-migration-based = pkgs.writeScriptBin "run-migration" ''
          ${pkgs.sea-orm-cli}/bin/sea-orm-cli migration run --migration-dir ${self}/migrations-based
        '';
        inherit chef maid;
      };

      devShells."x86_64-linux".default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ pkg-config postgresql_14 openssl sea-orm-cli ];
      };

      overlays.default = final: prev: {
        inherit (self.packages.${prev.system})
          chef maid;
      };

      nixosModules = {
        maid = import ./nixos-module/maid.nix;
        chef = import ./nixos-module/chef.nix;
      };
    };
}
