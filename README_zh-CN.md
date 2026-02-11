# Longbridge Terminal

基于 [Longbridge OpenAPI](https://open.longbridge.com) 构建的_实验性_终端股票交易应用。

这是一个基于 Rust 的 TUI（终端用户界面）应用，用于监控市场数据和管理股票投资组合。旨在展示 Longbridge OpenAPI SDK 的功能。

[![asciicast](https://asciinema.org/a/785102.svg)](https://asciinema.org/a/785102)

## 功能特性

- 实时自选股列表与实时市场数据
- 投资组合管理
- 股票搜索与报价
- K 线图（蜡烛图）
- 多市场支持（港股、美股、A 股）
- 基于 Rust + Ratatui 构建
- Vim 风格的快捷键

## 系统要求

- macOS 或 Linux
- Longbridge OpenAPI 凭证（可免费获取）

## 安装

### 从二进制文件安装

如果您使用的是 macOS 或 Linux，请在终端中运行以下命令：

```bash
curl -sSL https://github.com/longbridge/longbridge-terminal/raw/main/install | sh
```

这将在您的终端中安装 `longbridge` 命令。

## 配置

在运行应用之前，您需要配置 Longbridge OpenAPI 凭证：

1. **获取 API 凭证**：访问 [长桥开放平台](https://open.longbridge.com) 创建应用并获取：
   - `APP_KEY`
   - `APP_SECRET`
   - `ACCESS_TOKEN`

2. **配置环境变量**：

   在项目根目录下创建一个 `.env` 文件：

   ```bash
   cp .env.example .env
   ```

   编辑 `.env` 并添加您的凭证：

   ```bash
   LONGPORT_APP_KEY=your_app_key
   LONGPORT_APP_SECRET=your_app_secret
   LONGPORT_ACCESS_TOKEN=your_access_token
   ```

   或者，将它们导出为环境变量：

   ```bash
   export LONGPORT_APP_KEY=your_app_key
   export LONGPORT_APP_SECRET=your_app_secret
   export LONGPORT_ACCESS_TOKEN=your_access_token
   ```

3. **运行应用**：

   ```bash
   longbridge
   ```

## API 频率限制

Longbridge OpenAPI 有频率限制：

- 每秒最多 10 次 API 调用
- Access Token 每 3 个月过期一次，需要续期

## 文档

- [Longbridge OpenAPI 文档](https://open.longbridge.com)
- [Rust SDK 文档](https://longportapp.github.io/openapi/rust/longport/)
- [生产就绪清单（中文）](docs/production_readiness_zh-CN.md)
- [发布日 Runbook（中文）](docs/release_runbook_zh-CN.md)
- [值班速查（中文）](docs/oncall_cheatsheet_zh-CN.md)
- [事故复盘模板（中文）](docs/postmortem_template_zh-CN.md)

## 许可证

MIT
