{
  description = "inori mpd client";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;
      systems = lib.systems.flakeExposed;
      eachDefaultSystem = f: builtins.foldl' lib.attrsets.recursiveUpdate { }
        (map f systems);
    in
    eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        formatters = [ pkgs.cargo pkgs.rustfmt pkgs.nixpkgs-fmt ];
        linters = [ pkgs.clippy pkgs.statix ];
      in
      {
        legacyPackages.${system} = self.packages.${system};

        packages.${system} = (import ./pkgs { inherit pkgs; }) // {
          default = self.packages.${system}.inori;
        };

        formatter.${system} = pkgs.writeShellApplication {
          name = "formatter";
          runtimeInputs = formatters;
          text = ''
            cargo fmt
            nixpkgs-fmt "$@"
          '';
        };

        checks.${system}.lint = pkgs.rustPlatform.buildRustPackage {
          name = "lint";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          doCheck = true;
          nativeCheckInputs = linters ++ formatters;
          checkPhase = ''
            cargo fmt --check
            cargo check
            cargo clippy
            cargo test
            nixpkgs-fmt --check .
            statix check
          '';
          installPhase = "touch $out";
        };

        apps.${system} = {
          update-logo = {
            type = "app";
            program = lib.getExe (pkgs.writeShellApplication {
              name = "update-logo";
              runtimeInputs = [ ];
              text = ''
                cp ${self.packages.${system}.inori-logo}/inori-logo.svg \
                  -T images/inori-logo.svg
                cp ${self.packages.${system}.inori-logo}/inori-logo-white.svg \
                  -T images/inori-logo-white.svg
                chmod +w images/inori-logo.svg
                chmod +w images/inori-logo-white.svg
              '';
            });
          };
        };

        devShells.${system}.default = pkgs.mkShell {
          packages = [
            pkgs.rust-analyzer
            pkgs.rustc
          ]
          ++ linters
          ++ formatters;
        };
      }
    );
}
