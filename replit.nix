
{ pkgs }: {
  deps = [
    pkgs.hex
    pkgs.bitcoin
    pkgs.rustc
    pkgs.cargo
    pkgs.rustfmt
    pkgs.clippy
    pkgs.openssl
    pkgs.pkg-config
    pkgs.openssl.dev
  ];
}
