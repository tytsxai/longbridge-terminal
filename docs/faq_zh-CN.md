# FAQ（实战版）

> 这份 FAQ 按“先止血、再定位”写法组织。遇到问题先看 Q1/Q2。

---

## Q1：启动失败，第一步该做什么？

先跑诊断：

```bash
changqiao doctor
```

优先处理 `FAIL` 项；`WARN` 可暂时继续，但建议修复。

---

## Q2：提示“缺少必需环境变量”怎么办？

检查 `.env` 是否包含这三项：

- `LONGPORT_APP_KEY`
- `LONGPORT_APP_SECRET`
- `LONGPORT_ACCESS_TOKEN`

推荐重建：

```bash
cp .env.example .env
# 再填入真实值
```

---

## Q2.1：`doctor` 提示 “.env 权限过宽” 怎么办？

这是安全基线告警：你的凭证文件可能被同机其他用户读取。

修复命令（macOS/Linux）：

```bash
chmod 600 .env
```

再执行一次：

```bash
changqiao doctor
```

---

## Q3：提示“需要在交互式终端（TTY）中运行”怎么办？

说明当前不是交互终端（例如输出被管道/重定向）。

正确方式：

```bash
changqiao
```

不要写成：

```bash
changqiao > out.log
```

---

## Q4：提示“已有 changqiao 进程在运行”怎么办？

这是单实例保护。请先确认旧进程是否仍在运行：

```bash
ps aux | grep changqiao
```

确认无误后结束残留进程，再重启。

---

## Q5：行情不更新或频繁报错怎么办？

按顺序排查：

1. 网络是否可用（DNS/代理）
2. Token 是否过期（常见）
3. 是否触发限流

建议同时打开日志面板（`` ` ``）观察实时错误。

---

## Q6：日志在哪里？

- macOS：`~/Library/Logs/ChangQiao/`
- Linux：`~/.local/share/changqiao/logs/`

默认目录不可写时，会降级到临时目录。

如果你希望在终端托管环境中固定目录，配置：

```bash
CHANGQIAO_LOG_DIR=/var/log/changqiao
CHANGQIAO_DATA_DIR=/var/lib/changqiao
```

---

## Q7：如何查看更详细日志？

```bash
CHANGQIAO_LOG=error,changqiao=debug changqiao
```

本地开发时可用：

```bash
CHANGQIAO_LOG=error,changqiao=debug cargo run
```

---

## Q8：为什么重启后还能记住上次分组和标的？

程序会自动保存 `workspace.json`，包含：

1. 分组
2. 选中标的
3. K 线周期与偏移
4. 日志面板状态

如需重置，删除本地 `workspace.json` 后重启。

---

## Q9：预警规则文件在哪里？

- macOS：`~/Library/Application Support/ChangQiao/alerts.json`
- Linux：`~/.local/share/changqiao/alerts.json`

文件损坏时会自动备份为 `*.corrupt.*.bak` 并重置为空规则。

---

## Q10：安装脚本失败怎么办？

可以显式指定仓库与版本：

```bash
CHANGQIAO_REPO=longbridge/longbridge-terminal \
CHANGQIAO_VERSION=v0.7.0-preview0 \
sh install
```

补充：安装成功时若检测到旧版二进制，脚本会自动备份到 `/usr/local/bin/changqiao.prev`，故障时可直接回滚。

---

## Q11：提交 issue 时最好附什么？

请附以下信息，定位会快很多：

1. 复现步骤（越短越好）
2. `changqiao --version` 输出
3. `changqiao doctor` 输出
4. 最近 50 行日志
5. 操作系统与终端类型

---

## Q12：贡献代码时，`origin` 和 `upstream` 怎么配？

```bash
# origin -> 你的 fork
git remote set-url origin git@github.com:<your-username>/longbridge-terminal.git

# upstream -> 官方仓库
git remote add upstream https://github.com/longbridge/longbridge-terminal \
  || git remote set-url upstream https://github.com/longbridge/longbridge-terminal
```

开发前建议：

```bash
git fetch upstream
git checkout main
git rebase upstream/main
```

---

## Q13：发布/值班/回滚规范看哪里？

- 发布：[`release_runbook_zh-CN.md`](release_runbook_zh-CN.md)
- 值班：[`oncall_cheatsheet_zh-CN.md`](oncall_cheatsheet_zh-CN.md)
- 生产就绪：[`production_readiness_zh-CN.md`](production_readiness_zh-CN.md)
