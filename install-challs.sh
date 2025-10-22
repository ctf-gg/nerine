#!/bin/sh

set -e

do_install() {
    echo "Running nerine challenges server install script." 
    if [ ! "$(id -u)" = 0 ]; then
	echo "ERROR: You must run this script as root."
	exit 1
    fi

    if [ ! -x "$(command -v curl)" ]; then
	echo "ERROR: curl is not available. You must have curl to install nerine."
	exit 1
    fi    

    echo "Installing dependencies."
    
    if [ ! -x "$(command -v docker)" ]; then
	curl -fsS https://get.docker.com | sh
    fi

    if [ ! -x "$(command -v go)" ]; then
	# TODO(aiden): arm64 -> x86_64 issue
	curl -SL "https://go.dev/dl/go1.25.3.linux-amd64.tar.gz" -o ./go.tar.gz
	rm -rf /usr/local/go && tar -C /usr/local -xzf go.tar.gz
	export PATH=$PATH:/usr/local/go/bin
	chmod -R a+rx /usr/local/go/bin
	chmod a+r /usr/local/go/
	echo 'export PATH=$PATH:/usr/local/go/bin' >> /etc/profile
	rm go.tar.gz
    fi

    read -p "Where is the keys archive (default keys.tar.gz)? " -r keys_path </dev/tty 
    keys_path="${keys_path:-keys.tar.gz}"
    tar -xzf "$keys_path"

    mkdir -p /var/docker /etc/docker
    cp keys/docker/server-key.pem keys/docker/server-cert.pem keys/docker/ca.pem /var/docker

    cat <<EOF > /etc/docker/daemon.json
{
  "hosts": ["tcp://127.0.0.1:996", "unix:///var/run/docker.sock"],
  "tls": true,
  "tlscacert": "/var/docker/ca.pem",
  "tlscert": "/var/docker/server-cert.pem",
  "tlskey": "/var/docker/server-key.pem",
  "tlsverify": true
}
EOF
    
    sed -i 's/-H fd:\/\/ //' /lib/systemd/system/docker.service
  
    systemctl daemon-reload
    systemctl restart docker

    echo "Docker ready."

    echo "Setting up caddy." 

    NERINE_GIT_REF="${NERINE_GIT_REF:-main}"

    if [ ! -x "$(command -v caddy)" ]; then
	prev_dir="$(pwd)"
	router_temp_dir="$(mktemp -d)"
	cd $router_temp_dir
	curl -SL "https://github.com/caddyserver/xcaddy/releases/download/v0.4.5/xcaddy_0.4.5_linux_amd64.tar.gz" -o xcaddy.tar.gz
	tar -xzf xcaddy.tar.gz xcaddy
	mkdir pkg
	curl -SL "https://raw.githubusercontent.com/ctf-gg/nerine/$NERINE_GIT_REF/caddyrouter/dynamicrouter.go" -o pkg/dynamicrouter.go
	curl -SL "https://raw.githubusercontent.com/ctf-gg/nerine/$NERINE_GIT_REF/caddyrouter/go.mod" -o pkg/go.mod
	curl -SL "https://raw.githubusercontent.com/ctf-gg/nerine/$NERINE_GIT_REF/caddyrouter/go.sum" -o pkg/go.sum
	

	./xcaddy build --with github.com/ctf-gg/nerine=./pkg/
	mv caddy /usr/bin/
	groupadd --system caddy
	useradd --system \
		--gid caddy \
		--create-home \
		--home-dir /var/lib/caddy \
		--shell /usr/sbin/nologin \
		--comment "Caddy web server" \
		caddy

	mkdir -p /etc/caddy/
	cd $prev_dir
	rm -r $router_temp_dir
    fi
    cp keys/caddy/cert.pem keys/caddy/ca.pem keys/caddy/ca-key.pem /var/lib/caddy/
    chown -R caddy:caddy /var/lib/caddy
    read_pem_json() {
	<"$1" sed '1d;$d' | sed 's/\\/\\\\/g; s/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g'
    }
    
    read -p "What hostname will challenges be hosted on (e.g., challs.example.com)? " -r challs_hostname </dev/tty
    cat <<EOF > /etc/caddy/config.json
{
  "admin": {
    "listen": "localhost:990",
    "remote": {
      "listen": "0.0.0.0:995",
      "access_control": [{
        "public_keys": ["$(read_pem_json /var/lib/caddy/cert.pem)"]
      }]
    },
    "identity": {
      "identifiers": ["$(hostname -i)", "$(curl -s ifconfig.me)", "127.0.0.1"],
      "issuers": [{
        "module": "internal",
        "ca": "local-admin",
        "sign_with_root": true
      }]
    }
  },
  "apps": {
    "http": {
      "servers": {
        "srv0": {
          "@id": "default-server",
          "automatic_https": {
            "disable": true
          },
          "listen": [
            ":80"
          ],
          "routes": [
            {
              "match": [{
                "host": ["*.$challs_hostname"]
              }],
              "handle": [
                {
                  "handler": "dynamic_router"
                },
                {
                  "handler": "reverse_proxy",
                  "upstreams": [{
                    "dial": "{http.vars.dynamic.upstream}"
                  }]
                }
              ]
            }
          ]
        }
      }
    },
    "pki": {
      "certificate_authorities": {
        "local-admin": {
          "name": "local-admin",
          "install_trust": false,
          "root": {
            "certificate": "/var/lib/caddy/ca.pem",
            "private_key": "/var/lib/caddy/ca-key.pem"
          }
        }
      }
    }
  }
}
EOF

    cat <<EOF > /etc/systemd/system/caddy.service
[Unit]
Description=Caddy
Documentation=https://caddyserver.com/docs/
After=network.target network-online.target
Requires=network-online.target

[Service]
Type=notify
User=caddy
Group=caddy
ExecStart=/usr/bin/caddy run --environ --config /etc/caddy/config.json
ExecReload=/usr/bin/caddy reload --config /etc/caddy/config.json --force
TimeoutStopSec=5s
LimitNOFILE=1048576
PrivateTmp=true
ProtectSystem=full
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_BIND_SERVICE

[Install]
WantedBy=multi-user.target
EOF
    systemctl daemon-reload
    systemctl enable --now caddy
    echo "Caddy ready."
}

do_install
