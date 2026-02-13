# 长桥终端

基于 [长桥 OpenAPI](https://open.longbridge.com) 的终端投资观察台（TUI）。

它的目标很直接：

1. **真实可用**：能日常看盘，不是只会跑 Demo。
2. **工程可读**：代码结构清晰，适合作为 Rust + OpenAPI 参考项目。
3. **运维可诊断**：出现问题能快速定位、快速恢复。

[![asciicast](https://asciinema.org/a/785102.svg)](https://asciinema.org/a/785102)

---

## 1. 你会得到什么

- 实时自选行情（含多市场）
- 资产概览与持仓查看
- 股票搜索、个股详情、K 线
- 工作区自动记忆（分组/选中标的/K 线周期等）
- 本地预警规则持久化（`alerts.json`）
- `doctor` 一键诊断（TTY/环境变量/日志目录/DNS/单实例）

---

## 2. 3 分钟快速开始（推荐路径）

### 步骤 1：安装

```bash
curl -sSL https://github.com/longbridge/longbridge-terminal/raw/main/install | sh
```

### 步骤 2：准备凭证

```bash
cp .env.example .env
```

编辑 `.env`：

```bash
LONGPORT_APP_KEY=your_app_key
LONGPORT_APP_SECRET=your_app_secret
LONGPORT_ACCESS_TOKEN=your_access_token
```

### 步骤 3：先做诊断，再启动

```bash
changqiao doctor
changqiao
```

> 建议：第一次使用时，先确保 `doctor` 没有 `FAIL` 项。

---

## 3. 最常用命令

```bash
changqiao --help      # 查看命令帮助
changqiao --version   # 查看版本
changqiao doctor      # 环境诊断
changqiao             # 启动主程序
```

---

## 4. 高价值快捷键（先记这几个）

- `?`：帮助面板
- `/`：股票搜索
- `Enter`：进入详情
- `R`：手动刷新
- `` ` ``：日志面板开关
- `q` / `ESC`：返回上层
- `Ctrl+C`：退出

详细键位：[`docs/quickstart_zh-CN.md`](docs/quickstart_zh-CN.md)

---

## 5. 配置速查

### 必需变量

- `LONGPORT_APP_KEY`
- `LONGPORT_APP_SECRET`
- `LONGPORT_ACCESS_TOKEN`

### 常用可选变量

- `CHANGQIAO_LOCALE`：界面语言（如 `zh-CN` / `en`）
- `CHANGQIAO_LOG`：日志级别（如 `error,changqiao=info`）
- `LONGPORT_REGION`
- `LONGPORT_HTTP_URL` / `LONGPORT_QUOTE_WS_URL`

兼容旧变量：`LONGBRIDGE_LOCALE`、`LONGBRIDGE_LOG`。

---

## 6. 本地文件位置（排障必看）

### macOS

- 日志：`~/Library/Logs/ChangQiao/`
- 工作区：`~/Library/Application Support/ChangQiao/workspace.json`
- 预警规则：`~/Library/Application Support/ChangQiao/alerts.json`

### Linux

- 日志：`~/.local/share/changqiao/logs/`
- 工作区：`~/.local/share/changqiao/workspace.json`
- 预警规则：`~/.local/share/changqiao/alerts.json`

---

## 7. 常见问题入口

- [快速上手](docs/quickstart_zh-CN.md)
- [FAQ](docs/faq_zh-CN.md)
- [发布与回滚 Runbook](docs/release_runbook_zh-CN.md)
- [生产就绪检查](docs/production_readiness_zh-CN.md)

---

## 8. 文档导航（按场景）

### 新手上手

- [项目定位](docs/project_positioning_zh-CN.md)
- [快速上手](docs/quickstart_zh-CN.md)
- [FAQ](docs/faq_zh-CN.md)

### 开发维护

- [贡献指南](CONTRIBUTING.md)
- [中文化检查清单](docs/chinese_localization_checklist_zh-CN.md)
- [限流设计](docs/rate_limiting_zh-CN.md)
- [渲染优化](docs/render_optimization_zh-CN.md)

### 运维值班

- [发布 Runbook](docs/release_runbook_zh-CN.md)
- [值班速查](docs/oncall_cheatsheet_zh-CN.md)
- [运行治理模板](docs/ops_governance_zh-CN.md)

### 产品规划

- [产品战略](docs/product_strategy_zh-CN.md)
- [Roadmap（2026 Q2）](docs/roadmap_2026Q2_zh-CN.md)
- [预警中心 PRD](docs/prd_alert_center_zh-CN.md)

---

## 9. 贡献者远程约定（Fork 模式）

```bash
# origin: 你的 fork（用于 push）
git remote set-url origin git@github.com:<your-username>/longbridge-terminal.git

# upstream: 官方仓库（用于同步）
git remote add upstream https://github.com/longbridge/longbridge-terminal \
  || git remote set-url upstream https://github.com/longbridge/longbridge-terminal
```

---

## 10. 风险提示

- 本项目为实验性观察工具，不构成投资建议。
- Access Token 通常会过期，请定期更新。
- 网络抖动/限流会导致短时刷新失败，建议配合日志排查。

---

## 许可证

MIT
