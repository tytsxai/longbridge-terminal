# 长桥终端 发布日 Runbook（中文）

> 适用场景：版本即将上线，要求“可验证、可回滚、可追责”。

## 0. 角色与职责（发布前 1 天确认）

- **发布负责人（Owner）**：执行发布与最终放行。
- **观察负责人（Observer）**：发布后 30 分钟盯盘日志与错误率。
- **回滚负责人（Rollback）**：出现故障时 5 分钟内执行回滚。

## 1. 发布前 30 分钟检查（Pre-flight）

### 1.1 代码与产物

在待发布 commit 上执行：

```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
cargo check

# 可执行文件基础可运维性检查（无需 TTY）
cargo run -- --help > /dev/null
cargo run -- --version > /dev/null
```

必须全绿，任何一项失败禁止发布。

### 1.2 配置与凭证

确认目标环境已配置：

- `LONGPORT_APP_KEY`
- `LONGPORT_APP_SECRET`
- `LONGPORT_ACCESS_TOKEN`

并确认 Token 未过期（有效期通常 3 个月）。

### 1.3 回滚预案就绪

先备份线上旧版本二进制：

```bash
cp /usr/local/bin/changqiao /usr/local/bin/changqiao.prev
```

## 2. 发布步骤（T0）

### 2.1 安装/升级

```bash
curl -sSL https://github.com/longbridge/longbridge-terminal/raw/main/install | sh
```

### 2.2 启动冒烟（人工）

```bash
changqiao
```

重点验证：

1. 应用正常进入终端 UI（非空白/非卡死）。
2. 自选股行情正常刷新（价格与涨跌变化）。
3. 账户/持仓页可打开（若账号有交易权限）。
4. 搜索功能可返回结果。

若提示 `已有 changqiao 进程在运行，请先关闭后再启动。`，先确认并终止旧进程后再继续发布。

## 3. 发布后 30 分钟观察（T+30）

### 3.1 日志观察（必须）

检查日志目录最新文件（按平台）：

- macOS：`~/Library/Logs/ChangQiao/`
- Linux：`~/.local/share/changqiao/logs/`
- 降级目录（若主目录不可写）：系统临时目录 `.../changqiao/logs/`

关注关键错误：

- `Failed to fetch`
- `Rate limit error`
- `配置错误` / `Configuration Error`

### 3.2 放行标准（Go）

满足以下条件可放行：

- 无持续性启动失败。
- 无高频 API 错误（同类错误 5 分钟内 < 20 次）。
- 核心功能（行情/搜索/持仓）可用。

## 4. 故障回滚（No-Go）

### 4.1 回滚触发条件（任一满足立即回滚）

- 应用无法稳定启动。
- 行情持续不可用（> 5 分钟）。
- 关键功能不可用且无法在 5 分钟内热修。

### 4.2 回滚命令

```bash
mv /usr/local/bin/changqiao.prev /usr/local/bin/changqiao
```

回滚后立即执行冒烟：

```bash
changqiao
```

## 5. 事后复盘（T+1 天）

至少记录：

1. 发布时间线（开始/完成/回滚点）。
2. 实际异常与影响范围。
3. 预防动作（代码/文档/流程）及负责人。

建议直接使用模板：`docs/postmortem_template_zh-CN.md`

---

如果你愿意，我可以再补一版“值班告警清单（On-call Cheatsheet）”，把日志关键字、处理动作、升级路径整理成一页。
