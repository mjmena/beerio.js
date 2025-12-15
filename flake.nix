{
  description = "A simple Nix flake for beerio.js with Python";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, fenix }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      toolchain = fenix.packages.${system}.stable.toolchain;
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          nodejs
          toolchain
          pkg-config
          openssl
        ];
      };

      packages.${system}.default = pkgs.writeShellScriptBin "beerio-server" ''
        if [ ! -d "node_modules" ]; then
          ${pkgs.nodejs}/bin/npm install
        fi
        ${pkgs.nodejs}/bin/npm run dev -- "$@"
      '';
    };
}
