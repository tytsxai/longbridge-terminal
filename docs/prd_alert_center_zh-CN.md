# PRD：价格预警中心（最小可行版）

> 文档状态：Draft v1  
> 目标版本：v0.8.x（M2）

## 1. 背景

当前长桥终端已支持行情查看，但“每天回来打开”的留存抓手不足。  
预警中心用于把“被动查看”升级为“主动提醒”。

## 2. 目标

1. 用户可在终端内创建、查看、启停、删除预警规则。
2. 规则命中时，UI 显示明显提醒，并记录日志。
3. 首版只做本地规则，不做云同步。

## 3. 非目标

1. 不发送外部通知（邮件/短信/飞书）
2. 不做策略回测
3. 不触发自动下单

## 4. 用户故事

1. 作为观察者，我希望“某股票涨到目标价时提醒我”。
2. 作为短线用户，我希望“涨跌幅超过阈值时提醒我”。
3. 作为谨慎用户，我希望“成交量异动时提醒我”。

## 5. 功能范围（MVP）

## 5.1 规则类型

```text
RuleType:
  - PriceAbove(symbol, threshold)
  - PriceBelow(symbol, threshold)
  - ChangePercentAbove(symbol, percent)
  - ChangePercentBelow(symbol, percent)
  - VolumeAbove(symbol, volume_threshold)
```

## 5.2 规则状态

```text
RuleStatus:
  - Enabled
  - Disabled
  - TriggeredCooldown(可选，避免短时间重复刷屏)
```

## 5.3 交互入口

1. 快捷键打开“预警中心弹窗”
2. 在个股详情页快捷创建预警
3. 在预警中心管理（启停/删除）

## 6. 触发逻辑

1. 行情 push 到达后进行规则匹配。
2. 命中后：
   - UI 顶部/底部提醒
   - 记录日志（symbol、规则 ID、触发值、时间）
3. 同一规则短时间内可配置去抖窗口（如 30 秒）。

## 7. 数据模型（建议）

```text
AlertRule {
  id: String,
  symbol: String,
  rule_type: RuleType,
  status: RuleStatus,
  created_at: i64,
  updated_at: i64
}

AlertEvent {
  rule_id: String,
  symbol: String,
  triggered_price: Decimal,
  triggered_at: i64
}
```

## 8. 存储策略

1. 本地 JSON 持久化（用户目录）
2. 启动时加载，退出时落盘
3. 格式错误时自动备份并重建空文件

## 9. 验收标准

1. 可创建 5 类规则，重启后规则不丢失
2. 命中规则后 1 秒内看到 UI 提醒
3. 误触发率低（手工回放验证）
4. 异常格式不会导致主程序崩溃

## 10. 埋点指标

1. `alert_rule_created`
2. `alert_rule_enabled` / `alert_rule_disabled`
3. `alert_rule_triggered`
4. `alert_panel_opened`

## 11. 风险

1. 高频 push 下匹配开销增加  
   - 对策：按 symbol 建索引，仅匹配相关规则
2. 频繁触发导致 UI 噪声  
   - 对策：去抖 + 冷却窗口
3. 本地文件损坏  
   - 对策：备份 + 容错解析 + 自动重建
