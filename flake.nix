{
  description = "browzer";
  inputs = {
    nixpkgs.url =
      "github:nixos/nixpkgs/2f9173bde1d3fbf1ad26ff6d52f952f9e9da52ea";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust_1_84_0-pkgs.url =
      "github:nixos/nixpkgs/d98abf5cf5914e5e4e9d57205e3af55ca90ffc1d";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };
  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust-pkgs = inputs.rust_1_84_0-pkgs.legacyPackages.${system};

        ctx = {
          package = {
            name = "browzer";
            version = "0.0.1";
            src = ./.;
          };
          rust = pkgs.rust-bin.stable."1.84.0".default;
          build-deps = [ ];
        };

        package = import ./nix/package.nix { inherit crane pkgs ctx; };
        devShell = import ./nix/shell.nix { inherit pkgs rust-pkgs ctx; };
      in {
        formatter = pkgs.nixfmt-classic;
        devShells.default = devShell;
        packages.default = package;
        apps.default = {
          type = "app";
          program = "${package}/bin/${ctx.package.name}";
        };
      });
}
