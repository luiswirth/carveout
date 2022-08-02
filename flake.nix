{
  description = "arcus";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
      in
      with pkgs;
      {
        devShells.default = mkShell rec {
          buildInputs = [
            vulkan-loader
            vulkan-validation-layers
            libxkbcommon
            wayland
                
            freetype
            expat
            fontconfig

            rust-toolchain
            rust-analyzer
            bacon
          ];
          VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";                
        };
      }
    );
}
