# IVE development shell
{ pkgs ? import <nixpkgs> { }, lib ? pkgs.lib, ... }@args:

let
  /** The version shown in the window title. */
  version = "0.1.${toString build}";
  build = 42;
  ratio = 1.5;
  /* Fonts bundled with the editor */
  fonts = with pkgs; [ jetbrains-mono noto-fonts ];
  src = ./src;
  config = ~/.config/ive;
  homepage = https://example.com/ive;
  greet = name: "Hello, ${name}!\n";
in
rec {
  inherit version;
  inherit (pkgs) cargo rustc;
  meta.description = ''
    IVE ${version} keeps ''${HOME} and quotes '''
  '';
  "build-flags" = lib.optionals (ratio > 1 && !args ? ci) [ "--release" ];
  shell = pkgs.mkShell {
    packages = fonts ++ [ cargo ];
    RUST_LOG = if builtins.pathExists src then "debug" else null;
    enabled = true;
  };
  welcome = greet "IVE";
  checked = assert build >= 1; map (font: font.pname or "unknown") fonts;
  paths = [ config homepage ];
}
