# challenge.toml reference

Template `challenge.toml` with all options:
```toml
id = "my-cool-challenge"
name = "My cool challenge"
author = "You"
description = "A cool description"
flag = "flag{a-cool-flag}"
# flag.file = "flag.txt"
# visible = true
# group = "some-group"
# build-group = "some-build-group"
category = "pwn"
# set min=max for static scoring
points.min = 100
points.max = 500

strategy = "instanced"
# strategy = "static"

# host = "default"

# provide = ["file.txt"]

[[provide]]
file = "file.txt"
as = "attachment.txt"

[[provide]]
dir = "dist/"
# without archive extension
as = "chall"
exclude = ["flag.txt"]

[container.default]
# build context dir for docker
build = "."
limits.cpu = 1000000000 # units of 10^-9 cpu
limits.mem = 1048576 # bytes
# privileged = false
env.VAR = "VALUE"

[container.default.expose]
5000 = "http"
4242 = "tcp"
```

## Reference

`id` - The ID of the challenge. This should be a slug, that is, only `[a-zA-Z0-9\-]+`

`name` - The name of the challenge, visible to users.

`author` - The author of the challenge, visible to users.

`description` - Description/flavor text for the challenge. Markdown is supported.

`flag` - The flag for the challenge. This can either be a string (static flag), or a dictionary with a `file` entry containing the path to the flag file. Example:
```toml
# Static flag
flag = "flag{a-flag}"

# File flag
flag.file = "flag.txt"
```

`visible` - (optional) Whether this challenge is visible to users. Useful to build challenge images ahead of time.

`group` - (optional) ??? idr

`build_group` - (optional) ??? idr

`category` - The category of this challenge, visible to users (and filterable, so be consistent.)

`points` - Range of points for dynamic scoring, from `max` points down to `min`. Example:
```toml
points.min = 100
points.max = 500

# For static scoring, set them to the same value:
points.min = 500
points.max = 500
```

`provide` - A list of the attachments that will be provided to users as handouts. Each attachment is either a file or an archive. Example:
```toml
# Identity-named file: just a string (deprecated)
provide = ["file.txt"]

# Renamed file:
[[provide]]
file = "local-name.txt"
as = "handout-name.txt"

# Archive of a directory:
[[provide]]
dir = "./my-dir"
as = "chall" # without the archive extension
exclude = ["./flag/**"] # list of wildcards to exclude
```

`container` - A dictionary of the containers to run for this challenge. Each container is built from a directory and can have exposed TCP or HTTP ports. Example:
```toml
[container.default]
# Docker context dir for build, should contain a Dockerfile
build = "./backend"
# Container limits:
# cpu - "nano-cpus", in units of 10^-9 cpu
# mem - in bytes
limits = { cpu = 100000000, mem = 1048576 }
# Whether this should be a privileged container (dangerous!)
# Useful for redpwn jails (but you really should just use instancing)
privileged = false
# Ports to expose
# HTTP ports get automatic HTTPS and subdomain mapping
# TCP ports get a randomized port assignment
expose = { 3000 = "tcp", 9876 = "http" }
```

`strategy` - The deployment strategy to use for this challenge. Can be either `static` (default, one instance for all users) or `instanced` (one instance per user).

`host` - The host that this challenge should be deployed on. Defaults to `default`. See keychain docs.
