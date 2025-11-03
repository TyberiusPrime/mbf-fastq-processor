{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.05"; # that's 23.05
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    rust-overlay,
  }:
    utils.lib.eachDefaultSystem (system: let
      #pkgs = nixpkgs.legacyPackages."${system}";
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {inherit system overlays;};
      rust = pkgs.rust-bin.stable."1.86.0".default.override {
        targets = ["x86_64-unknown-linux-musl"];
        extensions = ["llvm-tools-preview"];
      };

      # Override the version used in naersk
      naersk-lib = naersk.lib."${system}".override {
        cargo = rust;
        rustc = rust;
      };

      bacon = pkgs.bacon;
    in rec {
      # `nix build`
      packages.mbf-fastq-processor = naersk-lib.buildPackage {
        pname = "mbf-fastq-processor";
        root = ./.;
        nativeBuildInputs = with pkgs; [pkg-config];
        buildInputs = with pkgs; [openssl cmake];
        release = true;
        CARGO_PROFILE_RELEASE_debug = "0";
        # copyBinsFilter = ''
        #   select(.reason == "compiler-artifact" and .executable != null and .profile.test == false and .target.name != "mbf-fastq-processor-test-runner")
        # '';
      };
      packages.mbf-fastq-processor_other_linux =
        (naersk-lib.buildPackage {
          pname = "mbf-fastq-processor";
          root = ./.;
          nativeBuildInputs = with pkgs; [pkg-config];
          buildInputs = with pkgs; [openssl cmake];
          release = true;
          CARGO_PROFILE_RELEASE_debug = "0";
          # copyBinsFilter = ''
          #   select(.reason == "compiler-artifact" and .executable != null and .profile.test == false and .target.name != "mbf-fastq-processor-test-runner")
          # '';
        })
        .overrideAttrs {
          # make it compatible with other linuxes. It's statically linked anyway
          postInstall = ''
            patchelf $out/bin/mbf-fastq-processor --set-interpreter "/lib64/ld-linux-x86-64.so.2"
          '';
        };
      packages.mbf-fastq-processor-docker = let
        binary = packages.mbf-fastq-processor_other_linux;
      in
        pkgs.dockerTools.buildLayeredImage {
          name = "mbf-fastq-processor";
          tag = "latest";
          # provide a minimal base with glibc and a busybox shell
          contents = [pkgs.busybox pkgs.glibc binary];
          config = {
            Env = ["PATH=/usr/local/bin:/bin"];
            Entrypoint = ["/bin/mbf-fastq-processor"];
            WorkingDir = "/work";
          };
        };
      packages.check = naersk-lib.buildPackage {
        src = ./.;
        mode = "check";
        name = "mbf-fastq-processor";
        nativeBuildInputs = with pkgs; [pkg-config zstd.bin];
        buildInputs = with pkgs; [openssl cmake zstd.dev];
      };
      packages.test = naersk-lib.buildPackage {
        # not using naersk test mode, it eats the binaries, we need that binary
        pname = "mbf-fastq-processor";
        root = ./.;
        nativeBuildInputs = with pkgs; [pkg-config python3];
        buildInputs = with pkgs; [openssl cmake hugo];
        release = true;
        CARGO_PROFILE_RELEASE_debug = "0";
        postInstall = ''
          # run the friendly panic test, expect a non 0 return code.
          # capture stderr

          result=$( { cargo run --release --bin mbf-fastq-processor -- --test-friendly-panic 1>/dev/null; } 2>&1 ) || status=$? : "${status:=0}"
          if [ "$status" -eq 0 ]; then
            echo "Unexpected success"
            exit 1
          fi
          if [[ ! $result =~ "this is embarrassing" ]]; then
              echo "Error: friendly panic message ' not found in stderr"
              exit 1
          fi

          cargo test --release
        '';

        # src = ./.;
        # buildInputs = with pkgs; [openssl cmake];
        # mode = "test";
        # nativeBuildInputs = with pkgs; [pkg-config cargo-nextest];
        # cargoTestCommands = old: ["cargo nextest run $cargo_test_options --no-fail-fast"];
        # copySources = ["tests" "test_cases" "dev"];
        # copyBins = true;

        # override = {
        #   buildPhase = ":";
        #   postCheck = ''
        #      # make sure that the friendly panic test outputs a friendly panic
        #      ls -la
        #     cargo build --release
        #      if [ $? -ne 0 ]; then
        #          echo "Error: Command failed with non-zero status code"
        #          exit 1
        #      fi
        #      result=`cargo run --release -- --friendly-panic-test`

        #      # Check if stderr contains 'this is embarrasing'
        #      if grep -q "this is embarrasing" <(echo "$result"); then
        #          echo "Error: 'this is embarrasing' found in stderr"
        #          exit 1
        #      fi

        #      # now run our actual test cases
        #      cat Cargo.toml
        #     cargo run --release --bin mbf-fastq-processor-test-runner test_cases
        #   '';
        # };
        # doCheck = true;
      };
      # haven't been able to get this to work
      # packages.coverage = naersk-lib.buildPackage {
      #   src = ./.;
      #   buildInputs = with pkgs; [openssl cmake];
      #   mode = "test";
      #   nativeBuildInputs = with pkgs; [pkg-config cargo-nextest cargo-llvm-cov];
      #   cargoTestCommands = old: ["cargo llvm-cov nextest --no-tests=fail --run-ignored all"];
      #   override = {
      #     buildPhase = ":";
      #     postCheck = ''
      #       cp  target/llvm-cov/html $out/ -r
      #       '';
      #   };
      #   doCheck = true;
      # };
      #cargoTestCommands = old: ["cargo llvm-cov --html nextest --verbose $cargo_test_options"];

      defaultPackage = packages.mbf-fastq-processor;

      # `nix run`
      apps.mbf-fastq-processor = utils.lib.mkApp {drv = packages.mbf-fastq-processor;};
      defaultApp = apps.mbf-fastq-processor;

      # `nix develop`
      devShell = pkgs.mkShell {
        # supply the specific rust version
        nativeBuildInputs = [
          bacon
          pkgs.cargo-audit
          pkgs.cargo-crev
          pkgs.cargo-flamegraph
          pkgs.cargo-insta
          pkgs.cargo-nextest
          pkgs.cargo-llvm-cov
          pkgs.cargo-outdated
          #pkgs.cargo-udeps
          pkgs.cargo-machete
          pkgs.cargo-vet
          pkgs.cargo-license
          pkgs.cargo-deny
          pkgs.cmake
          pkgs.git
          pkgs.openssl
          pkgs.pkg-config
          pkgs.ripgrep
          pkgs.rust-analyzer
          pkgs.hugo
          rust
        ];
      };
    });
}
# {

