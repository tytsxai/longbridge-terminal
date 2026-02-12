# 长桥终端 贡献指南

感谢你关注并参与 长桥终端！本文档说明本仓库的开发约定、提交流程与质量要求。

## 快速开始

### 环境要求

- Rust 工具链（建议使用最新 stable）
- Longport OpenAPI 凭证（可在 https://open.longbridge.com 获取）
- macOS 或 Linux

### 本地启动

1. 克隆仓库：

   ```bash
   git clone https://github.com/longbridge/longbridge-terminal.git
   cd longbridge-terminal
   ```

2. 配置凭证：

   ```bash
   cp .env.example .env
   # 编辑 .env，填入 LONGPORT_APP_KEY / LONGPORT_APP_SECRET / LONGPORT_ACCESS_TOKEN
   ```

3. 运行程序：

   ```bash
   cargo run
   ```

## 代码规范

### 文案与国际化

- 用户可见文案必须通过 `rust-i18n` 的 `t!` 宏读取，不要在代码里硬编码显示文本。
- 新增文案时需同时更新：
  - `locales/en.yml`
  - `locales/zh-CN.yml`
  - `locales/zh-HK.yml`
- 代码注释保持简洁、可读，优先与现有代码风格一致。

示例：

```rust
let status = t!("TradeStatus.Normal");
```

### 命名约定

- 类型：`UpperCamelCase`
- 函数/变量：`snake_case`
- 常量：`SCREAMING_SNAKE_CASE`

### 静态检查

提交前至少执行：

```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
```

项目启用了较严格的 `clippy::pedantic` 策略；如需放宽规则，请在 PR 中明确说明原因与影响。

## 提交流程

1. 新建分支（建议）：

   ```bash
   git checkout -b feature/your-change
   ```

2. 按模块完成改动并自测通过。

3. 提交变更：

   - 提交信息要清晰描述目的与范围
   - 避免把不相关改动混在同一次提交

4. 发起 Pull Request：

   - 说明变更背景、核心改动、验证结果
   - 如涉及界面变化，可附截图或录屏

## 架构速览

- `src/openapi/`：Longport OpenAPI 集成与上下文管理
- `src/data/`：数据模型与全局状态
- `src/app.rs`：应用主循环（Bevy ECS + Tokio）
- `src/system.rs`：页面渲染与交互逻辑
- `src/widgets/`、`src/views/`：可复用 UI 组件

核心流程：

```text
初始化 -> 订阅行情 -> 接收推送 -> 更新缓存 -> 触发渲染
```

## 调试与排障

日志目录：

- macOS：`~/Library/Logs/ChangQiao/`
- Linux：`~/.local/share/changqiao/logs/`

开启调试日志：

```bash
CHANGQIAO_LOG=error,changqiao=debug cargo run
```

## 交流与反馈

- 缺陷反馈：请提交可复现步骤、期望行为、实际行为
- 功能建议：请描述业务场景与收益
- 代码评审：聚焦问题本身，保持尊重、务实、可执行

感谢贡献！
