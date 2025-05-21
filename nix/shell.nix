{ pkgs, rust-pkgs, ctx }:

pkgs.mkShell {
  packages = [ ctx.rust pkgs.just rust-pkgs.cargo-watch ] ++ ctx.build-deps;
}
