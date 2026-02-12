# 长桥终端 值班速查（On-call Cheatsheet）

## 1) 启动失败

### 现象

- 进程立即退出。
- 控制台出现 `配置错误` / `Configuration Error` / `缺少必需环境变量`。

### 立即处理

1. 检查并补齐：`LONGPORT_APP_KEY`、`LONGPORT_APP_SECRET`、`LONGPORT_ACCESS_TOKEN`。
2. 确认在交互式终端（TTY）运行，而不是后台管道。
3. 重启：`changqiao`。

补充：可先执行 `changqiao --version` 验证二进制可用；若输出正常但 `changqiao` 启动时报“已有 changqiao 进程在运行，请先关闭后再启动。”，说明存在残留进程或并发启动。

### 升级条件

- 连续 3 次启动失败仍未恢复。

## 2) 行情/持仓请求大量失败

### 现象

- 日志连续出现 `Failed to fetch ...`。
- 页面数据长时间不刷新。

### 立即处理

1. 检查外网连通性与长桥 OpenAPI 可达性。
2. 检查 Token 是否过期并重新生成。
3. 观察日志是否出现 `Rate limit error`（触发限流重试）。

### 升级条件

- 5 分钟内同类错误 ≥ 20 次。

快速判断是否达告警阈值：

```bash
./scripts/log_alert_guard.sh
```

## 3) 资产页刷新失败或长时间不更新

### 现象

- 日志出现 `获取资产数据超时` 或 `获取资产数据失败，已回退为展示上次成功数据`。
- 资产页数字不再更新，但仍显示旧数据。

### 立即处理

1. 先确认上游 OpenAPI 连通性与 Token 状态。
2. 在资产页手动按 `R` 触发一次刷新，观察日志是否恢复成功。
3. 若连续失败，告知用户当前展示的是“上次成功快照”，避免误判为实时数据。

### 升级条件

- 连续 3 次手动刷新仍失败。
- 15 分钟内无法恢复资产数据刷新。


## 4) 安装升级失败

### 现象

- `install` 输出 `sha256 不匹配` 或下载失败。

### 立即处理

1. 确认目标版本 release 与 `*.sha256` 文件已发布。
2. 检查网络代理/镜像是否篡改下载内容。
3. 恢复到上一版本：

```bash
mv /usr/local/bin/changqiao.prev /usr/local/bin/changqiao
```

## 5) 标准回滚步骤

```bash
cp /usr/local/bin/changqiao /usr/local/bin/changqiao.prev
# 部署新版本...
# 出现故障则回滚：
mv /usr/local/bin/changqiao.prev /usr/local/bin/changqiao
```

回滚后必须重新冒烟：

```bash
changqiao
```

## 6) 单实例冲突

### 现象

- 启动报错：`已有 changqiao 进程在运行，请先关闭后再启动。`

### 立即处理

1. 查找现有进程并确认是否有值班同学正在使用。
2. 若确认为僵尸/残留进程，终止后重试。
3. 避免同一机器同一账号并发拉起多个实例。
