# 快速上手（实用版）

> 目标：第一次使用时，10 分钟内完成“安装 → 诊断 → 启动 → 会用”。

---

## 1. 最短路径（复制即可）

```bash
# 1) 安装
curl -sSL https://github.com/longbridge/longbridge-terminal/raw/main/install | sh

# 2) 准备配置
cp .env.example .env
# 编辑 .env，填入 LONGPORT_APP_KEY / LONGPORT_APP_SECRET / LONGPORT_ACCESS_TOKEN

# 3) 先诊断后启动
changqiao doctor
changqiao
```

如果第 3 步里 `doctor` 出现 `FAIL`，先解决再启动。

安装说明：若本机已有旧版 `changqiao`，安装脚本会自动备份到 `/usr/local/bin/changqiao.prev`，可直接回滚。

---

## 2. 安装后先确认三件事

```bash
changqiao --version
changqiao --help
changqiao doctor
```

你需要看到：

1. `--version` 能输出版本号
2. `--help` 能显示命令
3. `doctor` 没有阻塞项（`FAIL`）

---

## 3. `.env` 最小配置

```bash
LONGPORT_APP_KEY=your_app_key
LONGPORT_APP_SECRET=your_app_secret
LONGPORT_ACCESS_TOKEN=your_access_token
```

常用可选项：

```bash
CHANGQIAO_LOCALE=zh-CN
CHANGQIAO_LOG=error,changqiao=info
# 终端托管时建议固定目录，方便采集与备份
CHANGQIAO_LOG_DIR=/var/log/changqiao
CHANGQIAO_DATA_DIR=/var/lib/changqiao
```

---

## 4. `doctor` 输出怎么读

- `PASS`：通过
- `WARN`：可运行，但建议修复
- `FAIL`：阻塞项，通常会导致主程序无法启动

高频问题：

1. 缺少环境变量：回到 `.env` 检查三项凭证
2. stdout 不是 TTY：不要在重定向/非交互 shell 中启动
3. DNS 失败：先确认网络与代理
4. 单实例锁冲突：确认是否已有 changqiao 进程在运行
5. `.env` 权限过宽：执行 `chmod 600 .env`，避免凭证被其他本机用户读取

---

## 5. 首次启动后先学的 6 个键

- `?`：打开帮助
- `/`：搜索股票
- `Enter`：进入当前项详情
- `R`：刷新
- `` ` ``：日志面板
- `q` / `ESC`：返回

退出：`Ctrl+C`

---

## 6. 本地文件位置（排障常用）

### macOS

- 日志：`~/Library/Logs/ChangQiao/`
- 工作区：`~/Library/Application Support/ChangQiao/workspace.json`
- 预警规则：`~/Library/Application Support/ChangQiao/alerts.json`

### Linux

- 日志：`~/.local/share/changqiao/logs/`
- 工作区：`~/.local/share/changqiao/workspace.json`
- 预警规则：`~/.local/share/changqiao/alerts.json`

---

## 7. 升级与回滚

### 升级

```bash
curl -sSL https://github.com/longbridge/longbridge-terminal/raw/main/install | sh
```

### 回滚（安装脚本默认会自动备份）

```bash
mv /usr/local/bin/changqiao.prev /usr/local/bin/changqiao
```

详细流程：[`release_runbook_zh-CN.md`](release_runbook_zh-CN.md)

---

## 8. 下一步阅读

- 常见问题：[`faq_zh-CN.md`](faq_zh-CN.md)
- 项目定位：[`project_positioning_zh-CN.md`](project_positioning_zh-CN.md)
- 生产运维：[`production_readiness_zh-CN.md`](production_readiness_zh-CN.md)
