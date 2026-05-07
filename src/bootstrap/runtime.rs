use std::sync::Arc;

use crate::{
    config::AppConfig,
    domain::services::{
        spawn_recovery_monitor_persistent, ElectricityFetcherService, ElectricityService,
        NotificationChannels, NotificationGate, NotificationService, RoomPathTree, RoomSyncCache,
        RoomSyncService,
    },
    infrastructure::{
        email::{EmailDelivery, EmailSender},
        repositories::{RoomRepository, UserRepository, UserRoomBindingRepository},
        DbPool, QQClient, RedisPool,
    },
    state::AppState,
};

#[derive(Clone, Copy)]
enum TaskFailureCategory {
    Init,
    Dependency,
    Query,
}

fn log_task_started(task_name: &str, module: &str) {
    tracing::info!(task_name = task_name, module = module, "后台任务启动");
}

fn log_task_failed(task_name: &str, module: &str, category: TaskFailureCategory, error: &str) {
    let failure_category = match category {
        TaskFailureCategory::Init => "init",
        TaskFailureCategory::Dependency => "dependency",
        TaskFailureCategory::Query => "query",
    };

    tracing::error!(
        task_name = task_name,
        module = module,
        failure_category = failure_category,
        error = error,
        "后台任务失败"
    );
}

pub async fn run_startup_room_sync(config: &AppConfig, db_pool: &DbPool) -> anyhow::Result<()> {
    if !config.room_sync.enabled {
        return Ok(());
    }

    tracing::info!("检查是否需要启动时房间同步...");

    let room_repo = RoomRepository::new(db_pool.clone());
    let room_count = room_repo.count_all().await?;

    if room_count != 0 {
        tracing::info!(
            room_count = room_count,
            "数据库已有房间数据，跳过启动时同步"
        );
        return Ok(());
    }

    tracing::info!("数据库无房间数据，触发启动时自动同步");
    log_task_started("startup_room_sync", "room_sync");

    let client = Arc::new(
        crate::domain::services::room_sync::crawler::RoomClient::new(&config.room_sync.crawler)?,
    );
    let fetcher = Arc::new(crate::domain::services::room_sync::crawler::RoomFetcher::new(client));

    let room_sync_cache = Arc::new(
        RoomSyncCache::new(db_pool.clone())
            .await
            .map_err(|error| anyhow::anyhow!("创建RoomSyncCache失败: {}", error))?,
    );

    let sync_service = RoomSyncService::new(
        Arc::new(db_pool.clone()),
        Arc::new(room_repo.clone()),
        fetcher,
        room_sync_cache,
        config.room_sync.default_threshold,
    );

    match sync_service.sync().await {
        Ok(stats) => {
            tracing::info!(
                total = stats.total,
                new = stats.new,
                updated = stats.updated,
                failed = stats.failed,
                "启动时房间同步完成"
            );
        }
        Err(error) => {
            log_task_failed(
                "startup_room_sync",
                "room_sync",
                TaskFailureCategory::Dependency,
                &error.to_string(),
            );
            tracing::error!(error = %error, "启动时房间同步失败，将继续启动但可能无数据");
        }
    }

    Ok(())
}

pub async fn initialize_electricity_fetcher_service(
    config: &AppConfig,
    db_pool: &DbPool,
    redis_pool: &RedisPool,
) -> anyhow::Result<Option<Arc<ElectricityFetcherService>>> {
    if !config.electricity_fetcher.enabled {
        tracing::info!("Electricity Fetcher Service disabled");
        return Ok(None);
    }

    tracing::info!("Initializing Electricity Fetcher Service...");
    log_task_started("electricity_fetcher_scheduler", "electricity");

    let service = ElectricityFetcherService::new(
        config.electricity_fetcher.api_url.clone(),
        db_pool.clone(),
        redis_pool.clone(),
    )
    .await?;

    let service = Arc::new(service);

    let scheduler = ElectricityFetcherService::start_scheduler(
        service.clone(),
        config.electricity_fetcher.fetch_interval_minutes,
        config.electricity_fetcher.history_interval_hours,
        config.electricity_fetcher.max_retries,
        config.electricity_fetcher.retry_delay_seconds,
        config.electricity_fetcher.retry_backoff_multiplier,
    )
    .await?;

    scheduler.start().await?;
    tracing::info!(
        "Electricity Fetcher Service started (fetch: {}min, history: {}h, retries: {}, delay: {}s, backoff: {}x)",
        config.electricity_fetcher.fetch_interval_minutes,
        config.electricity_fetcher.history_interval_hours,
        config.electricity_fetcher.max_retries,
        config.electricity_fetcher.retry_delay_seconds,
        config.electricity_fetcher.retry_backoff_multiplier
    );

    Ok(Some(service))
}

pub fn initialize_email_sender(
    config: &AppConfig,
) -> anyhow::Result<Option<Arc<dyn EmailDelivery>>> {
    if !config.email.is_delivery_configured() {
        tracing::info!("Email sender disabled: SMTP delivery is not fully configured");
        return Ok(None);
    }

    let sender = EmailSender::new(config.email.clone())
        .map_err(|error| anyhow::anyhow!("邮件发送器初始化失败: {}", error))?;
    tracing::info!("Email sender initialized");
    Ok(Some(Arc::new(sender)))
}

pub async fn initialize_path_tree(state: &AppState, db_pool: &DbPool) {
    tracing::info!("正在初始化房间路径树...");

    let room_repo = RoomRepository::new(db_pool.clone());

    match room_repo.find_all_active_path_entries().await {
        Ok(rooms) => {
            let room_count = rooms.len();
            let tree = RoomPathTree::build_from_primary_paths(rooms);
            state.update_path_tree(tree).await;
            tracing::info!("房间路径树初始化完成，包含 {} 个房间", room_count);
        }
        Err(error) => {
            tracing::warn!("初始化路径树失败: {}，将使用空树", error);
        }
    }
}

pub fn spawn_background_services(state: AppState) {
    let db_pool = state.db_pool.clone();
    let redis_pool = state.redis_pool.clone();
    let rate_limiter = state.rate_limiter.clone();

    let room_repository = RoomRepository::new(db_pool.clone());
    let user_repository = UserRepository::new(db_pool.clone());
    let binding_repository = UserRoomBindingRepository::new(db_pool.clone());

    let electricity_service = ElectricityService::new(
        room_repository.clone(),
        redis_pool.clone(),
        rate_limiter.clone(),
    );
    electricity_service.spawn_worker();
    log_task_started("electricity_insert_worker", "electricity");
    tracing::info!("Electricity insertion service started");

    let config = AppConfig::global();
    let email_sender = state.email_sender.clone();
    match QQClient::new(config.qq_bot.clone()) {
        Ok(qq_client) => {
            let qq_client = Arc::new(qq_client);
            let notification_gate = Arc::new(NotificationGate::new(Some(
                std::time::Duration::from_secs(config.notification.debounce_period_secs),
            )));
            tracing::info!(
                "NotificationGate created (debounce_period: {}s)",
                config.notification.debounce_period_secs
            );

            let gate_for_init = notification_gate.clone();
            let binding_repo_for_init = binding_repository.clone();
            let room_repo_for_init = room_repository.clone();
            let recovery_interval = config.notification.recovery_monitor_interval_secs;

            tokio::spawn(async move {
                log_task_started("notification_history_loader", "notification");
                if let Err(error) = gate_for_init
                    .load_from_database(&binding_repo_for_init, &room_repo_for_init)
                    .await
                {
                    log_task_failed(
                        "notification_history_loader",
                        "notification",
                        TaskFailureCategory::Query,
                        &error.to_string(),
                    );
                    tracing::error!(
                        "Failed to load notification history from database: {}",
                        error
                    );
                } else {
                    tracing::info!("Notification history loaded from database");
                }

                let _recovery_monitor_handle = spawn_recovery_monitor_persistent(
                    room_repo_for_init,
                    binding_repo_for_init,
                    gate_for_init,
                    recovery_interval,
                );
                log_task_started("notification_recovery_monitor", "notification");
                tracing::info!(
                    "Recovery monitor started with persistence (interval: {}s)",
                    recovery_interval
                );
            });

            let notification_service = NotificationService::new(
                room_repository.clone(),
                user_repository,
                binding_repository,
                NotificationChannels::new(qq_client, email_sender),
                rate_limiter.clone(),
                notification_gate,
                &config.notification,
            );
            notification_service.spawn_worker();
            log_task_started("notification_worker", "notification");
            tracing::info!(
                "Notification service started (interval: {}s, concurrent: {})",
                config.notification.query_interval_secs,
                config.notification.concurrent_send_limit
            );
        }
        Err(error) => {
            log_task_failed(
                "notification_worker",
                "notification",
                TaskFailureCategory::Init,
                &error.to_string(),
            );
            tracing::error!("QQ客户端初始化失败，通知服务未启动: {}", error);
        }
    }
}
