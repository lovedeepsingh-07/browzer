{ pkgs, crane, ctx }:

let craneLib = (crane.mkLib pkgs).overrideToolchain ctx.rust;

in craneLib.buildPackage {
  src = ctx.package.src;
  strictDeps = true;
  buildInputs = ctx.build-deps;
}
