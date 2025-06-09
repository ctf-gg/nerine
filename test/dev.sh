#!/usr/bin/env bash

sudo -v || exit 1
command -v expect &>/dev/null || (echo "missing expect" && exit 1)

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

# create some fds
# https://superuser.com/a/633185
tail -f /dev/null | tail -f /dev/null & PID2=$!; PID1=$(jobs -p %+); exec 5>/proc/$PID1/fd/1 6</proc/$PID2/fd/0; disown $PID2; kill $PID1 $PID2
tail -f /dev/null | tail -f /dev/null & PID2=$!; PID1=$(jobs -p %+); exec 7>/proc/$PID1/fd/1 8</proc/$PID2/fd/0; disown $PID2; kill $PID1 $PID2
tail -f /dev/null | tail -f /dev/null & PID2=$!; PID1=$(jobs -p %+); exec 9>/proc/$PID1/fd/1 10</proc/$PID2/fd/0; disown $PID2; kill $PID1 $PID2

RUST_LOG=debug unbuffer ./target/debug/sctf-api >&5 2>&5 &
(cd frontend && unbuffer pnpm dev) >&7 2>&7 0</dev/null &
sudo -n nginx -c $PWD/test/nginx.conf >&9 2>&9 &
while read -r line; do printf "[\033[35msctf \033[m] %s\n" "${line}"; done <&6 &
while read -r line; do printf "[\033[34mastro\033[m] %s\n" "${line}"; done <&8 &
while read -r line; do printf "[\033[1;30mnginx\033[m] %s\n" "${line}"; done <&10 &
wait
