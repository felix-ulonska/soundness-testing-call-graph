{
  description = "Nix flake with Ghidra as a dependency";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    #cwe_checker.url = "github:felix-ulonska/cwe_checker/feat/split_stack_mem";
    cwe_checker.url = "/home/jabbi/Projects/masterarbeit/cwe_checker";
  };

  outputs = { self, nixpkgs, cwe_checker }:
    let
      pkgs = nixpkgs.legacyPackages."x86_64-linux";
    in
    {
      devShell.x86_64-linux = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustc
          cargo
          valgrind
          croc
          cwe_checker.outputs.packages.${system}.default
        ];
      };
    };
}

