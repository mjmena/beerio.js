{
  description = "A simple Nix flake for beerio.js with Python";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          nodejs
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
