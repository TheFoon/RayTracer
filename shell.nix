{ pkgs ? import <nixpkgs> { } }:
let
  libPath = with pkgs; lib.makeLibraryPath [
    libGL
    libxkbcommon
    wayland
  ];
in
with pkgs; mkShell {
  inputsFrom = [];
  buildInputs = [ pkg-config ];
  LD_LIBRARY_PATH = "${libPath}";
}
