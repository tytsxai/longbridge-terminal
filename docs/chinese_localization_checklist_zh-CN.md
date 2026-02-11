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

- [ ] 将 `README` 标题与少量英文术语进一步本地化（品牌名可保留）
- [ ] 增加“中文化守门检查脚本”（扫描新增硬编码英文 UI 文案）

