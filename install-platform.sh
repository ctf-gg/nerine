#!/bin/sh

set -e

get_key() {
    head -c 32 /dev/urandom | base64 -w 0
}

do_install() {
    echo "Running nerine install script."
    if [ ! "$(id -u)" = 0 ]; then
	echo "ERROR: You must run this script as root."
	exit 1
    fi

    if [ ! -x "$(command -v curl)" ]; then
	echo "ERROR: curl is not available. You must have curl to install nerine."
	exit 1
    fi

    NERINE_INSTALL_PATH="${NERINE_INSTALL_PATH:-/srv/nerine}"

    if [ -d "$NERINE_INSTALL_PATH" ]; then
	echo "nerine appears to already have been installed in ${NERINE_INSTALL_PATH}"
	exit 1
    fi

    mkdir "$NERINE_INSTALL_PATH"
    cd "$NERINE_INSTALL_PATH"

    echo "Installing dependencies."
    
    if [ ! -x "$(command -v docker)" ]; then
	curl -fsS https://get.docker.com | sh
    fi

    DOCKER_CONFIG=${DOCKER_CONFIG:-/usr/local/lib/docker}
    if [ ! -f $DOCKER_CONFIG/cli-plugins/docker-compose ]; then
	mkdir -p $DOCKER_CONFIG/cli-plugins
	curl -SL "https://github.com/docker/compose/releases/download/v2.40.1/docker-compose-$(uname -s)-$(uname -m)" -o $DOCKER_CONFIG/cli-plugins/docker-compose
	chmod +x $DOCKER_CONFIG/cli-plugins/docker-compose
    fi

    mkdir -p data/postgres site-assets
    
    NERINE_POSTGRES_PASSWORD=$(get_key)

    read -p "What host will nerine be hosted at? " -r nerine_url </dev/tty
    nerine_url="${nerine_url##*://}"

    echo "Generating configuration"

    NERINE_ADMIN_TOKEN="${NERINE_ADMIN_TOKEN:-$(get_key)}"

    printf "%s\n" \
	   "RUST_LOG=debug" \
	   "CORS_ORIGIN=https://${nerine_url}" \
	   "NERINE_POSTGRES_PASSWORD=${NERINE_POSTGRES_PASSWORD}" \
	   "DATABASE_URL=postgres://nerine:${NERINE_POSTGRES_PASSWORD}@db/nerine" \
	   "ADMIN_TOKEN=${ADMIN_TOKEN}" \
	   "JWT_SECRET=$(get_key)" \
	   > .env

    printf "%s\n" \
	   "name = \"nerineCTF\"" \
	   "description = \"\"\"
Write markdown here for the front page!
You can put things like sponsor logos in \`site-assets\`, then access them at \`https://${nerine_url}/assets/\`
\"\"\"" \
	   "start_time = \"$(date -Iseconds)\"" \
	   "end_time = \"$(date -d +1week -Iseconds)\"" \
	   > event.toml

    mkdir caddy
    cat <<EOF > caddy/Caddyfile
https://${nerine_url:-"<insert-platform-url>"} {
        reverse_proxy /api/* localhost:3333
        reverse_proxy /* localhost:3334

        log {
                output file /var/log/caddy/access.log {
                        roll_size 1gb
                        roll_keep 20
                        roll_keep_for 720h
                }
        }
}
EOF

  NERINE_GIT_REF="${NERINE_GIT_REF:-main}"

  curl -fsSo docker-compose.yml "https://raw.githubusercontent.com/ctf-gg/nerine/$NERINE_GIT_REF/docker-compose.prod.yml"
  docker compose pull

  echo "Finished installation to $NERINE_INSTALL_PATH."
  echo "... Your admin token is: $NERINE_ADMIN_TOKEN. It can also be found in $NERINE_INSTALL_PATH/.env"
  echo "... Configuration files can be found in $NERINE_INSTALL_PATH."
  echo "... If you would like to start nerine, run \`docker compose up -d\` in $NERINE_INSTALL_PATH."
}

do_install
