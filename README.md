# 长桥终端

基于 [长桥 OpenAPI](https://open.longbridge.com) 构建的 **实验性终端股票应用（终端界面，TUI）**。

它不是“炫技演示（Demo）”，而是一个面向中文开发者与交易研究者的参考实现：
- 一边在终端里看行情与持仓；
- 一边学习如何用 Rust + Longport 软件开发工具包（SDK）组织实时数据、状态与界面（UI）。

[![asciicast](https://asciinema.org/a/785102.svg)](https://asciinema.org/a/785102)

---

## 1. 项目用途（先看这个）

### 这个项目适合你，如果你希望：

- 在命令行里快速查看自选行情、资产、K 线
- 用一个可运行项目学习 Longport OpenAPI 软件开发工具包（SDK）
- 参考 Rust 终端界面（Ratatui）在实时场景下的工程组织方式

### 这个项目不适合你，如果你需要：

- 自动下单机器人（本项目当前以行情/资产查看为主）
- 高频低延迟交易系统
- 面向机构的大规模交易网关

> 一句话：**它是“可用的终端投资工作台 + 可读的 SDK 参考工程”。**

---

## 2. 功能特性

- 实时自选股列表与市场数据
- 资产概览与持仓查看
- 股票搜索与报价
- K 线图（蜡烛图）
- 多市场支持（港股、美股、A 股）
- 基于 Rust + Ratatui 构建
- Vim 风格快捷键
- 工作区自动记忆（分组、选中标的、K 线周期等）
- 本地价格预警规则引擎（JSON 持久化）

---

## 3. 系统要求

- macOS 或 Linux
- 长桥 OpenAPI 凭证（可免费获取）

---

## 4. 5 分钟快速上手

### 4.1 安装

```bash
curl -sSL https://github.com/longbridge/longbridge-terminal/raw/main/install | sh
```

安装后可用命令：

```bash
changqiao --help
changqiao doctor
```

### 4.2 配置凭证

1. 在 [长桥开放平台](https://open.longbridge.com) 创建应用并获取：
   - `APP_KEY`
   - `APP_SECRET`
   - `ACCESS_TOKEN`

2. 复制配置模板并填写：

```bash
cp .env.example .env
```

`.env` 示例：

```bash
LONGPORT_APP_KEY=your_app_key
LONGPORT_APP_SECRET=your_app_secret
LONGPORT_ACCESS_TOKEN=your_access_token
```

如需快速自检环境，请先运行：

```bash
changqiao doctor
```

`doctor` 会检查：TTY、必需环境变量、日志目录写入、DNS 解析、单实例锁。

### 4.3 启动

```bash
changqiao
```

---

## 5. 常用键位（高频）

- `?`：打开帮助
- `/`：打开股票搜索
- `` ` ``：打开/关闭日志面板
- `q` / `ESC`：返回上一层或关闭当前窗口
- `Enter`：确认当前选择
- `R`：手动刷新数据

更多键位请查看：[`docs/quickstart_zh-CN.md`](docs/quickstart_zh-CN.md)

---

## 6. 配置项速查

### 必需环境变量

- `LONGPORT_APP_KEY`
- `LONGPORT_APP_SECRET`
- `LONGPORT_ACCESS_TOKEN`

### 可选环境变量

- `CHANGQIAO_LOCALE`：界面语言（如 `zh-CN` / `en`）
- `CHANGQIAO_LOG`：日志过滤（如 `error,changqiao=info`）
- `LONGPORT_REGION`
- `LONGPORT_HTTP_URL`、`LONGPORT_QUOTE_WS_URL`

兼容旧变量（仍可用）：

- `LONGBRIDGE_LOCALE`
- `LONGBRIDGE_LOG`

---

## 7. 中文文档导航

### 新手与使用

- [项目定位与适用场景（中文）](docs/project_positioning_zh-CN.md)
- [5 分钟快速上手（中文）](docs/quickstart_zh-CN.md)
- [常见问题（FAQ，中文）](docs/faq_zh-CN.md)

### 维护与运维

- [生产就绪清单（中文）](docs/production_readiness_zh-CN.md)
- [发布日操作手册（Runbook，中文）](docs/release_runbook_zh-CN.md)
- [值班速查（中文）](docs/oncall_cheatsheet_zh-CN.md)
- [事故复盘模板（中文）](docs/postmortem_template_zh-CN.md)
- [运行治理与责任模板（中文）](docs/ops_governance_zh-CN.md)

### 深入阅读

- [产品战略（中文）](docs/product_strategy_zh-CN.md)
- [Roadmap（2026 Q2，中文）](docs/roadmap_2026Q2_zh-CN.md)
- [预警中心 PRD（中文）](docs/prd_alert_center_zh-CN.md)
- [限流设计说明（中文）](docs/rate_limiting_zh-CN.md)
- [渲染优化说明（中文）](docs/render_optimization_zh-CN.md)
- [中文化检查清单（中文）](docs/chinese_localization_checklist_zh-CN.md)

外部文档：

- [长桥 OpenAPI 文档](https://open.longbridge.com)
- [Rust SDK 文档](https://longportapp.github.io/openapi/rust/longport/)

第三方依赖说明：

- `crates/cli-candlestick-chart/README.md` 为上游项目文档，默认保持英文原文，避免偏离上游更新。

---

## 8. 风险提示

- 本项目为实验性工具，请勿将其视为投资建议。
- Access Token 通常 3 个月过期，需定期续期。
- Longport OpenAPI 存在调用频率限制（默认建议不超过每秒 10 次请求）。

---

## 9. 贡献者 Git 远程约定

如果你要向官方仓库提交 PR，建议使用标准 Fork 远程结构：

```bash
# origin：你的 fork（用于 push）
git remote set-url origin git@github.com:<your-username>/longbridge-terminal.git

# upstream：官方仓库（用于同步）
git remote add upstream https://github.com/longbridge/longbridge-terminal \
  || git remote set-url upstream https://github.com/longbridge/longbridge-terminal
git remote -v
```

建议在每次开发前先同步上游：

```bash
git fetch upstream
git checkout main
git rebase upstream/main
```

完整流程见：[`CONTRIBUTING.md`](CONTRIBUTING.md)

---

## 许可证

MIT
