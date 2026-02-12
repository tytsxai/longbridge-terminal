# 长桥终端 生产就绪清单（Ready）

> 目标：在不做大改的前提下，让项目达到“可交付、可上线、可维护”的最小健康状态。

## 1. 上线前必检（Go/No-Go）

### 1.1 构建与质量门槛

必须全部通过：

```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
cargo check
./scripts/release_preflight.sh
```

### 1.2 运行环境

- 仅支持交互式终端（TTY）运行。
- 运行账号需可写日志目录（失败时会自动降级到系统临时目录）。
- 网络可访问 长桥 OpenAPI 端点。

### 1.3 必需配置（环境变量）

- `LONGPORT_APP_KEY`
- `LONGPORT_APP_SECRET`
- `LONGPORT_ACCESS_TOKEN`

可选：

- `LONGPORT_REGION`（示例：`cn`）
- `CHANGQIAO_LOCALE`（示例：`zh-CN` / `en`）
- `CHANGQIAO_LOG`（示例：`error,changqiao=info`）
- 兼容旧变量：`LONGBRIDGE_LOCALE`、`LONGBRIDGE_LOG`
- `LONGPORT_HTTP_URL`、`LONGPORT_QUOTE_WS_URL`（私有化场景）

## 2. 关键稳定性机制（当前已具备）

### 2.0 启动与进程治理

- CLI 基础参数已可用：`--help`、`--version`、`--logout`（保留位）。
- `--help` / `--version` 不依赖 TTY，可在 CI、安装脚本和巡检脚本中调用。
- 单实例锁已启用：同一用户环境下禁止重复启动多个 `changqiao` 进程，避免终端状态和订阅状态互相污染。
- 支持系统信号优雅退出（`SIGINT`/`SIGTERM` 等），确保退出时恢复终端状态。

### 2.1 API 限流与重试

- 全局限流器：默认 10 req/s，突发桶容量 20。
- 429 / rate limit 自动指数退避重试。
- 已将关键 API 路径接入限流 helper（行情、静态信息、账户、持仓等）。

### 2.2 启动失败可诊断

- 启动时会加载 `.env`（若存在）。
- 缺少配置会输出明确引导并以非 0 退出码失败。
- 缺少配置会在真正初始化 SDK 前被预检并明确列出缺失变量名。
- 非 TTY 启动会直接拒绝运行，避免在 CI/后台脚本中“假启动”。

### 2.3 日志容错

- 优先写入平台默认日志目录。
- 若目录不可写，自动降级到系统临时目录。
- 日志初始化失败会明确报错并终止，避免“静默无日志”。
- 当 `CHANGQIAO_LOG` / `LONGBRIDGE_LOG` 配置非法时，不会导致启动失败，会自动回退到默认过滤规则并打印告警。
- 日志面板与后台日志监听统一读取 `active_log_dir`，避免“文件存在但面板为空”的路径漂移。

### 2.4 订阅生命周期治理

- 自选/详情订阅支持“重建前先退订”，避免重复订阅导致的数据推送放大。
- 订阅目标为空时会短路，避免无效 API 调用与额外限流消耗。
- 订阅/退订失败会产生日志告警，便于值班排障。

### 2.5 资产刷新容错

- 资产刷新加了“并发保护”，同一时刻只允许一个刷新任务执行，避免快速连按 `R` 造成请求堆积。
- 资产刷新增加了 12 秒超时控制，防止上游抖动时刷新任务长期挂起。
- 当刷新失败或超时时，保留并继续展示上一次成功数据（stale-but-usable），并在日志里明确告警。

## 3. 部署与发布建议

### 3.1 安装脚本安全基线

安装脚本已具备：

- `set -eu` 失败即退出。
- `curl --fail --location --show-error`，下载失败不继续。
- 使用临时目录并 `trap` 清理。
- 使用 `install -m 0755` 原子覆盖目标二进制。

### 3.2 推荐发布流程

1. 在 CI 完成 `fmt + clippy + test`。
2. 打 tag 触发 release 构建。
3. 发布后在干净环境执行一次安装与启动冒烟：

```bash
curl -sSL https://github.com/longbridge/longbridge-terminal/raw/main/install | sh
changqiao
```

## 4. 运行监控与告警（最小可行）

项目当前未内置 metrics/告警通道，建议先采用日志驱动的最小方案：

- 日志关键字告警（外部采集）：
  - `Failed to fetch`（外部 API 失败）
  - `Rate limit error`（触发限流）
  - `Configuration Error` / `配置错误`
- 告警阈值建议：
  - 5 分钟内同类错误 ≥ 20 次告警
  - 连续 3 次启动失败告警
- 可直接落地的扫描脚本：

```bash
./scripts/log_alert_guard.sh
```

返回码约定：
- `0`：未触发阈值
- `2`：触发告警阈值（建议外部平台据此告警）

## 5. 回滚策略

### 5.1 二进制回滚

保留上一版本二进制：

```bash
cp /usr/local/bin/changqiao /usr/local/bin/changqiao.prev
```

升级失败后回滚：

```bash
mv /usr/local/bin/changqiao.prev /usr/local/bin/changqiao
```

### 5.2 配置回滚

- `.env` 版本化存档（至少保留最近 2 版）。
- 变更凭证后若启动失败，优先回滚 `.env`。

## 6. 仍需确认（上线前最后澄清）

以下信息若不确认，线上会有持续风险：

1. **目标部署形态**：用户本地终端运行，还是托管在跳板机/容器？
2. **日志采集方式**：是否有统一日志平台（如 ELK/Datadog）？
3. **凭证轮换机制**：Access Token 3 个月过期，谁负责轮换、如何验证？
4. **发布窗口与回滚责任人**：出现行情拉取失败时，谁有权限快速回滚？

## 7. 安全审计基线说明

- 已接入 `cargo audit`，并默认拒绝告警。
- 当前通过 `scripts/cargo_audit.sh` 明确列出少量豁免（均为上游依赖链的暂不可快速替换项）。
- 每次升级 `bevy` / `ratatui` / `rust-i18n` / `longport` 后，应重新审查并尽量移除豁免项。

---

如果你希望，我可以继续补一份英文版 `docs/production_readiness.md`，并把这份清单链接到 `README.md`。
