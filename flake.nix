{
  description = "Nix flake with Ghidra as a dependency";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    #cwe_checker.url = "github:felix-ulonska/cwe_checker/feat/split_stack_mem";
    cwe_checker.url = "/home/jabbi/Projects/masterarbeit/cwe_checker";
  };

  outputs = { self, nixpkgs, cwe_checker }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages."x86_64-linux";
      busybox-variants = import ./pkgs/busybox.nix { inherit pkgs; };
      cwe_checker_bin = cwe_checker.outputs.packages.${system}.default;
      soundness-test-bins = pkgs.rustPlatform.buildRustPackage {
        pname = "cwe_checker";
        name = "cwe_checker";
        src = ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };
      };
      cwe_checker_soundness_test = pkgs.writeScriptBin "cwe-checker-sound-test" ''
      #!/bin/sh
      PATH="${pkgs.valgrind}/bin:${cwe_checker_bin}/bin:$PATH" ${soundness-test-bins}/bin/soundness-testing-valgrind $@;
      '';

    in
    {
      devShell.x86_64-linux = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustc
          cargo
          valgrind
          croc
          cwe_checker.outputs.packages.${system}.default
          busybox-variants.awk
        ];
      };
      packages.x86_64-linux.default = cwe_checker_soundness_test;
    };
}

