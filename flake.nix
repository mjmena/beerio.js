{
  description = "Beerio Kart Mission Randomizer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
        # Use Fenix for the dev toolchain
        toolchain = fenix.packages.${system}.stable.toolchain;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.makeWrapper ];

          buildInputs = [ ];

          postInstall = ''
            mkdir -p $out/share/beerio
            cp -r assets missions.json $out/share/beerio/

            wrapProgram $out/bin/beerio \
              --run "cd $out/share/beerio"
          '';
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            toolchain
            pkgs.pkg-config
            pkgs.openssl
          ];
        };
      }
    )
    // {
      nixosModules.default = import ./nix/module.nix;
    };
}
