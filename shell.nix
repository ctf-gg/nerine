{ pkgs ? import <nixpkgs> {} }: pkgs.mkShell rec {
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
    caddy
    xcaddy
    go
    gopls
  ];

  DATABASE_URL = "postgres://postgres:postgres@localhost/sctf";
  # example_secret in base64
  JWT_SECRET = "ZXhhbXBsZV9zZWNyZXQK";
  ADMIN_TOKEN = "example_admin_token";

  PLATFORM_BASE = "http://sctf.localhost";
  PLATFORM_ADMIN_TOKEN = ADMIN_TOKEN;

  CHALLENGES_DIR = "./test-deploy/challenges";
  HOST_KEYCHAINS = "./keychain-dev-real-certs.json";
  DEPLOYER_BASE = "http://localhost:3001";
}
