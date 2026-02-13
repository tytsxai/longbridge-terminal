# AGENTS

## 项目定位

长桥终端是基于 Longport OpenAPI 的终端投资观察台（TUI），强调三件事：  
可用（真实看盘）、可维护（清晰结构）、可运维（可诊断与可回滚）。

## 目录结构（骨架）

```text
.
├── src
│   ├── main.rs                 # 启动入口：命令分发、TTY/配置预检、生命周期管理
│   ├── cli.rs                  # CLI 解析：run/help/version/doctor
│   ├── doctor.rs               # 环境诊断：TTY/环境变量/日志目录/DNS/单实例锁
│   ├── workspace.rs            # 工作区快照：启动恢复与退出持久化
│   ├── alerts.rs               # 预警规则：本地 JSON 存储与实时命中评估
│   ├── app.rs                  # 主事件循环：状态机、输入事件、渲染调度
│   ├── api/                    # 业务 API 封装（账户、行情、搜索等）
│   ├── openapi/                # SDK 上下文、限流器、重试包装
│   ├── data/                   # 领域数据模型与共享状态
│   ├── render/                 # 脏标记与增量渲染策略
│   ├── views/                  # 页面级组合视图
│   ├── widgets/                # 可复用 UI 组件
│   └── logger.rs               # 日志目录策略与 tracing 初始化
├── docs/                       # 使用、运维、值班、发布、FAQ 与产品规划文档
├── scripts/                    # preflight、告警扫描、安全审计脚本
├── install                     # 安装脚本
└── AGENTS.md                   # 架构意图与职责镜像
```

## 模块依赖与边界

1. `main -> cli`：先解析命令，再决定进入诊断或主程序。
2. `main -> doctor`：`doctor` 属于“非交互诊断路径”，不依赖 TTY。
3. `main -> workspace`：主循环退出时兜底保存工作区快照，保障下次恢复体验。
4. `app -> alerts`：行情 push 更新后进行规则评估，命中后记录日志并更新持久化状态。
5. `app -> api/openapi/data/render/views/widgets`：运行态编排中心，仅在 `app` 做流程串联。
6. `openapi` 负责“调用策略”（限流/重试）；`api` 负责“业务语义”。
7. `views/widgets` 不应直接初始化 SDK，上游数据由 `app/api/data` 提供。

## 开发规范（与当前架构强绑定）

- 新增命令优先放在 `cli.rs` 与独立模块，不把业务逻辑塞进 `main.rs`。
- 所有可诊断能力优先做成命令化入口（如 `doctor`），避免仅靠 FAQ 人工排障。
- 产品规划文档统一放在 `docs/`（战略、Roadmap、PRD），并在 README 导航中可达。
- 架构级变更（新增核心模块、目录调整）必须同步更新 `AGENTS.md` 与 `docs/`。
- 发布前必须通过 `scripts/release_preflight.sh`。

## 变更日志（本次）

- 2026-02-12：新增 `src/doctor.rs`，提供可脚本化的环境诊断命令。
- 2026-02-12：扩展 `src/cli.rs` 与 `src/main.rs`，支持 `changqiao doctor` / `--doctor`。
- 2026-02-12：更新 `README*.md`、`docs/quickstart_zh-CN.md`、`docs/faq_zh-CN.md`，补充诊断流程。
- 2026-02-12：新增 `docs/product_strategy_zh-CN.md`、`docs/roadmap_2026Q2_zh-CN.md`、`docs/prd_alert_center_zh-CN.md`，固化产品方案。
- 2026-02-12：新增 `src/workspace.rs`，实现工作区自动恢复与退出持久化。
- 2026-02-12：新增 `src/alerts.rs`，实现本地预警规则存储与实时命中评估。
