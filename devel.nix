{ nixpkgs, rust-overlay }:
let
  systems = [
    "aarch64-linux"
    "x86_64-linux"
  ];
  forAllSystems = nixpkgs.lib.genAttrs systems;
in
{
  devShells = forAllSystems (
    system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (
        toolchain:
        toolchain.default.override {
          extensions = [
            "clippy"
            "miri"
            "rust-src"
            "rustfmt"
          ];
        }
      );
    in
    {
      default = pkgs.mkShell {
        packages = [
          rustToolchain
          pkgs.cargo-nextest
          pkgs.cargo-expand
          pkgs.rust-analyzer
        ];

        RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
      };
    }
  );
}
