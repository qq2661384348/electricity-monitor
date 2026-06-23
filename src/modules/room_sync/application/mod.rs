use std::sync::{Arc, OnceLock};

use chrono::Utc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::{
    config::AppConfig,
    domain::{
        models::{RoomAggregate, RoomSyncLog, UpdateRoomSyncLog},
        services::{
            ElectricityFetcherService, RoomClient, RoomFetcher, RoomPathTree, RoomSyncCache,
            RoomSyncService,
        },
    },
    errors::{AppError, Result},
    infrastructure::{repositories::RoomRepository, DbPool},
    state::AppState,
};

static MANUAL_SYNC_JOBS: OnceLock<Mutex<ManualSyncJobRegistry>> = OnceLock::new();

#[derive(Default)]
struct ManualSyncJobRegistry {
    running_job_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManualSyncStart {
    Started(Uuid),
    AlreadyRunning(Uuid),
}

impl ManualSyncStart {
    fn job_id(self) -> Uuid {
        match self {
            Self::Started(job_id) | Self::AlreadyRunning(job_id) => job_id,
        }
    }

    fn should_spawn(self) -> bool {
        matches!(self, Self::Started(_))
    }
}

impl ManualSyncJobRegistry {
    fn begin(&mut self, new_job_id: Uuid) -> ManualSyncStart {
        if let Some(job_id) = self.running_job_id {
            return ManualSyncStart::AlreadyRunning(job_id);
        }

        self.running_job_id = Some(new_job_id);
        ManualSyncStart::Started(new_job_id)
    }

    fn finish(&mut self, job_id: Uuid) {
        if self.running_job_id == Some(job_id) {
            self.running_job_id = None;
        }
    }
}

fn manual_sync_jobs() -> &'static Mutex<ManualSyncJobRegistry> {
    MANUAL_SYNC_JOBS.get_or_init(|| Mutex::new(ManualSyncJobRegistry::default()))
}

async fn begin_manual_sync_job(job_id: Uuid) -> ManualSyncStart {
    manual_sync_jobs().lock().await.begin(job_id)
}

async fn finish_manual_sync_job(job_id: Uuid) {
    manual_sync_jobs().lock().await.finish(job_id);
}

#[derive(Clone)]
pub struct RoomSyncUseCase {
    db_pool: DbPool,
    repository: Arc<RoomRepository>,
    electricity_fetcher: Option<Arc<ElectricityFetcherService>>,
    room_path_tree: Arc<RwLock<RoomPathTree>>,
}

impl RoomSyncUseCase {
    pub fn from_state(state: &AppState) -> Self {
        Self {
            db_pool: state.db_pool.clone(),
            repository: Arc::new(RoomRepository::new(state.db_pool.clone())),
            electricity_fetcher: state.electricity_fetcher_service.clone(),
            room_path_tree: state.room_path_tree.clone(),
        }
    }

    pub async fn trigger_sync(&self) -> Result<Uuid> {
        let start = begin_manual_sync_job(Uuid::new_v4()).await;
        let job_id = start.job_id();

        if !start.should_spawn() {
            tracing::info!(
                task_name = "manual_room_sync",
                module = "room_sync",
                job_id = %job_id,
                "手动同步任务已在运行，复用现有任务ID"
            );
            return Ok(job_id);
        }

        let repository = self.repository.clone();
        let db_pool = self.db_pool.clone();
        let electricity_fetcher = self.electricity_fetcher.clone();
        let room_path_tree = self.room_path_tree.clone();
        let config = AppConfig::global().clone();

        tokio::spawn(async move {
            run_manual_sync_job(
                job_id,
                repository,
                db_pool,
                electricity_fetcher,
                room_path_tree,
                config,
            )
            .await;
            finish_manual_sync_job(job_id).await;
        });

        Ok(job_id)
    }

    pub async fn get_room_paths(&self, roomid: i64) -> Result<RoomAggregate> {
        self.repository
            .find_room_with_all_paths(roomid)
            .await?
            .ok_or(AppError::NotFound)
    }

    pub async fn get_sync_status(&self, job_id: Uuid) -> Result<Option<RoomSyncLog>> {
        self.repository.get_sync_log(job_id).await
    }

    pub async fn get_sync_history(&self, limit: i64) -> Result<Vec<RoomSyncLog>> {
        self.repository.get_sync_history(limit).await
    }
}

async fn run_manual_sync_job(
    job_id: Uuid,
    repository: Arc<RoomRepository>,
    db_pool: DbPool,
    electricity_fetcher: Option<Arc<ElectricityFetcherService>>,
    room_path_tree: Arc<RwLock<RoomPathTree>>,
    config: AppConfig,
) {
    tracing::info!(task_name = "manual_room_sync", module = "room_sync", job_id = %job_id, "开始异步同步任务");

    let new_log = crate::domain::models::NewRoomSyncLog {
        id: Some(job_id),
        sync_type: "manual".to_string(),
        started_at: Utc::now().naive_utc(),
        status: "running".to_string(),
    };

    if let Err(error) = repository.create_sync_log(new_log).await {
        tracing::error!(task_name = "manual_room_sync", module = "room_sync", job_id = %job_id, error = %error, "创建同步日志失败");
        return;
    }

    let client = match RoomClient::new(&config.room_sync.crawler) {
        Ok(client) => Arc::new(client),
        Err(error) => {
            let _ = repository
                .update_sync_log(
                    job_id,
                    UpdateRoomSyncLog {
                        completed_at: Some(Utc::now().naive_utc()),
                        status: Some("failed".to_string()),
                        stats: None,
                        error_message: Some(format!("创建爬虫客户端失败: {}", error)),
                    },
                )
                .await;
            tracing::error!(task_name = "manual_room_sync", module = "room_sync", job_id = %job_id, error = %error, "创建爬虫客户端失败");
            return;
        }
    };

    let fetcher = Arc::new(RoomFetcher::new(client));
    let room_sync_cache = match RoomSyncCache::new(db_pool.clone()).await {
        Ok(cache) => Arc::new(cache),
        Err(error) => {
            let _ = repository
                .update_sync_log(
                    job_id,
                    UpdateRoomSyncLog {
                        completed_at: Some(Utc::now().naive_utc()),
                        status: Some("failed".to_string()),
                        stats: None,
                        error_message: Some(format!("创建RoomSyncCache失败: {}", error)),
                    },
                )
                .await;
            tracing::error!(task_name = "manual_room_sync", module = "room_sync", job_id = %job_id, error = %error, "创建RoomSyncCache失败");
            return;
        }
    };

    let sync_service = RoomSyncService::new(
        Arc::new(db_pool.clone()),
        repository.clone(),
        fetcher,
        room_sync_cache,
        config.room_sync.default_threshold,
    );

    match sync_service.sync().await {
        Ok(stats) => {
            let stats_json = serde_json::to_value(&stats).ok();
            let _ = repository
                .update_sync_log(
                    job_id,
                    UpdateRoomSyncLog {
                        completed_at: Some(Utc::now().naive_utc()),
                        status: Some("completed".to_string()),
                        stats: stats_json,
                        error_message: None,
                    },
                )
                .await;

            if let Some(fetcher_service) = electricity_fetcher.as_ref() {
                if let Err(error) = fetcher_service.refresh_cache().await {
                    tracing::error!(task_name = "manual_room_sync", module = "room_sync", job_id = %job_id, error = %error, "刷新电费缓存失败");
                }
            }

            match repository.find_all_active_path_entries().await {
                Ok(path_entries) => {
                    let tree = RoomPathTree::build_from_path_entries(path_entries);
                    let mut current_tree = room_path_tree.write().await;
                    *current_tree = tree;
                }
                Err(error) => {
                    tracing::error!(task_name = "manual_room_sync", module = "room_sync", job_id = %job_id, error = %error, "重建路径树失败");
                }
            }
        }
        Err(error) => {
            let _ = repository
                .update_sync_log(
                    job_id,
                    UpdateRoomSyncLog {
                        completed_at: Some(Utc::now().naive_utc()),
                        status: Some("failed".to_string()),
                        stats: None,
                        error_message: Some(error.to_string()),
                    },
                )
                .await;
            tracing::error!(task_name = "manual_room_sync", module = "room_sync", job_id = %job_id, error = %error, "同步任务失败");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_sync_registry_reuses_running_job_id() {
        let mut registry = ManualSyncJobRegistry::default();
        let running_job_id = Uuid::new_v4();
        let duplicate_job_id = Uuid::new_v4();

        assert_eq!(
            registry.begin(running_job_id),
            ManualSyncStart::Started(running_job_id)
        );
        assert_eq!(
            registry.begin(duplicate_job_id),
            ManualSyncStart::AlreadyRunning(running_job_id)
        );

        registry.finish(duplicate_job_id);
        assert_eq!(registry.running_job_id, Some(running_job_id));

        registry.finish(running_job_id);
        assert_eq!(registry.running_job_id, None);
    }
}
