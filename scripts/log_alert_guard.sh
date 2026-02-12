#!/usr/bin/env sh
set -eu

# ==================== 参数（可通过环境变量覆盖） ====================
WINDOW_MINUTES="${WINDOW_MINUTES:-5}"
ALERT_THRESHOLD="${ALERT_THRESHOLD:-20}"
TAIL_LINES="${TAIL_LINES:-2000}"
ALERT_KEYWORDS="${ALERT_KEYWORDS:-Failed to fetch|Rate limit error|配置错误|Configuration Error|获取资产数据超时|获取资产数据失败，且当前无可回退数据}"

# ==================== 日志目录探测 ====================
default_log_dir() {
  case "$(uname -s 2>/dev/null || echo unknown)" in
    Darwin)
      printf '%s\n' "$HOME/Library/Logs/ChangQiao"
      ;;
    Linux)
      printf '%s\n' "${XDG_DATA_HOME:-$HOME/.local/share}/changqiao/logs"
      ;;
    *)
      printf '%s\n' "${TMPDIR:-/tmp}/changqiao/logs"
      ;;
  esac
}

find_latest_log_file() {
  dir="$1"
  if [ ! -d "$dir" ]; then
    return 1
  fi

  # shellcheck disable=SC2012
  ls -1t "$dir"/changqiao*.log "$dir"/longbridge*.log 2>/dev/null | head -n 1
}

log_dir="${LOG_DIR:-$(default_log_dir)}"
latest_log="$(find_latest_log_file "$log_dir" || true)"

if [ -z "$latest_log" ]; then
  fallback_dir="${TMPDIR:-/tmp}/changqiao/logs"
  latest_log="$(find_latest_log_file "$fallback_dir" || true)"
  if [ -n "$latest_log" ]; then
    log_dir="$fallback_dir"
  fi
fi

if [ -z "$latest_log" ]; then
  echo "WARN: 未找到日志文件，跳过告警扫描。"
  exit 0
fi

export WINDOW_MINUTES ALERT_THRESHOLD ALERT_KEYWORDS TAIL_LINES latest_log

# ==================== 统计窗口内关键错误次数 ====================
python3 - <<'PY'
from __future__ import annotations

import os
import sys
from collections import deque
from datetime import datetime, timedelta, timezone

window_minutes = int(os.environ["WINDOW_MINUTES"])
alert_threshold = int(os.environ["ALERT_THRESHOLD"])
tail_lines = int(os.environ["TAIL_LINES"])
alert_keywords = [k.strip() for k in os.environ["ALERT_KEYWORDS"].split("|") if k.strip()]
latest_log = os.environ["latest_log"]

cutoff = datetime.now(timezone.utc) - timedelta(minutes=window_minutes)
count = 0

try:
    with open(latest_log, "r", encoding="utf-8", errors="replace") as file:
        lines = deque(file, maxlen=max(tail_lines, 1))
except OSError as err:
    print(f"WARN: 读取日志失败（{latest_log}）：{err}")
    sys.exit(0)

for raw in lines:
    line = raw.rstrip("\n")
    if not line:
        continue
    if not any(keyword in line for keyword in alert_keywords):
        continue

    # tracing 默认 RFC3339 时间戳在行首；解析失败时按“保守计数”处理
    token = line.split(" ", 1)[0]
    try:
        ts = datetime.fromisoformat(token.replace("Z", "+00:00"))
        if ts.tzinfo is None:
            ts = ts.replace(tzinfo=timezone.utc)
        if ts >= cutoff:
            count += 1
    except Exception:
        count += 1

if count >= alert_threshold:
    print(
        f"ALERT: {window_minutes} 分钟内关键错误 {count} 次（阈值 {alert_threshold}），日志文件：{latest_log}"
    )
    sys.exit(2)

print(
    f"OK: {window_minutes} 分钟内关键错误 {count} 次（阈值 {alert_threshold}），日志文件：{latest_log}"
)
PY
