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
            pkgconfig
                
            vulkan-loader
            vulkan-validation-layers
            libxkbcommon
            wayland
                
            freetype
            expat
            fontconfig
            gdk-pixbuf
            gtk3
            gsettings-desktop-schemas

            rust-toolchain
            rust-analyzer
            bacon
          ];
          VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";                
          XDG_DATA_DIRS= "${gsettings-desktop-schemas}/share/gsettings-schemas/${gsettings-desktop-schemas.name}:${gtk3}/share/gsettings-schemas/${gtk3.name}";
        };
      }
    );
}
