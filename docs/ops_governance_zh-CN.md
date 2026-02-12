# 长桥终端 运行治理与责任模板（中文）

> 目标：把“知道要做”变成“明确谁做、何时做、失败怎么兜底”。

## 1. 发布与值班责任表（每次发布前必须填写）

| 角色 | 姓名/值班组 | 联系方式 | 兜底人 |
|---|---|---|---|
| 发布负责人（Owner） | TODO | TODO | TODO |
| 观察负责人（Observer） | TODO | TODO | TODO |
| 回滚负责人（Rollback） | TODO | TODO | TODO |
| Token 轮换负责人 | TODO | TODO | TODO |

> 要求：发布窗口开始前 30 分钟，Owner 在群里确认“责任表已生效”。

## 2. Token 轮换机制（最小可执行 SOP）

### 2.1 轮换节奏

- `LONGPORT_ACCESS_TOKEN` 通常约 3 个月过期。
- 建议在 **到期前 14 天** 启动轮换流程，避免踩最后期限。

### 2.2 轮换步骤

1. 在长桥开放平台生成新 Token。
2. 仅更新目标环境变量：`LONGPORT_ACCESS_TOKEN`（不要改其他配置）。
3. 在非生产环境先做一次启动验证：

```bash
changqiao --version
changqiao
```

4. 生产窗口内切换 Token，启动后做 5 分钟观察。
5. 记录轮换时间、操作人、验证结果。

### 2.3 失败回退

- 若切换后出现连续启动失败或核心数据不可用，立即回滚至上一版 `.env`。
- 回滚后执行冒烟：

```bash
changqiao
```

## 3. 日志告警基线（无集中监控时的最小方案）

可用脚本（建议接入 cron 或外部任务调度）：

```bash
./scripts/log_alert_guard.sh
```

默认规则：

- 统计最近 5 分钟关键错误。
- 当错误数 `>= 20` 时返回退出码 `2`（可由调度系统判定告警）。

可通过环境变量覆盖：

- `WINDOW_MINUTES`
- `ALERT_THRESHOLD`
- `ALERT_KEYWORDS`
- `LOG_DIR`

## 4. 发布窗口执行顺序（建议）

1. 执行 `./scripts/release_preflight.sh`。
2. Owner 宣布发布开始。
3. 升级并人工冒烟。
4. 执行 `./scripts/log_alert_guard.sh` 做首轮观察。
5. Observer 在 T+30 分钟给出放行/回滚结论。

## 5. 变更记录

| 日期 | 变更 | 操作人 | 备注 |
|---|---|---|---|
| 2026-02-12 | 初始化模板 | AI | 首版 |
