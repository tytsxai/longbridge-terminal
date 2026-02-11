# 频率限制实现

## 概述

本文档描述了为遵守 Longport API "每秒不超过 10 次调用" 限制而实施的频率限制系统。

## 架构

### 组件

1. **RateLimiter** (`src/openapi/rate_limiter.rs`)
   - 令牌桶算法实现
   - 配置为 10 次请求/秒，突发容量为 20
   - 基于经过的时间自动补充令牌
   - 内置带有指数退避的重试机制

2. **Rate-Limited Context Wrappers** (`src/openapi/wrapper.rs`)
   - `RateLimitedQuoteContext`: 包装 `QuoteContext` 并增加频率限制
   - `RateLimitedTradeContext`: 包装 `TradeContext` 并增加频率限制
   - 提供对内部上下文和限流器的访问

3. **Helper Functions** (`src/openapi/helpers.rs`)
   - 常用 API 操作的便捷函数
   - 内置自动频率限制
   - 为常见用例简化 API

## 用法

### 选项 1: 使用 Helper Functions (推荐用于常用操作)

```rust
use crate::openapi::helpers;

// 订阅行情
helpers::subscribe_quotes(
    vec!["700.HK", "AAPL.US"],
    longport::quote::SubFlags::QUOTE
).await?;

// 获取行情
let quotes = helpers::get_quotes(vec!["700.HK"]).await?;

// 获取账户余额
let balance = helpers::get_account_balance(None).await?;
```

### 选项 2: 直接使用带限流的 Context

```rust
use crate::openapi::quote_limited;

let ctx = quote_limited();

// 执行带限流的 API 调用
ctx.execute("custom_operation", || {
    let inner = ctx.inner();
    Box::pin(async move {
        inner.some_api_method().await.map_err(anyhow::Error::from)
    })
}).await?;
```

### 选项 3: 手动限流

```rust
use crate::openapi::global_rate_limiter;

let limiter = global_rate_limiter();

// 在 API 调用前获取令牌
limiter.acquire().await;

// 进行 API 调用
let result = ctx.some_api_method().await?;
```

## 配置

限流器在 `src/openapi/rate_limiter.rs` 中配置:

```rust
RateLimiter::new(
    10,  // tokens_per_second: 每秒最大请求数
    20,  // max_tokens: 突发容量
)
```

### 参数

- **tokens_per_second**: 10 (Longport API 限制)
- **max_tokens**: 20 (允许短时间内的突发流量而不被节流)

## 特性

### 1. 令牌桶算法

- 令牌以恒定速率补充 (10/秒)
- 突发容量允许流量的暂时峰值
- 平滑的频率限制，无硬性阻塞

### 2. 自动重试

- 检测限流错误 (429, "rate limit", "too many requests")
- 指数退避: 1s → 2s → 4s
- 放弃前最多重试 3 次

### 3. 监控

```rust
let limiter = global_rate_limiter();
let available = limiter.available_tokens();
tracing::info!("Available rate limit tokens: {}", available);
```

### 4. 线程安全

- 使用 `tokio::sync::Semaphore` 进行令牌管理
- 兼容 Bevy ECS 和 async/tokio 架构
- 无锁或阻塞操作

## 迁移指南

### 没有限流的现有代码

```rust
// 之前
let ctx = crate::openapi::quote();
let quotes = ctx.quote(&symbols).await?;
```

### 迁移后带限流的代码

```rust
// 之后 (选项 1: Helper function)
let quotes = crate::openapi::helpers::get_quotes(&symbols).await?;

// 之后 (选项 2: Rate-limited context)
let ctx = crate::openapi::quote_limited();
ctx.execute("quote", || {
    let inner = ctx.inner();
    let symbols = symbols.clone();
    Box::pin(async move {
        inner.quote(&symbols).await.map_err(anyhow::Error::from)
    })
}).await?;
```

## 测试

### 单元测试

运行限流器测试:

```bash
cargo test --lib rate_limiter
```

### 集成测试

监控限流器的实际运行:

1. 启用调试日志:
   ```bash
   export RUST_LOG=debug
   ```

2. 查找限流器日志:
   ```
   Rate limiter: token acquired, available permits: 19
   Rate limiter: refilled 5 tokens, total available: 15
   ```

### 性能影响

- 令牌获取: < 1µs (当有令牌可用时)
- 限流延迟: 基于令牌可用性动态计算
- 重试延迟: 指数退避 (1s, 2s, 4s)

## 故障排除

### 仍然遇到限流错误

1. 检查是否所有 API 调用都使用了限流
2. 验证令牌配置 (应为 10/秒)
3. 检查来自多个任务的并发 API 调用

### 性能缓慢

1. 检查突发容量 (正常运行应为 20)
2. 减少并发 API 调用模式
3. 尽可能批量处理 API 请求

### 调试日志

启用调试日志以查看限流器活动:

```bash
export RUST_LOG=longbridge=debug
cargo run
```

## 未来改进

1. **请求批处理**: 自动将多个 API 调用批量处理为单个请求
2. **优先级队列**: 优先处理关键操作而非后台更新
3. **自适应限流**: 基于 API 响应头动态调整速率
4. **指标仪表板**: API 使用情况和限流状态的实时监控
5. **每端点限制**: 不同 API 端点的不同速率限制

## 参考资料

- Longport API 文档: https://open.longbridge.com
- 令牌桶算法: https://en.wikipedia.org/wiki/Token_bucket
- Tokio Semaphore: https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html
