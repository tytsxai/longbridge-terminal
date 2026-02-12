# 中文化任务清单（执行版）

> 目标：确保“文档与操作界面中文优先”，并保证改动可验证、可回滚。

## 1. 界面文案（用户直接可见）

- [x] 顶栏、加载提示、资产页提示改为 i18n 中文文案
- [x] 调试日志面板标题改为 `Keyboard.Console` 翻译
- [x] 默认用户名与默认账户名改为 i18n 键
- [x] 避免硬编码英文 UI 字符串

## 2. 日志与可观测文案（日志面板可见）

- [x] 启动/退出/初始化等关键日志改为中文
- [x] 账户、自选、持仓、K 线刷新等关键路径日志改为中文
- [x] 错误日志统一中文描述并保留原始错误上下文

## 3. 语言包完整性

- [x] 新增并对齐 `Loading.General`、`Portfolio.Loading` 等新键
- [x] 补齐 `zh-HK` 缺失键（`watchlist_group.na`、`Portfolio.No Holdings`、`TradeStatus.*`）
- [x] 保持 `en/zh-CN/zh-HK` 三套键集一致

## 4. 文档中文化

- [x] `CONTRIBUTING.md` 改为中文版本
- [x] `docs/rate_limiting.md` 切换为中文内容（与 `rate_limiting_zh-CN.md` 一致）
- [x] 保留现有中文运维文档体系（上线清单、runbook、值班速查、复盘模板）

## 5. 仓库操作界面中文化

- [x] GitHub Actions 工作流名称改为中文
- [x] CI/发布步骤名称改为中文，便于中文团队值守

## 6. 验证与交付

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy -- -D warnings`
- [x] `cargo test`
- [x] 按模块分批提交并推送

## 7. 后续可选项（不影响当前交付）

- [x] 将 `README` 标题与少量英文术语进一步本地化（品牌名可保留）
- [x] 增加“中文化守门检查脚本”（扫描新增硬编码英文 UI 文案）

## 8. 2026-02-13 本轮补充（执行完成）

- [x] K 线周期标签改为 i18n 键（`KlineType.Day/Week/Month/Year`），清除 UI 英文硬编码
- [x] 成交量单位按 locale 显示（en: `K/M/B`，zh-CN: `万/亿/万亿`，zh-HK: `萬/億/萬億`）
- [x] `openapi/rate_limiter` 关键日志改为中文，便于日志面板排障
- [x] 统一初始化/异常提示中文文案（`openapi/context`、`api/account`、`render/dirty_flags`）
- [x] 修复语言包细节（`请登录`、`选择 [⏎]`、`切换图表`、`股票标识无效`）
- [x] 保证三套语言包键集一致（`en/zh-CN/zh-HK` 均为 184 个键）
- [x] 增加 i18n 回归测试（K 线周期本地化与成交量边界值）
- [x] CI 增加 `PyYAML` 安装步骤，避免守门脚本依赖缺失
- [x] `release_preflight.sh` 纳入中文化守门检查，保持“本地预检 = CI 规则”

## 9. 取舍决策（P2）

- [x] 对仓库内“工程文档”保持中文优先；对第三方子 crate（`crates/cli-candlestick-chart`）沿用上游英文文档，不做大规模改写，避免后续同步成本升高

## 8. 守门与流程强化（持续改进）

- [x] CI 安装中文化检查依赖（`pyyaml`）
- [x] CI 执行“中文化守门检查”
- [x] 贡献指南补充“守门脚本说明 + 常见失败修复建议”
- [x] 增加本地一键检查命令（`make check-i18n` / `make check-all`）
