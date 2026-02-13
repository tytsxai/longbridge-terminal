# 贡献指南（实用版）

这份文档只做一件事：让你**少踩坑、快提交、可合并**。

---

## 1. 开始前 5 分钟

### 1.1 必备环境

- Rust stable（建议最新）
- Python 3（用于 i18n 守门脚本）
- Longport OpenAPI 凭证（本地运行需要）
- macOS 或 Linux

### 1.2 推荐 Git 远程结构（Fork）

```bash
# 进入仓库
cd longbridge-terminal

# origin 指向你的 fork（可 push）
git remote set-url origin git@github.com:<your-username>/longbridge-terminal.git

# upstream 指向官方仓库（用于同步）
git remote add upstream https://github.com/longbridge/longbridge-terminal \
  || git remote set-url upstream https://github.com/longbridge/longbridge-terminal

git remote -v
```

期望：`origin` 是你的仓库，`upstream` 是官方仓库。

---

## 2. 本地运行（最短路径）

```bash
cp .env.example .env
# 填入 LONGPORT_APP_KEY / LONGPORT_APP_SECRET / LONGPORT_ACCESS_TOKEN

changqiao doctor
cargo run
```

如果 `doctor` 有 `FAIL`，先修复再继续。

---

## 3. 开发流程（推荐）

### 步骤 1：同步上游

```bash
git fetch upstream
git checkout main
git rebase upstream/main
```

### 步骤 2：开分支

```bash
git checkout -b feature/<short-topic>
```

### 步骤 3：开发并自测

至少执行：

```bash
make check-all
./scripts/release_preflight.sh
```

### 步骤 4：分批提交

建议“按模块/按意图”拆 commit：

- feat/fix（代码行为）
- docs（文档）
- chore（脚本/治理）

### 步骤 5：推送并提 PR

```bash
git push -u origin <your-branch>
```

PR 描述建议包含：

1. 背景与目标
2. 关键改动点
3. 验证结果（命令 + 输出摘要）
4. 风险与回滚方式

---

## 4. 必过质量门（合并前）

## 4.1 统一入口

```bash
make check-all
```

执行内容：

1. `scripts/check_i18n_guard.py`
2. `cargo fmt --check`
3. `cargo clippy -D warnings`
4. `cargo test`

## 4.2 发布前预检

```bash
./scripts/release_preflight.sh
```

该脚本会串行执行脚本语法、格式、i18n、clippy、测试、构建、冒烟、audit。

---

## 5. 文案与 i18n 规则（高频踩坑）

1. 用户可见文案尽量走 `t!(...)`，不要硬编码英文。
2. 新增文案键需同步三个语言包：
   - `locales/en.yml`
   - `locales/zh-CN.yml`
   - `locales/zh-HK.yml`
3. CLI 错误提示也属于“用户可见路径”，会被守门脚本检查。

常见失败修复：

- “缺失键/多余键”：三份语言包键集对齐。
- “疑似硬编码英文”：改为 i18n 键，或改成中文主文案 + 参数占位。

---

## 6. 架构改动的强制要求

只要出现以下任一项，视为架构级改动：

- 新增/删除核心模块
- 目录结构调整
- 模块职责变化

必须同步更新：

1. `AGENTS.md`（职责与依赖边界）
2. 相关 `docs/*`（使用/运维/开发流程）

---

## 7. 常用调试信息

### 日志目录

- macOS: `~/Library/Logs/ChangQiao/`
- Linux: `~/.local/share/changqiao/logs/`

### 打开调试日志

```bash
CHANGQIAO_LOG=error,changqiao=debug cargo run
```

### 快速附带排障信息（提 issue 时）

1. 复现步骤（最少步骤）
2. `changqiao --version`
3. `changqiao doctor` 输出
4. 日志最后 50 行
5. 系统与终端类型

---

## 8. 代码风格（简版）

- 类型：`UpperCamelCase`
- 函数/变量：`snake_case`
- 常量：`SCREAMING_SNAKE_CASE`
- 改动优先小而清晰，避免“大而混杂”提交

---

感谢贡献。目标一致：**让这个终端工具更稳、更好用、更容易维护**。
