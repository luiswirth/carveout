# carveout

digital pen note taking

## Run / Install

### Executable from github release page

x86 Linux executables are provided in the github release page.

### git + cargo

Run with
```sh
git clone https://github.com/lu15w1r7h/carveout.git
cargo run --release
```

Install with
```sh
cargo install --git https://github.com/lu15w1r7h/carveout.git
```

### In the browser

[carveout.lwirth.com](https://carveout.lwirth.com)
using webassembly and webgpu

(still work in progress)

### Nix with flakes

Run with
```sh
nix run "github:lu15w1r7h/carveout"
```

Install with
```sh
nix profile install "github:lu15w1r7h/carveout"
```

## Build with

rust, nix, webgpu/wgpu, webassembly, egui, lyon, nalgebra, parry, pdfium

## Screenshots

![2022-09-18T003741](https://user-images.githubusercontent.com/37505890/190878534-bd9ab0bb-7881-4530-9631-fb6d0054cc4b.png)
![2022-09-15T201538](https://user-images.githubusercontent.com/37505890/190836448-2b8c5de3-fe56-480b-96d0-57332d232b6b.png)
![2022-08-30T160250](https://user-images.githubusercontent.com/37505890/190836463-ee67157a-742f-4163-a5c2-7226ff2e0134.png)
