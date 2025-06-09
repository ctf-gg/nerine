{ pkgs ? import <nixpkgs> {} }: pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
    sqlx-cli
    (python3.withPackages (ps: with ps; [
      psycopg2
      python-dotenv
      tqdm
    ]))
    nodejs
    pnpm
    nginx
    postgresql
    pkg-config
    openssl.dev
    # for dev
    expect
  ];

  DATABASE_URL = "postgres://postgres:postgres@localhost/sctf";
  # example_secret in base64
  JWT_SECRET = "ZXhhbXBsZV9zZWNyZXQK";
  ADMIN_TOKEN = "example_admin_token";
}
