{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      naersk,
      rust-overlay,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        #pkgs = nixpkgs.legacyPackages."${system}";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.stable."1.93.1".default.override {
          targets = [ "x86_64-unknown-linux-musl" ];
          extensions = [
            "llvm-tools-preview"
            "rust-analyzer"
          ];
        };

        # Override the version used in naersk
        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };

        bacon = pkgs.bacon;

        # cargo-afl is not in nixpkgs, so we build it from the crates.io tarball.
        # The build produces just the `cargo-afl` binary — it does NOT compile
        # aflplusplus (build.rs only does that during `cargo install`, which we
        # bypass). Instead, we populate the xdg data dir cargo-afl looks in
        # with symlinks to `pkgs.aflplusplus` and wrap cargo-afl so it finds
        # them (plus the nix `cargo`, otherwise it panics with NotPresent).
        cargo-afl-unwrapped = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cargo-afl";
          version = "0.18.1";
          src = pkgs.fetchCrate {
            inherit pname version;
            hash = "sha256-W2ELM28vHs8xjgh0gRyH/O17kDgMFxKNOnnlbputQb0=";
          };
          cargoLock.lockFile = "${src}/Cargo.lock";
          doCheck = false;
        };

        # cargo-afl-common uses `rustc-<semver>-<short-hash>/afl.rs-<ver>` as
        # the xdg subdirectory. Extract it from the pinned rust toolchain so
        # the path matches at runtime.
        aflRustcDir = pkgs.lib.removeSuffix "\n" (builtins.readFile (pkgs.runCommand "afl-rustc-dir" { } ''
          ${rust}/bin/rustc -vV | ${pkgs.gawk}/bin/awk '
            /^rustc/ { ver=$2 }
            /^commit-hash:/ { printf "rustc-%s-%s", ver, substr($2, 1, 7) }
          ' > $out
        ''));

        aflXdgDataHome = pkgs.runCommand "afl-xdg-data-home" { } ''
          base=$out/afl.rs/${aflRustcDir}/afl.rs-${cargo-afl-unwrapped.version}
          mkdir -p "$base/afl/bin" "$base/afl-llvm"
          for b in ${pkgs.aflplusplus}/bin/afl-*; do
            ln -s "$b" "$base/afl/bin/$(basename "$b")"
          done
          ln -s ${pkgs.aflplusplus}/lib/afl/afl-compiler-rt.o "$base/afl-llvm/afl-compiler-rt.o"
        '';

        cargo-afl = pkgs.runCommand "cargo-afl-wrapped" {
          nativeBuildInputs = [ pkgs.makeWrapper ];
          inherit (cargo-afl-unwrapped) version meta;
          pname = "cargo-afl";
        } ''
          mkdir -p $out/bin
          makeWrapper ${cargo-afl-unwrapped}/bin/cargo-afl $out/bin/cargo-afl \
            --set-default XDG_DATA_HOME ${aflXdgDataHome} \
            --set-default CARGO ${rust}/bin/cargo \
            --prefix PATH : ${pkgs.lib.makeBinPath [ rust pkgs.aflplusplus ]}
        '';

        # Source filtered for the docker binary build: strips directories that
        # are not needed to compile the Rust workspace (dev tooling, test data,
        # documentation, build artefacts).  cookbooks/ must stay because it is
        # embedded via include_str!().
        dockerSrc = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            let
              baseName = baseNameOf (toString path);
              excludedDirs = [ "dev" "test_cases" "docs" "target" "target_claude" ".git" ".jj" ".github" ];
            in
            !(type == "directory" && builtins.elem baseName excludedDirs);
          name = "fastqrab-docker-src";
        };
      in
      rec {
        # `nix build`
        packages.fastqrab = naersk-lib.buildPackage {
          pname = "fastqrab";
          root = ./.;
          nativeBuildInputs = with pkgs; [
            pkg-config
            cmake
            gcc
            gnumake
          ];
          buildInputs = with pkgs; [
            openssl
            rapidgzip
            which
            mold
          ];
          release = true;
          CARGO_PROFILE_RELEASE_debug = "0";
          COMMIT_HASH = self.rev or (pkgs.lib.removeSuffix "-dirty" self.dirtyRev or "unknown-not-in-git");
          NIX_RAPIDGZIP = "${pkgs.rapidgzip}/bin/rapidgzip";

        };
        packages.fastqrab_other_linux =
          (naersk-lib.buildPackage {
            pname = "fastqrab";
            root = dockerSrc;
            nativeBuildInputs = with pkgs; [
              pkg-config
              cmake
              gcc
              gnumake
              mold
            ];
            buildInputs = with pkgs; [ openssl ];
            release = true;
            CARGO_PROFILE_RELEASE_debug = "0";
            COMMIT_HASH = self.rev or (pkgs.lib.removeSuffix "-dirty" self.dirtyRev or "unknown-not-in-git");
          }).overrideAttrs
            {
              # make it compatible with other linuxes. It's statically linked anyway
              postInstall = ''
                patchelf $out/bin/fastqrab --set-interpreter "/lib64/ld-linux-x86-64.so.2"
              '';
            };
        packages.fastqrab-docker =
          let
            binary = packages.fastqrab_other_linux;
          in
          pkgs.dockerTools.buildLayeredImage {
            name = "fastqrab";
            tag = "latest";
            # provide a minimal base with glibc and a busybox shell
            contents = [
              pkgs.busybox
              pkgs.bash
              pkgs.glibc
              pkgs.python3
              pkgs.rapidgzip
              pkgs.dockerTools.fakeNss
              binary
            ];
            config = {
              Env = [ "PATH=/usr/local/bin:/bin" ];
              Entrypoint = [ "/bin/fastqrab" ];
              WorkingDir = "/work";
              User = "nobody";
            };
          };
        packages.check = naersk-lib.buildPackage {
          src = ./.;
          mode = "check";
          name = "fastqrab";
          nativeBuildInputs = with pkgs; [
            pkg-config
            cmake
            gcc
            gnumake
            zstd.bin
            mold
          ];
          buildInputs = with pkgs; [ openssl ];
        };
        packages.test = naersk-lib.buildPackage {
          # not using naersk test mode, it eats the binaries, we need that binary
          pname = "fastqrab";
          root = ./.;
          nativeBuildInputs = with pkgs; [
            pkg-config
            cmake
            gcc
            gnumake
            python3
            rapidgzip
            which
            mold
            pkgs.shellcheck
          ];
          buildInputs = with pkgs; [ openssl ];
          release = true;
          CARGO_PROFILE_RELEASE_debug = "0";
          COMMIT_HASH = self.rev or (pkgs.lib.removeSuffix "-dirty" self.dirtyRev or "unknown-not-in-git");
          NIX_RAPIDGZIP_ = "${pkgs.rapidgzip}/bin/rapidgzip"; # note the _, it's special cased.
          RUST_LOG = "trace";
          # every other test happens wit hteh rapidgzip in the path.
          postInstall = ''
            # run the friendly panic test, expect a non 0 return code.
            # capture stderr

            result=$( { cargo run --release --bin fastqrab -- --test-friendly-panic 1>/dev/null; } 2>&1 ) || status=$? : "${"status:=0"}"
            if [ "$status" -eq 0 ]; then
              echo "Unexpected success"
              exit 1
            fi
            if [[ ! $result =~ "this is embarrassing" ]]; then
                echo "Error: friendly panic message ' not found in stderr"
                exit 1
            fi
            # without NIX_RAPIDGZIP, the test passes because the error is thrown
            echo 'without NIX_RAPIDGZIP'
            cargo test --release 
            # but with NIX_RAPIDGZIP, the test fails because there is a fallback

            echo 'with NIX_RAPIDGZIP'
            set +e  # Temporarily disable exit-on-error
            NIX_RAPIDGZIP=$NIX_RAPIDGZIP_ cargo test --release error_no_rapid_gzip
            set -e  # Re-enable exit-on-error
            if [ "$status" -eq 0 ]; then
              echo "Unexpected success when testing no-rapid-gzip-error-case"
              exit 1
            fi

          '';

        };

        defaultPackage = packages.fastqrab;

        # `nix run`
        apps.fastqrab = utils.lib.mkApp { drv = packages.fastqrab; };
        defaultApp = apps.fastqrab;

        # `nix develop`
        devShell = pkgs.mkShell {
          COMMIT_HASH = self.rev or (pkgs.lib.removeSuffix "-dirty" self.dirtyRev or "unknown-not-in-git");
          # we only link with mold in our dev environment for build speed. CI can use the old school rust linker
          shellHook = ''
            export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
            # Set shell for cmake builds
            export CONFIG_SHELL="${pkgs.bash}/bin/bash"
            export SHELL="${pkgs.bash}/bin/bash"
          '';
          # supply the specific rust version
          nativeBuildInputs = [
            bacon
            pkgs.bash
            pkgs.aflplusplus
            cargo-afl
            pkgs.cargo-audit
            pkgs.cargo-bloat
            pkgs.cargo-crev
            pkgs.cargo-deny
            pkgs.cargo-features-manager
            pkgs.cargo-flamegraph
            pkgs.cargo-insta
            pkgs.cargo-license
            pkgs.cargo-llvm-cov
            pkgs.cargo-llvm-lines
            pkgs.lcov
            pkgs.cargo-machete
            pkgs.cargo-mutants
            pkgs.cargo-nextest
            pkgs.cargo-outdated
            pkgs.cargo-shear
            #pkgs.cargo-udeps
            pkgs.cargo-vet
            pkgs.cmake
            pkgs.gcc
            pkgs.gnumake
            pkgs.git
            pkgs.hugo
            pkgs.jq
            pkgs.mold
            pkgs.openssl
            pkgs.pkg-config
            pkgs.samply
            (pkgs.python3.withPackages (
              ps: with ps; [
                scipy
                pysam
                pandas
                toml
              ]
            ))
            pkgs.rapidgzip
            pkgs.which
            pkgs.ripgrep
            #rust.rust-analyzer
            pkgs.shellcheck
            rust
          ];
        };
      }
    );
}
# {
