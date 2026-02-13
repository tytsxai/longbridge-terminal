#!/usr/bin/env sh
set -eu

printf '%s\n' "[1/10] 检查脚本语法"
sh -n install
sh -n scripts/release_preflight.sh
sh -n scripts/log_alert_guard.sh

printf '%s\n' "[2/10] 代码格式检查"
cargo fmt --all -- --check

printf '%s\n' "[3/10] 中文化守门检查"
python3 scripts/check_i18n_guard.py

printf '%s\n' "[4/10] 静态分析（clippy）"
cargo clippy -- -D warnings

printf '%s\n' "[5/10] 单元测试"
cargo test

printf '%s\n' "[6/10] 编译检查"
cargo check

printf '%s\n' "[7/10] CLI 非交互冒烟"
cargo run -- --help > /dev/null
cargo run -- --version > /dev/null

printf '%s\n' "[8/10] doctor 诊断链路冒烟"
LONGPORT_APP_KEY=dummy \
LONGPORT_APP_SECRET=dummy \
LONGPORT_ACCESS_TOKEN=dummy \
cargo run -- doctor > /dev/null

printf '%s\n' "[9/10] Release 构建检查"
cargo build --release

printf '%s\n' "[10/10] 依赖安全审计"
./scripts/cargo_audit.sh

printf '%s\n' "✅ Preflight 全部通过"
