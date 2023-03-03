{
  description = "carveout";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
          (self: super: {
            rust-toolchain = self.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          })
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        libPath = with pkgs; lib.makeLibraryPath [
          libxkbcommon

          # wayland
          wayland

          # X11
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr

          vulkan-loader
          vulkan-validation-layers
        ];

      in
      with pkgs;
      {
        formatter = nixpkgs-fmt;

        devShell = mkShell {
          buildInputs = [
            pkgconfig
            rust-toolchain
            rust-analyzer
            bacon
            cargo-edit
            trunk

            xorg.libxcb
          ];

          LD_LIBRARY_PATH = libPath;
          VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
        };

        defaultPackage = rustPlatform.buildRustPackage {
          name = "carveout";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "encase-0.4.1" = "sha256-xo25yCYPw1y/CbKTmWOyBcTGlijyy9ImMNARc6nn7w0=";
            };
          };

          nativeBuildInputs = [
            rust-toolchain
            makeWrapper
          ];
          buildInputs = [
            xorg.libxcb
          ];
          postInstall = ''
            wrapProgram "$out/bin/carveout" --prefix LD_LIBRARY_PATH : "${libPath}"
          '';
        };
      }
    );
}
