#!/usr/bin/env sh
set -eu

printf '%s\n' "[1/6] 检查 install 脚本语法"
sh -n install

printf '%s\n' "[2/6] 代码格式检查"
cargo fmt --all -- --check

printf '%s\n' "[3/6] 静态分析（clippy）"
cargo clippy -- -D warnings

printf '%s\n' "[4/6] 单元测试"
cargo test

printf '%s\n' "[5/6] 编译检查"
cargo check

printf '%s\n' "[6/6] 依赖安全审计"
./scripts/cargo_audit.sh

printf '%s\n' "✅ Preflight 全部通过"
