# 5 分钟快速上手（中文）

> 目标：第一次使用本项目的同学，可以在 5 分钟内跑起来并看见实时数据。

## 0. 准备条件

- macOS 或 Linux
- 可访问外网（至少可访问 GitHub 与长桥 OpenAPI）
- 已申请 Longport OpenAPI 凭证

---

## 1. 安装

```bash
curl -sSL https://github.com/longbridge/longbridge-terminal/raw/main/install | sh
```

安装成功后，先做基本检查：

```bash
changqiao --version
changqiao --help
```

> 安装脚本兼容新旧发布产物：优先下载 `changqiao-terminal`，不存在时自动回退到 `longbridge-terminal`。

---

## 2. 配置凭证

在项目目录创建 `.env`：

```bash
cp .env.example .env
```

填写以下必需项：

```bash
LONGPORT_APP_KEY=your_app_key
LONGPORT_APP_SECRET=your_app_secret
LONGPORT_ACCESS_TOKEN=your_access_token
```

可选项：

```bash
CHANGQIAO_LOCALE=zh-CN
CHANGQIAO_LOG=error,changqiao=info
```

---

## 3. 启动与首屏确认

```bash
changqiao
```

正常情况下你应看到：

1. 程序进入全屏终端 UI
2. 自选或行情区域开始刷新
3. 底部显示可用快捷键提示

若启动报错，请直接看：[FAQ](faq_zh-CN.md)

---

## 4. 5 个最常用操作

1. `?`：查看完整帮助
2. `/`：搜索股票
3. `Enter`：进入当前选择的详情
4. `R`：手动刷新数据
5. `` ` ``：打开日志面板

退出相关：

- `q` / `ESC`：返回上层/关闭弹窗
- `Ctrl+C`：退出程序

---

## 5. 常见环境变量速查

| 变量名 | 必需 | 说明 | 示例 |
|---|---|---|---|
| `LONGPORT_APP_KEY` | 是 | OpenAPI 应用 Key | `abc123` |
| `LONGPORT_APP_SECRET` | 是 | OpenAPI 应用 Secret | `secret_xyz` |
| `LONGPORT_ACCESS_TOKEN` | 是 | 访问令牌 | `token_xxx` |
| `CHANGQIAO_LOCALE` | 否 | 界面语言 | `zh-CN` |
| `CHANGQIAO_LOG` | 否 | 日志过滤规则 | `error,changqiao=debug` |

兼容旧变量：`LONGBRIDGE_LOCALE`、`LONGBRIDGE_LOG`。

---

## 6. 升级与回滚

### 升级

```bash
curl -sSL https://github.com/longbridge/longbridge-terminal/raw/main/install | sh
```

### 回滚（你有备份时）

```bash
mv /usr/local/bin/changqiao.prev /usr/local/bin/changqiao
```

详细流程见：[发布日 Runbook](release_runbook_zh-CN.md)

---

## 7. 下一步阅读

- 项目定位：[`project_positioning_zh-CN.md`](project_positioning_zh-CN.md)
- 常见问题：[`faq_zh-CN.md`](faq_zh-CN.md)
- 生产运维：[`production_readiness_zh-CN.md`](production_readiness_zh-CN.md)
