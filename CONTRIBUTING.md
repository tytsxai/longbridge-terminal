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

2. （推荐）配置 Fork 远程：

   ```bash
   # origin 指向你的 fork（用于 push）
   git remote set-url origin git@github.com:<your-username>/longbridge-terminal.git

   # upstream 指向官方仓库（用于同步）
   git remote add upstream https://github.com/longbridge/longbridge-terminal \
     || git remote set-url upstream https://github.com/longbridge/longbridge-terminal
   git remote -v
   ```

   期望结果示例：

   ```text
   origin   git@github.com:<your-username>/longbridge-terminal.git (fetch)
   origin   git@github.com:<your-username>/longbridge-terminal.git (push)
   upstream https://github.com/longbridge/longbridge-terminal (fetch)
   upstream https://github.com/longbridge/longbridge-terminal (push)
   ```

3. 配置凭证：

   ```bash
   cp .env.example .env
   # 编辑 .env，填入 LONGPORT_APP_KEY / LONGPORT_APP_SECRET / LONGPORT_ACCESS_TOKEN
   ```

4. 运行程序：

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
python3 scripts/check_i18n_guard.py
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test -- --test-threads=1
```

### 中文化守门脚本说明

`scripts/check_i18n_guard.py` 会执行以下检查：

1. `en/zh-CN/zh-HK` 语言包键集一致性；
2. UI 渲染路径（`src/system.rs`、`src/views/*`、`src/widgets/*`、`src/ui/*`）中是否新增疑似硬编码英文文案；
3. CLI 用户可见路径（`src/cli.rs`）是否新增疑似硬编码英文帮助/错误文案；
4. 中文入口文档是否存在，且 `README.md` 是否保留关键中文文档链接。

常见失败与修复建议：

- 报“缺失键/多余键”：
  - 同步更新 `locales/en.yml`、`locales/zh-CN.yml`、`locales/zh-HK.yml`；
  - 避免只改单一语言包。
- 报“疑似硬编码英文 UI/CLI 文案”：
  - 优先改为 `t!("...")` i18n 键；
  - 若确实是协议字段/占位符/URL，请调整实现避免直接在渲染/提示路径硬编码。

若本地缺少依赖，请先安装：

```bash
python3 -m pip install pyyaml
```

项目启用了较严格的 `clippy::pedantic` 策略；如需放宽规则，请在 PR 中明确说明原因与影响。

## 提交流程

1. 新建分支（建议）：

   ```bash
   git checkout -b feature/your-change
   ```

2. 与上游同步（建议每次开发前执行）：

   ```bash
   git fetch upstream
   git checkout main
   git rebase upstream/main
   ```

3. 按模块完成改动并自测通过。

4. 提交变更：

   - 提交信息要清晰描述目的与范围
   - 避免把不相关改动混在同一次提交

5. 推送到 fork 并发起 Pull Request：

   ```bash
   git push -u origin <your-branch>
   ```

6. 发起 Pull Request：

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
