#!/bin/sh

set -e

get_key() {
    head -c 32 /dev/urandom | base64 -w 0
}

setup_keychain() {
  echo "Creating keys."
  mkdir keys
  cd keys
  
  mkdir docker
  cd docker
  # Create CA
  openssl genrsa -out ca-key.pem 4096
  openssl req -new -x509 -days 365 -key ca-key.pem -sha256 -out ca.pem <<EOF 2> /dev/null
.
.
.
.
.
$1
.
EOF
  
  # Generate server key & cert signing request
  openssl genrsa -out server-key.pem 4096
  openssl req -subj "/CN=docker" -sha256 -new -key server-key.pem -out server.csr
  cat >extfile.cnf <<EOF
subjectAltName = IP:$(hostname -i),IP:$(curl -s ifconfig.me),IP:127.0.0.1
extendedKeyUsage = serverAuth
EOF

  # Sign server cert
  openssl x509 -req -days 365 -sha256 -in server.csr -CA ca.pem -CAkey ca-key.pem -CAcreateserial -out server-cert.pem -extfile extfile.cnf

  # Create client key & cert signing request
  openssl genrsa -out key.pem 4096
  openssl req -subj '/CN=client' -new -key key.pem -out client.csr
  cat >extfile-client.cnf <<EOF
extendedKeyUsage = clientAuth
EOF

  # Sign client cert
  openssl x509 -req -days 365 -sha256 -in client.csr -CA ca.pem -CAkey ca-key.pem -CAcreateserial -out cert.pem -extfile extfile-client.cnf

  ########################
  ### Repeat for caddy ###
  ########################

  cd ..
  mkdir caddy
  cd caddy
  # Create CA
  openssl genrsa -out ca-key.pem 4096
  openssl req -new -x509 -days 365 -key ca-key.pem -sha256 -out ca.pem <<EOF 2> /dev/null
.
.
.
.
.
$1
.
EOF
  
  # Generate server key & cert signing request
  openssl genrsa -out server-key.pem 4096
  openssl req -subj "/CN=caddy" -sha256 -new -key server-key.pem -out server.csr
  cat >extfile.cnf <<EOF
subjectAltName = IP:$(hostname -i),IP:$(curl -s ifconfig.me),IP:127.0.0.1
extendedKeyUsage = serverAuth
EOF

  # Sign server cert
  openssl x509 -req -days 365 -sha256 -in server.csr -CA ca.pem -CAkey ca-key.pem -CAcreateserial -out server-cert.pem -extfile extfile.cnf

  # Create client key & cert signing request
  openssl genrsa -out key.pem 4096
  openssl req -subj '/CN=client' -new -key key.pem -out client.csr
  cat >extfile-client.cnf <<EOF
extendedKeyUsage = clientAuth
EOF

  # Sign client cert
  openssl x509 -req -days 365 -sha256 -in client.csr -CA ca.pem -CAkey ca-key.pem -CAcreateserial -out cert.pem -extfile extfile-client.cnf

  echo "Created keys and certs"
  cd ../..
  tar -czf keys.tar.gz keys

  read_pem_json() {
    <"$1" sed 's/\\/\\\\/g; s/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g'
  }
  

  cd keys
  read -p "What hostname will challenges be hosted on (e.g., challs.example.com)? " -r challs_hostname </dev/tty
  
  challs_ip=""
  if [ -n "$challs_hostname" ]; then
    resolved_ip=$(getent hosts "$challs_hostname" | awk '{print $1}' | head -n 1)
    
    if [ -n "$resolved_ip" ]; then
      echo "Hostname '$challs_hostname' resolved to IP: $resolved_ip"
      challs_ip="$resolved_ip"
    else
      echo "WARN: Could not resolve IP for hostname '$challs_hostname'."
    fi
  fi
  
  if [ -z "$challs_ip" ]; then
    read -p "Please enter the challenges IP address directly: " -r direct_ip </dev/tty
    challs_ip="$direct_ip"
  fi

  challs_ip="${challs_ip:-'<insert-chall-ip>'}"


  cat <<EOF > ../keychain.json
[{
  "id": "default",
  "caddy": {
    "endpoint": "https://$challs_ip:995",
    "base": "$challs_hostname",
    "cacert": "$(read_pem_json caddy/ca.pem)",
    "cert": "$(read_pem_json caddy/client-cert.pem)",
    "key": "$(read_pem_json caddy/client-key.pem)"
  },
  "docker": {
    "docker": {
      "type": "ssl",
      "address": "$challs_ip:996",
      "ca": "$(read_pem_json docker/ca.pem)",
      "cert": "$(read_pem_json docker/client-cert.pem)",
      "key": "$(read_pem_json docker/client-key.pem)"
    },
    "docker_credentials": {
      "username": "<docker-registry-username>",
      "password": "<docker-registry-password>",
      "serveraddress": "<docker-registry-address>",
      "image_prefix": "",
      "repo": "<docker-registry-repo>"
    }
  }
}]
EOF
  cd ..
  rm -r keys

  echo "Keychain created. Next steps:"
  echo "... Copy keys.tar.gz (in install dir) to your challenges VM and run install-challenges.sh"
  echo "... Edit keychain.json (in install dir) to add docker registry credentials."
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
	echo "Would you like to setup deployer keychain (y/N)? "
	read -r result </dev/tty
	

	if [ "$result" = "n" ] || [ "$result" = "N" ]; then
	    exit 1
	fi

	# This can be read from the installation but lazy
	read -p "What host will nerine be hosted at (used for CA CN)? " -r nerine_url </dev/tty
	nerine_url="${nerine_url##*://}"
	setup_keychain "$nerine_url"
	return
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

    mkdir -p site-assets
    
    NERINE_POSTGRES_PASSWORD=$(get_key)

    read -p "What host will nerine be hosted at? " -r nerine_url </dev/tty
    nerine_url="${nerine_url##*://}"

    echo "Generating configuration."

    NERINE_ADMIN_TOKEN="${NERINE_ADMIN_TOKEN:-$(get_key)}"

    printf "%s\n" \
	   "RUST_LOG=debug" \
	   "CORS_ORIGIN=https://${nerine_url}" \
	   "NERINE_POSTGRES_PASSWORD=${NERINE_POSTGRES_PASSWORD}" \
	   "DATABASE_URL=postgres://nerine:${NERINE_POSTGRES_PASSWORD}@db/nerine" \
	   "ADMIN_TOKEN=${NERINE_ADMIN_TOKEN}" \
	   "JWT_SECRET=$(get_key)" \
	   > .env

    printf "%s\n" \
	   "name = \"nerineCTF\"" \
	   "description = \"\"\"
Write markdown here for the front page!
You can put things like sponsor logos in \`site-assets\`, then access them at \`https://${nerine_url}/assets/\`
\"\"\"" \
	   "start_time = \"$(date +%FT%T.000)\"" \
	   "end_time = \"$(date -d +1week +%FT%T.000)\"" \
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

  echo "Pulling Images."
  curl -fsSo docker-compose.yml "https://raw.githubusercontent.com/ctf-gg/nerine/$NERINE_GIT_REF/docker-compose.prod.yml"
  docker compose pull

  read -p "Would you like to setup deployer keychain (Y/n)? " -r result </dev/tty
  if [ "$result" = "n" ] || [ "$result" = "N" ]; then
      return
  fi

  setup_keychain "$nerine_url"
}


do_install
echo "Finished installation to $NERINE_INSTALL_PATH."
echo "... Your admin token is: $NERINE_ADMIN_TOKEN. It can also be found in $NERINE_INSTALL_PATH/.env"
echo "... Configuration files can be found in $NERINE_INSTALL_PATH."
echo "... If you would like to start nerine, run \`docker compose up -d\` in $NERINE_INSTALL_PATH."
