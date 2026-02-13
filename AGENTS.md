# AGENTS

## 项目定位

长桥终端是基于 Longport OpenAPI 的终端投资观察台（TUI）。

工程目标只有三条：

1. **可用**：日常看盘稳定可用。
2. **可维护**：模块边界清晰，改动影响可控。
3. **可运维**：出现问题可诊断、可回滚、可追踪。

---

## 目录骨架（核心）

```text
.
├── src
│   ├── main.rs                 # 启动入口：命令分发、环境预检、生命周期收敛
│   ├── cli.rs                  # CLI 解析：run/help/version/doctor
│   ├── doctor.rs               # 诊断链路：TTY/ENV/日志目录/DNS/单实例锁
│   ├── workspace.rs            # 工作区快照：启动恢复、退出持久化
│   ├── alerts.rs               # 预警规则：本地 JSON 存储、命中评估、冷却策略
│   ├── app.rs                  # 运行编排中心：状态机、事件循环、渲染调度
│   ├── api/                    # 业务语义层（账户/行情/搜索）
│   ├── openapi/                # SDK 调用策略层（限流/重试/上下文）
│   ├── data/                   # 领域模型与共享状态
│   ├── render/                 # 脏标记与增量渲染
│   ├── views/                  # 页面组合
│   ├── widgets/                # 通用组件
│   └── logger.rs               # tracing 初始化与日志目录策略
├── docs
│   ├── quickstart_zh-CN.md     # 新手最短启动路径
│   ├── faq_zh-CN.md            # 高频问题与止血步骤
│   ├── release_runbook_zh-CN.md# 发布与回滚流程
│   └── production_readiness_zh-CN.md # 生产就绪检查
├── scripts
│   ├── check_i18n_guard.py     # 中文化守门
│   ├── release_preflight.sh    # 发布前全链路检查
│   └── cargo_audit.sh          # 依赖安全审计
├── Makefile                    # check-all 聚合入口
└── AGENTS.md                   # 架构意图镜像
```

---

## 模块边界与依赖规则

1. `main -> cli`：先解析命令，再决定进入诊断路径或主程序路径。
2. `main -> doctor`：诊断路径必须可脚本化，不依赖交互式 UI。
3. `main -> app/workspace`：主程序退出时必须有快照兜底。
4. `app -> api/openapi/data/render/views/widgets`：仅 `app` 负责运行态编排。
5. `openapi` 负责调用策略，`api` 负责业务语义；不得混层。
6. `views/widgets` 不直接初始化 SDK；数据由上游注入。
7. `app -> alerts`：行情更新后触发规则评估，命中后写日志并持久化。

---

## 文档维护约定（核心）

出现以下任一情况，必须同步更新文档：

- 新增核心模块
- 目录结构调整
- 命令行为变化
- 运维流程变化

最少需要同步：

1. `AGENTS.md`（结构与边界）
2. `README*.md`（用户入口）
3. `docs/quickstart_zh-CN.md`（上手路径）
4. `docs/faq_zh-CN.md`（排障路径）

要求：文档以“可执行步骤”为中心，避免只讲概念。

---

## 开发与发布质量门

开发自检：

```bash
make check-all
```

发布前必须：

```bash
./scripts/release_preflight.sh
```

该流程失败时不得发布。

---

## 本次架构能力快照（2026-02-13）

- 已支持 `changqiao doctor` 非交互诊断链路。
- 已支持工作区快照自动恢复与退出持久化。
- 已支持本地预警规则存储与实时命中评估。
- 已引入 `Makefile` 统一质量检查入口。
- 核心用户文档已改为“实战导向”结构。
