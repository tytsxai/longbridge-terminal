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

---

## Q10：如何一键检查我本地环境是否可启动？

执行：

```bash
changqiao doctor
```

诊断项包括：

1. 交互式终端（TTY）
2. 必需环境变量
3. 日志目录写入权限
4. DNS 解析能力
5. 单实例锁状态

结果判定：

- `PASS`：通过
- `WARN`：可继续，但建议处理
- `FAIL`：阻塞项，建议先修复

---

## Q11：重启后为什么还能记住我上次看的分组和标的？

程序会在退出时自动保存工作区快照（`workspace.json`），包括：

1. 当前分组
2. 选中的自选标的
3. K 线周期与偏移
4. 日志面板开关状态

如需重置，可删除本地 `workspace.json` 后重启。

---

## Q12：本地预警规则文件在哪里？

macOS 默认路径：

```text
~/Library/Application Support/ChangQiao/alerts.json
```

文件损坏时程序会自动备份为 `*.corrupt.*.bak` 并重置为空规则集合。

---

## Q13：贡献代码时，`origin` 和 `upstream` 应该怎么配？

推荐使用标准 Fork 结构：

```bash
# origin 指向你的 fork（用于推送分支）
git remote set-url origin git@github.com:<your-username>/longbridge-terminal.git

# upstream 指向官方仓库（用于同步主线）
git remote add upstream https://github.com/longbridge/longbridge-terminal \
  || git remote set-url upstream https://github.com/longbridge/longbridge-terminal
git remote -v
```

建议每次开发前先同步：

```bash
git fetch upstream
git checkout main
git rebase upstream/main
```

如果你看到多个远程都指向同一个仓库（例如 `origin` 和另一个自定义远程重复），可删除重复远程，保持 `origin + upstream` 两个即可。
