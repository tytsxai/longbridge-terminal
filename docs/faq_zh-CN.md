# 常见问题 FAQ（中文）

## Q1：启动时报“缺少必需环境变量”怎么办？

先确认三项必需变量都已设置：

- `LONGPORT_APP_KEY`
- `LONGPORT_APP_SECRET`
- `LONGPORT_ACCESS_TOKEN`

建议方式：

```bash
cp .env.example .env
# 然后编辑 .env
```

---

## Q2：启动时报“需要在交互式终端（TTY）中运行”怎么办？

说明你在非交互环境启动了程序（例如重定向、某些 CI shell）。

请在真实终端中直接执行：

```bash
changqiao
```

---

## Q3：启动时报“已有 changqiao 进程在运行”怎么办？

这是单实例保护机制，避免多个进程互相污染终端状态。

处理方式：

1. 确认是否已有正在使用的实例
2. 若是残留进程，终止后重启

---

## Q4：页面数据不刷新，或者一直提示请求失败？

请按顺序排查：

1. 外网连通性
2. Token 是否过期（通常 3 个月）
3. 是否触发限流（日志中可能有 `Rate limit error`）

可先手动刷新：`R`

---

## Q5：日志文件在哪里？

默认：

- macOS：`~/Library/Logs/ChangQiao/`
- Linux：`~/.local/share/changqiao/logs/`

若默认目录不可写，会自动降级到系统临时目录。

---

## Q6：我原来用的是 `LONGBRIDGE_LOCALE`，现在还能用吗？

可以。当前版本同时支持：

- 新变量：`CHANGQIAO_LOCALE`、`CHANGQIAO_LOG`
- 旧变量：`LONGBRIDGE_LOCALE`、`LONGBRIDGE_LOG`

优先使用新变量，旧变量用于兼容存量环境。

---

## Q7：安装脚本下载失败怎么办？

先检查网络和版本号，再看是否需要指定仓库：

```bash
CHANGQIAO_REPO=longbridge/longbridge-terminal \
CHANGQIAO_VERSION=v0.7.0-preview0 \
sh install
```

安装脚本会优先尝试新产物，失败时回退旧产物。

---

## Q8：如何快速收集问题信息给维护者？

建议提供：

1. 执行命令与完整报错
2. `changqiao --version` 输出
3. 日志文件最近 50 行
4. 操作系统与终端类型

---

## Q9：哪里看发布、值班、回滚规范？

- 发布：[`release_runbook_zh-CN.md`](release_runbook_zh-CN.md)
- 值班：[`oncall_cheatsheet_zh-CN.md`](oncall_cheatsheet_zh-CN.md)
- 生产就绪：[`production_readiness_zh-CN.md`](production_readiness_zh-CN.md)
