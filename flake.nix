{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/24.05"; # that's 23.05
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    jujutsu.url = "github:martinvonz/jj/bf76080f42f77cad934d9a5202c7b7d29ab2c890";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    rust-overlay,
    jujutsu,
  }:
    utils.lib.eachDefaultSystem (system: let
      #pkgs = nixpkgs.legacyPackages."${system}";
      overlays = [(import rust-overlay) (jujutsu.overlays.default)];
      pkgs = import nixpkgs {inherit system overlays;};
      rust = pkgs.rust-bin.stable."1.81.0".default.override {
        targets = ["x86_64-unknown-linux-musl"];
      };

      # Override the version used in naersk
      naersk-lib = naersk.lib."${system}".override {
        cargo = rust;
        rustc = rust;
      };

      bacon = pkgs.bacon;
    in rec {
      # `nix build`
      packages.my-project = naersk-lib.buildPackage {
        pname = "mbf_rust_processor";
        root = ./.;
        nativeBuildInputs = with pkgs; [pkg-config];
        buildInputs = with pkgs; [openssl cmake clang];
        cargoBuildOptions = old: old ++ ["--target" "x86_64-unknown-linux-musl"];
        CC="clang";
        release = true;
      };
      packages.check = naersk-lib.buildPackage {
        src = ./.;
        mode = "check";
        nativeBuildInputs = with pkgs; [pkg-config zstd.bin];
        buildInputs = with pkgs; [openssl cmake zstd.dev];
      };
      packages.test = naersk-lib.buildPackage {
        src = ./.;
        mode = "test";
        nativeBuildInputs = with pkgs; [pkg-config];
        buildInputs = with pkgs; [openssl cmake];
      };

      defaultPackage = packages.my-project;

      # `nix run`
      apps.my-project = utils.lib.mkApp {drv = packages.my-project;};
      defaultApp = apps.my-project;

      # `nix develop`
      devShell = pkgs.mkShell {
        # supply the specific rust version
        nativeBuildInputs = [
          rust
          pkgs.rust-analyzer
          pkgs.git
          pkgs.cargo-udeps
          pkgs.cargo-crev
          pkgs.cargo-vet
          pkgs.cargo-outdated
          pkgs.cargo-audit
          pkgs.jujutsu
          pkgs.openssl
          pkgs.pkg-config
          pkgs.cargo-flamegraph
          pkgs.cmake
          bacon
        ];
      };
    });
}
# {

