#!/usr/bin/env sh
set -eu

if ! cargo audit --version > /dev/null 2>&1; then
  echo "未检测到 cargo-audit，正在安装..."
  cargo install cargo-audit --locked
fi

cargo audit --deny warnings \
  --ignore RUSTSEC-2021-0141 \
  --ignore RUSTSEC-2024-0384 \
  --ignore RUSTSEC-2024-0436 \
  --ignore RUSTSEC-2024-0320 \
  --ignore RUSTSEC-2026-0002
