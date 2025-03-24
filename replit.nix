
{ pkgs }: {
  deps = [
    pkgs.rustc
    pkgs.cargo
    pkgs.rustfmt
    pkgs.clippy
    pkgs.openssl
    pkgs.pkg-config
    pkgs.openssl.dev
  ];
}
