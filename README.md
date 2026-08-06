# 长桥终端 · Longbridge Terminal（changqiao）

基于 [长桥 OpenAPI](https://open.longbridge.com) 的 **实验性终端股票 TUI**（Rust + Ratatui + Longport SDK）。

[![asciicast](https://asciinema.org/a/785102.svg)](https://asciinema.org/a/785102)

> **关键词**：长桥终端 · Longbridge TUI · Longport OpenAPI · Rust 股票终端 · Ratatui 行情 · 港股美股 A 股终端 · 命令行看盘 · changqiao
>
> **Keywords**: Longbridge terminal · Longport OpenAPI TUI · Rust stock terminal · Ratatui market data · HK US CN A-share CLI · command-line portfolio viewer · changqiao

**English summary**: Experimental terminal stock app built on Longport OpenAPI. Watchlists, quotes, portfolio, and candlestick charts in a Vim-friendly TUI. Good for learning the Rust SDK — not an auto-trading bot.

---

## 是什么 / What It Is

在终端里查看**实时自选行情、资产与持仓、搜索报价、K 线**，同时作为 **Rust + Longport OpenAPI SDK** 的可读参考工程。

一句话：**可用的终端投资工作台 + 可读的 SDK 参考实现。**

CLI 命令名：`changqiao`（包名与二进制见 `Cargo.toml` 的 `name = "changqiao"`，当前版本 `0.7.0-preview0`）。

## 解决什么问题

- 不想开完整桌面客户端，只想在 SSH / 本机终端快速看盘与持仓
- 学习 Longport OpenAPI（鉴权、行情、资产）在真实 TUI 里如何组织
- 参考 Ratatui 在实时推送场景下的状态与渲染结构

## 适合谁

- 有长桥账户与 OpenAPI 凭证的个人投资者 / 量化研究爱好者
- 想学 Rust TUI 或 Longport SDK 的中文开发者
- 需要港股 / 美股 / A 股多市场终端工作台的命令行用户

## 不适合 / 限制

- **不是**自动下单机器人（当前以行情 / 资产查看为主）
- **不是**高频低延迟交易系统或机构级交易网关
- Access Token 通常约 3 个月过期，需定期续期
- OpenAPI 有调用频率限制（文档建议量级约每秒 ≤ 10 次请求）
- 实验性工具，**不构成投资建议**
- 系统：macOS 或 Linux（见仓库依赖；Unix 与 Windows 有条件编译，安装脚本面向常见 Unix 环境）

---

## 核心功能

| 能力 | 说明 |
|------|------|
| 实时自选 | 自选股列表与市场数据 |
| 资产与持仓 | 资产概览、持仓查看 |
| 搜索与报价 | 股票搜索与报价 |
| K 线 | 蜡烛图（candlestick） |
| 多市场 | 港股、美股、A 股 |
| 交互 | Vim 风格快捷键；界面 i18n（`zh-CN` / `en` 等） |

## 技术栈 / Tech Stack

- **Rust** 2021 + **Tokio**
- **Ratatui** + Crossterm（TUI）
- **longport** SDK（OpenAPI）
- Bevy ECS 子集（`bevy_app` / `bevy_ecs`）组织应用状态
- `rust-i18n` 多语言（`locales/`）

---

## 快速开始 / Quick Start

### 安装

```bash
curl -sSL https://github.com/longbridge/longbridge-terminal/raw/main/install | sh
```

> 本仓库镜像维护地址：<https://github.com/tytsxai/longbridge-terminal>。安装脚本 URL 以你实际托管的上游/ fork 为准；源码构建见下文。

安装后：

```bash
changqiao --help
```

### 从源码构建

```bash
git clone https://github.com/tytsxai/longbridge-terminal.git
cd longbridge-terminal
cargo build --release
# 二进制位于 target/release/changqiao
```

### 配置凭证

1. 在 [长桥开放平台](https://open.longbridge.com) 创建应用并获取 `APP_KEY` / `APP_SECRET` / `ACCESS_TOKEN`
2. 复制模板：

```bash
cp .env.example .env
```

```bash
LONGPORT_APP_KEY=your_app_key
LONGPORT_APP_SECRET=your_app_secret
LONGPORT_ACCESS_TOKEN=your_access_token
```

### 启动

```bash
changqiao
```

---

## 常用键位

| 键 | 作用 |
|----|------|
| `?` | 帮助 |
| `/` | 股票搜索 |
| `` ` `` | 打开/关闭日志面板 |
| `q` / `ESC` | 返回或关闭当前窗口 |
| `Enter` | 确认 |
| `R` | 手动刷新 |

更多：[`docs/quickstart_zh-CN.md`](docs/quickstart_zh-CN.md)

---

## 配置项速查

### 必需

- `LONGPORT_APP_KEY`
- `LONGPORT_APP_SECRET`
- `LONGPORT_ACCESS_TOKEN`

### 可选

- `CHANGQIAO_LOCALE`：界面语言（如 `zh-CN` / `en`）
- `CHANGQIAO_LOG`：日志过滤（如 `error,changqiao=info`）
- `LONGPORT_REGION`
- `LONGPORT_HTTP_URL`、`LONGPORT_QUOTE_WS_URL`

兼容旧变量：`LONGBRIDGE_LOCALE`、`LONGBRIDGE_LOG`。

---

## 使用场景

- SSH 到服务器后快速看自选与持仓
- 本地终端日常看盘 + K 线
- 阅读/改造 SDK 调用与 TUI 工程结构

## 文档导航

### 新手与使用

- [项目定位与适用场景](docs/project_positioning_zh-CN.md)
- [5 分钟快速上手](docs/quickstart_zh-CN.md)
- [常见问题 FAQ](docs/faq_zh-CN.md)

### 维护与运维

- [生产就绪清单](docs/production_readiness_zh-CN.md)
- [发布日 Runbook](docs/release_runbook_zh-CN.md)
- [值班速查](docs/oncall_cheatsheet_zh-CN.md)
- [事故复盘模板](docs/postmortem_template_zh-CN.md)

### 深入阅读

- [限流设计](docs/rate_limiting_zh-CN.md)
- [渲染优化](docs/render_optimization_zh-CN.md)
- [中文化检查清单](docs/chinese_localization_checklist_zh-CN.md)
- [llms.txt](llms.txt)

外部：

- [长桥 OpenAPI](https://open.longbridge.com)
- [Rust SDK 文档](https://longportapp.github.io/openapi/rust/longport/)

---

## 风险提示

- 实验性工具，请勿视为投资建议
- 妥善保管 OpenAPI 凭证，勿提交 `.env`
- 注意 API 限流与 Token 续期

## 贡献与许可

- 贡献指南：[CONTRIBUTING.md](CONTRIBUTING.md)
- 许可证：MIT
