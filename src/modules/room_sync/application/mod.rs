use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
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
        let job_id = Uuid::new_v4();
        let repository = self.repository.clone();
        let db_pool = self.db_pool.clone();
        let electricity_fetcher = self.electricity_fetcher.clone();
        let room_path_tree = self.room_path_tree.clone();
        let config = AppConfig::global().clone();

        tokio::spawn(async move {
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

                    match repository.find_all_active().await {
                        Ok(rooms) => {
                            let room_data = rooms
                                .iter()
                                .map(|room| {
                                    crate::domain::services::room_sync::crawler::models::RoomData {
                                        roomid: room.roomid,
                                        roompaths: vec![room.primary_roompath.clone()],
                                        primary_roompath: room.primary_roompath.clone(),
                                        path_count: if room.has_additional_paths { 2 } else { 1 },
                                    }
                                })
                                .collect::<Vec<_>>();

                            let tree = RoomPathTree::build_from_rooms(&room_data);
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
        });

        Ok(job_id)
    }

    pub async fn get_room_paths(&self, roomid: i32) -> Result<RoomAggregate> {
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
