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
        formatter = pkgs.nixpkgs-fmt;
        linters = [ pkgs.statix ];
      in
      {
        legacyPackages.${system} = self.packages.${system};

        packages.${system} = import ./pkgs { inherit pkgs; };

        formatter.${system} = formatter;

        checks.${system}.lint = pkgs.stdenvNoCC.mkDerivation {
          name = "lint";
          src = ./.;
          doCheck = true;
          nativeCheckInputs = linters ++ lib.singleton formatter;
          checkPhase = ''
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
                chmod +w images/inori-logo.svg
              '';
            });
          };
        };

        devShells.${system}.default = (pkgs.mkShellNoCC.override {
          stdenv = pkgs.stdenvNoCC.override {
            initialPath = [ pkgs.coreutils ];
          };
        }) {
          packages = [
          ]
          ++ linters
          ++ lib.singleton formatter;
        };
      }
    );
}
