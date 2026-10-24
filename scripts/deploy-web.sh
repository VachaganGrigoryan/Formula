#!/usr/bin/env bash
# Build the web version and publish it on https://formula.vachagan.dev (ZimaOS, see deploy/zimaos).
#   /         web/index.html (landing page)    /media/  docs/media
#   /app/     index.html + pkg/ (the workbench)
set -euo pipefail
cd "$(dirname "$0")/.."

host=zimaos
dir=/DATA/AppData/formula
site=target/site

cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir pkg target/wasm32-unknown-unknown/release/formula.wasm

rm -rf "$site"
mkdir -p "$site/app"
cp web/index.html "$site/"
cp -R docs/media "$site/media"
cp index.html "$site/app/"
cp -R pkg "$site/app/pkg"

rsync -a deploy/zimaos/ "$host:$dir/"
rsync -a --delete "$site/" "$host:$dir/site/"
ssh "$host" "cd $dir && DOCKER_CONFIG=/DATA/AppData/docker-config docker compose up -d"

curl -fsSI --retry 5 --retry-all-errors https://formula.vachagan.dev/app/ | head -1
