{
  description = "Nix flake with Ghidra as a dependency";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }:
    let
      pkgs = nixpkgs.legacyPackages."x86_64-linux";
    in
    {
      devShell.x86_64-linux = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustc
          cargo
          valgrind
        ];
      };
    };
}

