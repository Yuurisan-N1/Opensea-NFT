#!/usr/bin/env bash
set -euo pipefail

NAME="OpenSea-Mint"
BIN="opensea-mint-bot"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${ROOT}/dist"

rm -rf "${OUT}"
mkdir -p "${OUT}/${NAME}"

cd "${ROOT}"
cargo build --release

mkdir -p "${OUT}/${NAME}/src"
cp -f Cargo.toml Cargo.lock build.rs Makefile run.sh LICENSE README.md config.json .env .env.example .gitignore "${OUT}/${NAME}/"
cp -f proxy.txt "${OUT}/${NAME}/"
cp -rf assets .github scripts src "${OUT}/${NAME}/"

find "${OUT}/${NAME}" -name 'target' -prune -exec rm -rf {} + 2>/dev/null || true
find "${OUT}/${NAME}" -name '*.rs.bk' -delete 2>/dev/null || true

cd "${OUT}"
zip -qr "${ROOT}/${NAME}.zip" "${NAME}"
ls -l "${ROOT}/${NAME}.zip"
