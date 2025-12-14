{
  description = "A simple Nix flake for beerio.js with Python";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          python3
          nodePackages.vite
        ];
      };

      packages.${system}.default = pkgs.writeShellScriptBin "beerio-server" ''
        ${pkgs.nodePackages.vite}/bin/vite "$@"
      '';

      apps.${system}.default = {
        type = "app";
        program = "${self.packages.${system}.default}/bin/beerio-server";
      };
    };
}
