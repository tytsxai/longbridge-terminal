use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{OnceLock, RwLock};
use std::time::Duration;

use atomic::Atomic;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::{CommandQueue, InsertResource, SystemState};
use tokio::sync::mpsc;

use crate::data::{Counter, User, Watchlist, WatchlistGroup};
use crate::render::{DirtyFlags, RenderState};
use crate::system;
use crate::ui::Content;
use crate::widgets::{Carousel, Loading, LocalSearch, Search, Terminal};

pub static RT: OnceLock<tokio::runtime::Handle> = OnceLock::new();
pub static POPUP: AtomicU8 = AtomicU8::new(0);
pub static LAST_STATE: Atomic<AppState> = Atomic::new(AppState::Watchlist);
pub static QUOTE_BMP: Atomic<bool> = Atomic::new(false);
pub static LOG_PANEL_VISIBLE: Atomic<bool> = Atomic::new(false);
pub static WATCHLIST: std::sync::LazyLock<RwLock<Watchlist>> =
    std::sync::LazyLock::new(Default::default);
pub static USER: std::sync::LazyLock<RwLock<User>> = std::sync::LazyLock::new(Default::default);

pub const POPUP_HELP: u8 = 0b1;
pub const POPUP_SEARCH: u8 = 0b10;
pub const POPUP_ACCOUNT: u8 = 0b100;
pub const POPUP_CURRENCY: u8 = 0b1000;
pub const POPUP_WATCHLIST: u8 = 0b10000;

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States, strum::EnumIter, bytemuck::NoUninit,
)]
#[repr(u8)]
pub enum AppState {
    Error,
    #[default]
    Loading,
    TradeToken,
    Portfolio,
    Stock,
    Watchlist,
    WatchlistStock,
}

fn app_state_to_str(state: AppState) -> &'static str {
    match state {
        AppState::Error => "error",
        AppState::Loading => "loading",
        AppState::TradeToken => "trade_token",
        AppState::Portfolio => "portfolio",
        AppState::Stock => "stock",
        AppState::Watchlist => "watchlist",
        AppState::WatchlistStock => "watchlist_stock",
    }
}

fn app_state_from_str(raw: &str) -> Option<AppState> {
    match raw {
        "error" => Some(AppState::Error),
        "loading" => Some(AppState::Loading),
        "trade_token" => Some(AppState::TradeToken),
        "portfolio" => Some(AppState::Portfolio),
        "stock" => Some(AppState::Stock),
        "watchlist" => Some(AppState::Watchlist),
        "watchlist_stock" => Some(AppState::WatchlistStock),
        _ => None,
    }
}

fn persist_workspace_snapshot(app: &bevy_app::App) {
    let (selected_counter, watchlist_group_id, watchlist_sort_by, watchlist_hidden) = {
        let selected_idx = crate::system::WATCHLIST_TABLE
            .lock()
            .expect("poison")
            .selected();
        let selected = {
            let watchlist = WATCHLIST.read().expect("poison");
            selected_idx.and_then(|i| watchlist.counters().get(i).cloned())
        };
        let watchlist = WATCHLIST.read().expect("poison");
        (
            selected,
            watchlist.group_id,
            watchlist.sort_by,
            watchlist.hidden,
        )
    };

    let current_state = app
        .world
        .get_resource::<State<AppState>>()
        .map(|state| *state.get());
    let stock_detail_counter = app
        .world
        .get_resource::<crate::system::StockDetail>()
        .map(|detail| detail.0.clone());

    let mut snapshot = crate::workspace::WorkspaceSnapshot::empty_now();
    snapshot.last_state = current_state.map(app_state_to_str).map(str::to_string);
    snapshot.watchlist_group_id = watchlist_group_id;
    snapshot.watchlist_sort_by = watchlist_sort_by;
    snapshot.watchlist_hidden = watchlist_hidden;
    snapshot.selected_counter = selected_counter;
    snapshot.stock_detail_counter = stock_detail_counter;
    snapshot.kline_type = crate::system::KLINE_TYPE.load(Ordering::Relaxed);
    snapshot.kline_index = crate::system::KLINE_INDEX.load(Ordering::Relaxed);
    snapshot.log_panel_visible = LOG_PANEL_VISIBLE.load(Ordering::Relaxed);

    if let Err(err) = crate::workspace::save(&snapshot) {
        tracing::warn!(error = %err, "保存工作区快照失败");
    }
}

pub fn persist_workspace_fallback() {
    let selected_idx = crate::system::WATCHLIST_TABLE
        .lock()
        .expect("poison")
        .selected();
    let selected_counter = {
        let watchlist = WATCHLIST.read().expect("poison");
        selected_idx.and_then(|i| watchlist.counters().get(i).cloned())
    };
    let watchlist = WATCHLIST.read().expect("poison");
    let mut snapshot = crate::workspace::WorkspaceSnapshot::empty_now();
    snapshot.last_state = Some(app_state_to_str(LAST_STATE.load(Ordering::Relaxed)).to_string());
    snapshot.watchlist_group_id = watchlist.group_id;
    snapshot.watchlist_sort_by = watchlist.sort_by;
    snapshot.watchlist_hidden = watchlist.hidden;
    snapshot.selected_counter = selected_counter;
    snapshot.kline_type = crate::system::KLINE_TYPE.load(Ordering::Relaxed);
    snapshot.kline_index = crate::system::KLINE_INDEX.load(Ordering::Relaxed);
    snapshot.log_panel_visible = LOG_PANEL_VISIBLE.load(Ordering::Relaxed);
    if let Err(err) = crate::workspace::save(&snapshot) {
        tracing::warn!(error = %err, "保存工作区兜底快照失败");
    }
}

fn is_log_file_name(name: &str) -> bool {
    (name.starts_with("changqiao") || name.starts_with("longbridge"))
        && std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("log"))
}

fn latest_log_file_in(log_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    use std::fs;

    let mut log_files: Vec<std::path::PathBuf> = fs::read_dir(log_dir)
        .ok()?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(is_log_file_name)
        })
        .collect();

    log_files.sort_by(|a, b| {
        let time_a = fs::metadata(a).and_then(|m| m.modified()).ok();
        let time_b = fs::metadata(b).and_then(|m| m.modified()).ok();
        match (time_a, time_b) {
            (Some(ta), Some(tb)) => tb.cmp(&ta),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    log_files.into_iter().next()
}

#[allow(clippy::too_many_lines)]
pub async fn run(
    _args: crate::Args,
    mut quote_receiver: impl tokio_stream::Stream<Item = longport::quote::PushEvent> + Unpin,
) {
    let (update_tx, mut update_rx) = mpsc::unbounded_channel();

    let startup_workspace = crate::workspace::load().unwrap_or_else(|| {
        tracing::debug!("未检测到工作区快照，使用默认运行状态");
        crate::workspace::WorkspaceSnapshot::empty_now()
    });

    {
        let mut watchlist = WATCHLIST.write().expect("poison");
        watchlist.group_id = startup_workspace.watchlist_group_id;
        watchlist.set_sortby(startup_workspace.watchlist_sort_by);
        watchlist.set_hidden(startup_workspace.watchlist_hidden);
    }
    LOG_PANEL_VISIBLE.store(startup_workspace.log_panel_visible, Ordering::Relaxed);
    crate::system::KLINE_TYPE.store(startup_workspace.kline_type, Ordering::Relaxed);
    crate::system::KLINE_INDEX.store(startup_workspace.kline_index, Ordering::Relaxed);
    crate::workspace::set_startup_selected_counter(startup_workspace.selected_counter.clone());

    if let Err(err) = crate::alerts::load_from_disk() {
        tracing::warn!(error = %err, "加载预警规则失败，将继续使用空规则集合");
    } else {
        let count = crate::alerts::ALERT_STORE
            .read()
            .expect("poison")
            .rules
            .len();
        tracing::info!(count, "预警规则加载完成");
    }

    // Initialize index subscriptions
    let indexes: Vec<[Counter; 3]> = vec![
        [".DJI.US".into(), ".IXIC.US".into(), "SPY.US".into()],
        ["HSI.HK".into(), "HSCEI.HK".into(), "HSTECH.HK".into()],
        ["000001.SH".into(), "399001.SZ".into(), "399006.SZ".into()],
    ];

    // Subscribe to indexes and fetch initial data
    let subs: Vec<Counter> = indexes.iter().flatten().cloned().collect();
    tokio::spawn({
        let subs = subs.clone();
        async move {
            let ctx = crate::openapi::quote_limited();
            let symbols: Vec<String> = subs.iter().map(std::string::ToString::to_string).collect();

            // First, fetch initial quote data (includes prev_close)
            match ctx
                .execute("startup.index.quote", || {
                    let inner = ctx.inner();
                    let symbols = symbols.clone();
                    Box::pin(
                        async move { inner.quote(&symbols).await.map_err(anyhow::Error::from) },
                    )
                })
                .await
            {
                Ok(quotes) => {
                    tracing::info!("已获取 {} 条指数行情", quotes.len());
                    for quote in quotes {
                        let counter = Counter::new(&quote.symbol);
                        let mut stock = crate::data::Stock::new(counter);
                        stock.update_from_security_quote(&quote);
                        crate::data::STOCKS.insert(stock);
                    }
                }
                Err(e) => {
                    tracing::error!("获取指数行情失败：{}", e);
                }
            }

            // Then subscribe for real-time updates
            if let Err(e) = ctx
                .execute("startup.index.subscribe", || {
                    let inner = ctx.inner();
                    let symbols = symbols.clone();
                    Box::pin(async move {
                        inner
                            .subscribe(&symbols, longport::quote::SubFlags::QUOTE)
                            .await
                            .map_err(anyhow::Error::from)
                    })
                })
                .await
            {
                tracing::error!("订阅指数失败：{}", e);
            } else {
                tracing::info!("成功订阅 {} 个指数", symbols.len());
            }
        }
    });

    // Create search components
    let search_stock = Search::new(update_tx.clone(), |keyword| {
        Box::pin(async move {
            let query = crate::api::search::StockQuery {
                keyword,
                market: "HK,SG,SH,SZ,US".to_string(),
                product: "BK,ETF,IX,ST,WT".to_string(),
                account_channel: USER
                    .read()
                    .expect("poison")
                    .get_account_channel()
                    .to_string(),
            };
            crate::api::search::fetch_stock(&query)
                .await
                .map(|v| v.product_list)
                .unwrap_or_default()
        })
    });
    let search_watchlist = LocalSearch::new(Vec::<WatchlistGroup>::new(), |_keyword, _group| false);

    if RT.set(tokio::runtime::Handle::current()).is_err() {
        tracing::debug!("运行时句柄已初始化，复用已存在句柄");
    }
    let mut app = bevy_app::App::new();
    app.add_state::<AppState>()
        .add_event::<system::Key>()
        .add_event::<system::TuiEvent>()
        .init_resource::<Terminal>()
        .init_resource::<Loading>()
        .insert_resource(search_stock)
        .insert_resource(search_watchlist)
        .insert_resource(system::Command(update_tx.clone()))
        .insert_resource(Carousel::new(indexes, Duration::from_secs(5)))
        .insert_resource(system::WsState(crate::data::ReadyState::Open))
        .add_systems(Update, system::loading.run_if(in_state(AppState::Loading)))
        .add_systems(Update, system::error.run_if(in_state(AppState::Error)))
        .add_systems(OnExit(AppState::Watchlist), system::exit_watchlist)
        .add_systems(
            Update,
            system::render_watchlist.run_if(in_state(AppState::Watchlist)),
        )
        .add_systems(OnEnter(AppState::Stock), system::enter_stock)
        .add_systems(OnExit(AppState::Stock), system::exit_stock)
        .add_systems(
            Update,
            system::render_stock.run_if(in_state(AppState::Stock)),
        )
        .add_systems(OnEnter(AppState::WatchlistStock), system::enter_stock)
        .add_systems(OnExit(AppState::WatchlistStock), system::exit_stock)
        .add_systems(
            Update,
            system::render_watchlist_stock.run_if(in_state(AppState::WatchlistStock)),
        )
        .add_systems(OnEnter(AppState::Portfolio), system::enter_portfolio)
        .add_systems(OnExit(AppState::Portfolio), system::exit_portfolio)
        .add_systems(
            Update,
            system::render_portfolio.run_if(in_state(AppState::Portfolio)),
        );

    // Don't refresh watchlist when transitioning between Watchlist and WatchlistStock
    for v in <AppState as strum::IntoEnumIterator>::iter() {
        if v == AppState::Watchlist || v == AppState::WatchlistStock {
            continue;
        }
        for watch in [AppState::Watchlist, AppState::WatchlistStock] {
            app.add_systems(
                OnTransition { from: v, to: watch },
                system::enter_watchlist_common,
            );
            app.add_systems(
                OnTransition { from: watch, to: v },
                system::exit_watchlist_common,
            );
        }
    }

    // Get WebSocket receiver (already initialized in main.rs)
    // We need to re-acquire the receiver or pass it from main.rs
    // Skip WebSocket handling for now, focus on getting code to compile

    let restored_state = startup_workspace
        .last_state
        .as_deref()
        .and_then(app_state_from_str)
        .unwrap_or(AppState::Watchlist);
    let restored_stock_detail = startup_workspace.stock_detail_counter.clone();

    // Initialize account information
    tokio::spawn({
        let tx = update_tx.clone();
        let restored_stock_detail = restored_stock_detail.clone();
        async move {
            tracing::info!("正在获取账户列表...");
            match crate::api::account::fetch_account_list().await {
                Ok(accounts) => {
                    tracing::info!("成功获取 {} 个账户", accounts.status.len());
                    if accounts.status.is_empty() {
                        tracing::error!("未找到可用账户");
                        let mut queue = CommandQueue::default();
                        queue.push(InsertResource {
                            resource: Content::new(
                                t!("user.open_account.heading"),
                                t!("user.open_account.content"),
                            ),
                        });
                        queue.push(InsertResource {
                            resource: NextState(Some(AppState::Error)),
                        });
                        _ = tx.send(queue);
                        return;
                    }

                    // Set default account
                    let account = &accounts.status[0];
                    {
                        let mut user = USER.write().expect("poison");
                        user.account_channel.clone_from(&account.account_channel);
                        user.aaid.clone_from(&account.aaid);
                    }

                    let mut queue = CommandQueue::default();

                    // Add Select<Account> resource for Portfolio
                    queue.push(InsertResource {
                        resource: crate::widgets::Select::new(accounts.status.clone()),
                    });

                    queue.push(InsertResource {
                        resource: LocalSearch::new(accounts.status.clone(), |keyword, account| {
                            account
                                .account_name
                                .to_ascii_lowercase()
                                .contains(&keyword.to_ascii_lowercase())
                        }),
                    });

                    // Get currency list
                    if let Ok(currencies) =
                        crate::api::account::currencies(&account.account_channel)
                    {
                        queue.push(InsertResource {
                            resource: LocalSearch::new(currencies.clone(), |keyword, currency| {
                                currency
                                    .currency_iso
                                    .contains(&keyword.to_ascii_uppercase())
                            }),
                        });
                    }

                    let next_state = match restored_state {
                        AppState::Watchlist
                        | AppState::WatchlistStock
                        | AppState::Stock
                        | AppState::Portfolio => restored_state,
                        _ => AppState::Watchlist,
                    };

                    // Portfolio 页面要求 Portfolio 资源先就绪，这里优先回退到 Watchlist
                    let next_state = if next_state == AppState::Portfolio {
                        AppState::Watchlist
                    } else {
                        next_state
                    };

                    let next_state =
                        if matches!(next_state, AppState::Stock | AppState::WatchlistStock)
                            && restored_stock_detail.is_none()
                        {
                            AppState::Watchlist
                        } else {
                            next_state
                        };

                    if let Some(counter) = restored_stock_detail.clone() {
                        queue.push(InsertResource {
                            resource: crate::system::StockDetail(counter),
                        });
                    }

                    queue.push(InsertResource {
                        resource: NextState(Some(next_state)),
                    });
                    _ = tx.send(queue);

                    // Load watchlist data
                    tracing::info!("正在加载自选列表数据...");
                    system::refresh_watchlist(tx.clone());
                }
                Err(e) => {
                    tracing::error!("获取账户列表失败：{}", e);
                    let mut queue = CommandQueue::default();
                    queue.push(InsertResource {
                        resource: Content::new(t!("error.api.heading"), e.to_string()),
                    });
                    queue.push(InsertResource {
                        resource: NextState(Some(AppState::Error)),
                    });
                    _ = tx.send(queue);
                }
            }
        }
    });

    // Start log file watcher for auto-refresh when log panel is visible
    tokio::spawn({
        let tx = update_tx.clone();
        async move {
            use std::fs;
            use std::time::SystemTime;

            let mut last_modified: Option<SystemTime> = None;
            let mut last_size: u64 = 0;
            let log_dir = crate::logger::active_log_dir();

            tracing::debug!(log_dir = %log_dir.display(), "日志面板监听任务已启动");

            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Only check if log panel is visible
                if !LOG_PANEL_VISIBLE.load(Ordering::Relaxed) {
                    continue;
                }

                if let Some(log_file) = latest_log_file_in(&log_dir) {
                    if let Ok(metadata) = fs::metadata(&log_file) {
                        let modified = metadata.modified().ok();
                        let size = metadata.len();

                        // Check if file has been modified or size changed
                        if modified != last_modified || size != last_size {
                            last_modified = modified;
                            last_size = size;

                            // Trigger UI refresh by sending empty command queue
                            let queue = CommandQueue::default();
                            if tx.send(queue).is_err() {
                                tracing::debug!("应用事件通道已关闭，停止日志监听任务");
                                break;
                            }
                        }
                    }
                }
            }
        }
    });

    // FPS-based rendering: 30 FPS for smooth UI updates
    let render_interval = std::time::Duration::from_millis(33); // ~30 FPS
    let mut render_tick = tokio::time::interval(render_interval);
    render_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Wait briefly to ensure terminal is fully ready
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut events = crossterm::event::EventStream::new();
    let mut render_state = RenderState::new();
    // Initial render to display UI
    render_state.mark_all_dirty();

    loop {
        tokio::select! {
            // Render at fixed FPS
            _ = render_tick.tick() => {
                if render_state.needs_render() {
                    app.update();
                    render_state.clear();
                } else {
                    render_state.skip();
                }
            }
            // Handle commands (state changes, resource updates)
            Some(mut cmd) = update_rx.recv() => {
                cmd.apply(&mut app.world);
                // State changes typically affect all components
                render_state.mark_dirty(DirtyFlags::ALL);
            }
            // Handle quote push events (data updates)
            Some(push_event) = tokio_stream::StreamExt::next(&mut quote_receiver) => {
                // Handle WebSocket push events
                // PushEvent contains symbol and detail
                use longport::quote::PushEventDetail;

                let symbol = push_event.symbol;
                let counter = Counter::new(&symbol);
                 match push_event.detail {
                     PushEventDetail::Quote(quote) => {
                         tracing::debug!(
                             "行情更新：{} = {}，交易时段 = {:?}",
                             symbol,
                             quote.last_done,
                             quote.trade_session
                         );
                         crate::data::STOCKS.modify(counter.clone(), |stock| {
                             // Use update_from_push_quote to update all fields including trade_session
                             stock.update_from_push_quote(&quote);
                         });
                         if let Some(stock) = crate::data::STOCKS.get(&counter) {
                             crate::alerts::evaluate_quote(&symbol, stock.as_ref());
                         }
                         // Quote updates affect watchlist, stock detail, and indexes
                         render_state.mark_dirty(DirtyFlags::NONE.mark_quote_update());
                     }
                     PushEventDetail::Depth(depth) => {
                         tracing::debug!("深度更新：{}", symbol);
                         crate::data::STOCKS.modify(counter, |stock| {
                             use rust_decimal::Decimal;
                             // PushDepth structure may differ from SecurityDepth, update manually
                             stock.depth.asks = depth.asks.iter().map(|d| crate::data::Depth {
                                 position: d.position,
                                 price: d.price.unwrap_or(Decimal::ZERO),
                                 volume: d.volume,
                                 order_num: d.order_num,
                             }).collect();
                             stock.depth.bids = depth.bids.iter().map(|d| crate::data::Depth {
                                 position: d.position,
                                 price: d.price.unwrap_or(Decimal::ZERO),
                                 volume: d.volume,
                                 order_num: d.order_num,
                             }).collect();
                         });
                         // Depth updates only affect stock detail view and depth widget
                         render_state.mark_dirty(DirtyFlags::NONE.mark_depth_update());
                     }
                     _ => {
                         // Other event types not handled yet
                     }
                 }
            }
            // Handle user input events
            Some(event) = tokio_stream::StreamExt::next(&mut events) => {
                let event = match event {
                    Ok(crossterm::event::Event::Key(event)) => event,
                    Ok(_) => {
                        // Non-key events (mouse, resize, etc.) - ignore for now
                        continue
                    },
                    Err(err) => {
                        tracing::error!("接收事件失败：{err}");
                        app.world.insert_resource(Content::new(
                            t!("qrcode_view.error.heading"),
                            t!("qrcode_view.error.content"),
                        ));
                        app.world.insert_resource(NextState(Some(AppState::Error)));
                        render_state.mark_dirty(DirtyFlags::ERROR);
                        continue;
                    }
                };

                let popup = POPUP.load(Ordering::Relaxed);
                let state = *app.world.resource::<State<AppState>>().get();

                // Handle global shortcuts that should work even with popups open
                if event.code == crossterm::event::KeyCode::Char('`')
                    && event.modifiers == crossterm::event::KeyModifiers::NONE {
                    // Toggle log panel visibility
                    let was_visible = LOG_PANEL_VISIBLE.load(Ordering::Relaxed);
                    LOG_PANEL_VISIBLE.store(!was_visible, Ordering::Relaxed);
                    render_state.mark_dirty(DirtyFlags::ALL);
                    continue;
                }

                // Handle various popups
                if popup != 0 {
                    handle_popup_input(&mut app, popup, event, update_tx.clone());
                    render_state.mark_dirty(DirtyFlags::NONE.mark_popup_change(popup));
                    continue;
                }

                // Handle input for different states
                match state {
                    AppState::Error => {
                        persist_workspace_snapshot(&app);
                        return;
                    }
                    AppState::Loading => {
                        if matches!(event, ctrl!('c') | key!('q')) {
                            persist_workspace_snapshot(&app);
                            return;
                        }
                        continue;
                    },
                    AppState::TradeToken => {
                        match event {
                            ctrl!('c') => {
                                persist_workspace_snapshot(&app);
                                return;
                            }
                            key!(Esc) => {
                                app.world.insert_resource(NextState(Some(LAST_STATE.load(Ordering::Relaxed))));
                                render_state.mark_dirty(DirtyFlags::ALL);
                            }
                            _ => {
                                let evt = crossterm::event::Event::Key(event);
                                if let Some(evt) = tui_input::backend::crossterm::to_input_request(&evt) {
                                    send_evt(system::TuiEvent(evt), &mut app.world);
                                    render_state.mark_dirty(DirtyFlags::ALL);
                                }
                            }
                        }
                        continue;
                    }
                    AppState::Portfolio | AppState::Stock | AppState::Watchlist | AppState::WatchlistStock => (),
                }

                // Handle global keyboard shortcuts
                if handle_global_keys(
                    &mut app,
                    event,
                    state,
                    update_tx.clone(),
                    &mut render_state,
                ) {
                    persist_workspace_snapshot(&app);
                    return;
                }
            }
        }
    }
}

fn handle_popup_input(
    app: &mut bevy_app::App,
    popup: u8,
    event: crossterm::event::KeyEvent,
    update_tx: mpsc::UnboundedSender<CommandQueue>,
) {
    if popup == POPUP_ACCOUNT {
        let mut search = app
            .world
            .resource_mut::<LocalSearch<crate::data::Account>>();
        let (hidden, selected) = search.handle_key(event);
        if hidden {
            POPUP.store(0, Ordering::Relaxed);
        }
        if let Some(account) = selected {
            let mut user = USER.write().expect("poison");
            if user.get_account_channel() != account.account_channel {
                // TODO: Fetch currency list in background
            }
            user.account_channel = account.account_channel;
            user.aaid = account.aaid;
        }
    } else if popup == POPUP_CURRENCY {
        let mut search = app
            .world
            .resource_mut::<LocalSearch<crate::api::account::CurrencyInfo>>();
        let (hidden, selected) = search.handle_key(event);
        if hidden {
            POPUP.store(0, Ordering::Relaxed);
        }
        if let Some(currency) = selected {
            POPUP.store(0, Ordering::Relaxed);
            let mut user = USER.write().expect("poison");
            user.base_currency = currency.currency_iso;
        }
    } else if popup == POPUP_WATCHLIST {
        let mut search = app.world.resource_mut::<LocalSearch<WatchlistGroup>>();
        let (hidden, selected) = search.handle_key(event);
        if hidden {
            POPUP.store(0, Ordering::Relaxed);
        }
        if let Some(group) = selected {
            POPUP.store(0, Ordering::Relaxed);
            WATCHLIST.write().expect("poison").set_group_id(group.id);
            system::refresh_watchlist(update_tx.clone());
        }
    } else if popup == POPUP_SEARCH {
        let mut search = app
            .world
            .resource_mut::<Search<crate::api::search::StockItem>>();
        let (hidden, selected) = search.handle_key(event);
        if hidden {
            POPUP.store(0, Ordering::Relaxed);
        }
        if let Some(selected) = selected {
            POPUP.store(0, Ordering::Relaxed);
            app.world
                .insert_resource(system::StockDetail(selected.counter_id));
            let state = *app.world.resource::<State<AppState>>().get();
            let next_state = if state == AppState::Stock {
                AppState::Stock
            } else {
                AppState::WatchlistStock
            };
            app.world.insert_resource(NextState(Some(next_state)));
        }
    } else if popup == POPUP_HELP {
        POPUP.store(0, Ordering::Relaxed);
    }
}

#[allow(clippy::too_many_lines)]
fn handle_global_keys(
    app: &mut bevy_app::App,
    event: crossterm::event::KeyEvent,
    state: AppState,
    update_tx: mpsc::UnboundedSender<CommandQueue>,
    render_state: &mut RenderState,
) -> bool {
    match event {
        ctrl!('c') => return true,
        key!('1') if state != AppState::Watchlist => {
            app.world
                .insert_resource(NextState(Some(AppState::Watchlist)));
            render_state.mark_dirty(DirtyFlags::ALL);
        }
        key!('2') if state != AppState::Portfolio => {
            // Create default Portfolio resource if it doesn't exist
            if app.world.get_resource::<system::Portfolio>().is_none() {
                app.world.insert_resource(system::Portfolio {
                    props: system::portfolio::Props::default(),
                    view: system::portfolio::View::default(),
                });
            }
            app.world
                .insert_resource(NextState(Some(AppState::Portfolio)));
            render_state.mark_dirty(DirtyFlags::ALL);
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Char('a'),
            modifiers:
                ::crossterm::event::KeyModifiers::NONE | ::crossterm::event::KeyModifiers::SHIFT,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } if state == AppState::Portfolio => {
            if let Some(mut account) = app
                .world
                .get_resource_mut::<LocalSearch<crate::data::Account>>()
            {
                POPUP.store(POPUP_ACCOUNT, Ordering::Relaxed);
                account.visible();
                render_state.mark_dirty(DirtyFlags::POPUP_ACCOUNT);
            }
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Char('c'),
            modifiers:
                ::crossterm::event::KeyModifiers::NONE | ::crossterm::event::KeyModifiers::SHIFT,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } if state == AppState::Portfolio => {
            if let Some(mut currency) = app
                .world
                .get_resource_mut::<LocalSearch<crate::api::account::CurrencyInfo>>()
            {
                POPUP.store(POPUP_CURRENCY, Ordering::Relaxed);
                currency.visible();
                render_state.mark_dirty(DirtyFlags::POPUP_CURRENCY);
            }
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Char('g' | 'G'),
            modifiers: ::crossterm::event::KeyModifiers::NONE,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } if state == AppState::Watchlist || state == AppState::WatchlistStock => {
            if let Some(mut search) = app.world.get_resource_mut::<LocalSearch<WatchlistGroup>>() {
                POPUP.store(POPUP_WATCHLIST, Ordering::Relaxed);
                search.visible();
                render_state.mark_dirty(DirtyFlags::POPUP_WATCHLIST);
            }
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Char('Q'),
            modifiers:
                ::crossterm::event::KeyModifiers::NONE | ::crossterm::event::KeyModifiers::SHIFT,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } => {
            show_index(&mut app.world, 0);
            render_state.mark_dirty(DirtyFlags::STOCK_DETAIL | DirtyFlags::WATCHLIST);
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Char('W'),
            modifiers:
                ::crossterm::event::KeyModifiers::NONE | ::crossterm::event::KeyModifiers::SHIFT,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } => {
            show_index(&mut app.world, 1);
            render_state.mark_dirty(DirtyFlags::STOCK_DETAIL | DirtyFlags::WATCHLIST);
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Char('E'),
            modifiers:
                ::crossterm::event::KeyModifiers::NONE | ::crossterm::event::KeyModifiers::SHIFT,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } => {
            show_index(&mut app.world, 2);
            render_state.mark_dirty(DirtyFlags::STOCK_DETAIL | DirtyFlags::WATCHLIST);
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Char('t'),
            modifiers:
                ::crossterm::event::KeyModifiers::NONE | ::crossterm::event::KeyModifiers::SHIFT,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } => {
            if state == AppState::Stock {
                app.world
                    .insert_resource(NextState(Some(AppState::WatchlistStock)));
                render_state.mark_dirty(DirtyFlags::STOCK_DETAIL | DirtyFlags::WATCHLIST);
            } else if state == AppState::WatchlistStock {
                app.world.insert_resource(NextState(Some(AppState::Stock)));
                render_state.mark_dirty(DirtyFlags::STOCK_DETAIL | DirtyFlags::WATCHLIST);
            }
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Char('R'),
            modifiers:
                ::crossterm::event::KeyModifiers::NONE | ::crossterm::event::KeyModifiers::SHIFT,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } => match state {
            AppState::Portfolio => {
                system::refresh_portfolio();
                render_state.mark_dirty(DirtyFlags::PORTFOLIO);
            }
            AppState::Watchlist => {
                system::refresh_watchlist(update_tx.clone());
                render_state.mark_dirty(DirtyFlags::WATCHLIST);
            }
            AppState::WatchlistStock => {
                system::refresh_stock_debounced(
                    app.world.resource::<system::StockDetail>().0.clone(),
                );
                system::refresh_watchlist(update_tx.clone());
                render_state.mark_dirty(DirtyFlags::STOCK_DETAIL | DirtyFlags::WATCHLIST);
            }
            AppState::Stock => {
                system::refresh_stock_debounced(
                    app.world.resource::<system::StockDetail>().0.clone(),
                );
                render_state.mark_dirty(DirtyFlags::STOCK_DETAIL);
            }
            _ => {}
        },
        key!('?') => {
            POPUP.store(POPUP_HELP, Ordering::Relaxed);
            render_state.mark_dirty(DirtyFlags::POPUP_HELP);
        }
        key!('/') => {
            if let Some(mut search) = app
                .world
                .get_resource_mut::<Search<crate::api::search::StockItem>>()
            {
                POPUP.store(POPUP_SEARCH, Ordering::Relaxed);
                search.visible();
                render_state.mark_dirty(DirtyFlags::POPUP_SEARCH);
            }
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Esc | ::crossterm::event::KeyCode::Char('q'),
            modifiers: ::crossterm::event::KeyModifiers::NONE,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } => {
            let last_state = LAST_STATE.load(Ordering::Relaxed);
            if last_state != state {
                app.world.insert_resource(NextState(Some(last_state)));
                render_state.mark_dirty(DirtyFlags::ALL);
            }
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Up | ::crossterm::event::KeyCode::Char('k'),
            modifiers: ::crossterm::event::KeyModifiers::NONE,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        }
        | ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Char('k'),
            modifiers: ::crossterm::event::KeyModifiers::SHIFT,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } => {
            send_evt(system::Key::Up, &mut app.world);
            // Navigation keys affect current view
            render_state.mark_dirty(match state {
                AppState::Watchlist | AppState::WatchlistStock => DirtyFlags::WATCHLIST,
                AppState::Stock => DirtyFlags::STOCK_DETAIL,
                AppState::Portfolio => DirtyFlags::PORTFOLIO,
                _ => DirtyFlags::ALL,
            });
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Down | ::crossterm::event::KeyCode::Char('j'),
            modifiers: ::crossterm::event::KeyModifiers::NONE,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        }
        | ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Char('j'),
            modifiers: ::crossterm::event::KeyModifiers::SHIFT,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } => {
            send_evt(system::Key::Down, &mut app.world);
            render_state.mark_dirty(match state {
                AppState::Watchlist | AppState::WatchlistStock => DirtyFlags::WATCHLIST,
                AppState::Stock => DirtyFlags::STOCK_DETAIL,
                AppState::Portfolio => DirtyFlags::PORTFOLIO,
                _ => DirtyFlags::ALL,
            });
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Left | ::crossterm::event::KeyCode::Char('h'),
            modifiers: ::crossterm::event::KeyModifiers::NONE,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        }
        | ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Char('h'),
            modifiers: ::crossterm::event::KeyModifiers::SHIFT,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } => {
            send_evt(system::Key::Left, &mut app.world);
            render_state.mark_dirty(match state {
                AppState::Stock => DirtyFlags::STOCK_DETAIL,
                _ => DirtyFlags::ALL,
            });
        }
        ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Right | ::crossterm::event::KeyCode::Char('l'),
            modifiers: ::crossterm::event::KeyModifiers::NONE,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        }
        | ::crossterm::event::KeyEvent {
            code: ::crossterm::event::KeyCode::Char('l'),
            modifiers: ::crossterm::event::KeyModifiers::SHIFT,
            kind: ::crossterm::event::KeyEventKind::Press,
            state: ::crossterm::event::KeyEventState::NONE,
        } => {
            send_evt(system::Key::Right, &mut app.world);
            render_state.mark_dirty(match state {
                AppState::Stock => DirtyFlags::STOCK_DETAIL,
                _ => DirtyFlags::ALL,
            });
        }
        key!(Tab) => {
            send_evt(system::Key::Tab, &mut app.world);
            render_state.mark_dirty(match state {
                AppState::Stock => DirtyFlags::STOCK_DETAIL,
                _ => DirtyFlags::ALL,
            });
        }
        key!(Enter) => {
            send_evt(system::Key::Enter, &mut app.world);
            render_state.mark_dirty(DirtyFlags::ALL);
        }
        shift!(BackTab) => {
            send_evt(system::Key::BackTab, &mut app.world);
            render_state.mark_dirty(match state {
                AppState::Stock => DirtyFlags::STOCK_DETAIL,
                _ => DirtyFlags::ALL,
            });
        }
        _ => (),
    }
    false
}

fn send_evt<T: Event>(evt: T, world: &mut World) {
    let mut state = SystemState::<EventWriter<T>>::new(world);
    state.get_mut(world).send(evt);
}

fn show_index(world: &mut World, index: usize) {
    let indexes = world.resource::<Carousel<[Counter; 3]>>().current();
    world.insert_resource(system::StockDetail(indexes[index].clone()));
    world.insert_resource(NextState(Some(AppState::WatchlistStock)));
}

#[cfg(test)]
mod tests {
    use super::{is_log_file_name, latest_log_file_in};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new() -> Self {
            let unique = format!(
                "changqiao-app-tests-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or_default()
            );
            let path = std::env::temp_dir().join(unique);
            fs::create_dir_all(&path).expect("failed to create temp dir");
            Self { path }
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn accepts_expected_log_filenames() {
        assert!(is_log_file_name("changqiao.log"));
        assert!(is_log_file_name("changqiao.2026-02-12.log"));
        assert!(is_log_file_name("longbridge.log"));
        assert!(!is_log_file_name("changqiao.txt"));
        assert!(!is_log_file_name("other.log"));
    }

    #[test]
    fn returns_latest_log_file() {
        let temp_dir = TempDirGuard::new();

        let old_log = temp_dir.path.join("changqiao.old.log");
        let new_log = temp_dir.path.join("changqiao.new.log");

        fs::write(&old_log, "old").expect("failed to write old log");
        std::thread::sleep(Duration::from_millis(20));
        fs::write(&new_log, "new").expect("failed to write new log");

        let selected = latest_log_file_in(&temp_dir.path).expect("latest log not found");
        assert_eq!(selected, new_log);
    }
}
