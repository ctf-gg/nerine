{ pkgs ? import <nixpkgs> {} }: pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
    sqlx-cli
  ];

  DATABASE_URL = "postgres://postgres:postgres@localhost/sctf";
}
