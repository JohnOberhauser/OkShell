{
  description = "OkShell — a customizable shell for Hyprland";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};
    in
    {
      packages = forAllSystems (system:
        let pkgs = pkgsFor system; in
        {
          okshell = pkgs.callPackage ./package.nix { };
          default = self.packages.${system}.okshell;
        });

      devShells = forAllSystems (system:
        let pkgs = pkgsFor system; in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.okshell ];
            packages = with pkgs; [
              rustc
              cargo
              rust-analyzer
              rustfmt
              clippy
            ];
          };
        });

      homeManagerModules.default = import ./nix/hm-module.nix self;
      homeManagerModules.okshell = self.homeManagerModules.default;

      nixosModules.default = import ./nix/nixos-module.nix self;
      nixosModules.okshell = self.nixosModules.default;

      formatter = forAllSystems (system: (pkgsFor system).nixfmt-rfc-style);
    };
}
