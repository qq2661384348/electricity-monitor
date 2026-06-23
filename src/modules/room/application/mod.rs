use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    domain::{
        models::{NewRoom, Room},
        services::RoomPathTree,
    },
    errors::{AppError, Result},
    infrastructure::{
        repositories::{RoomRepository, UserRoomBindingRepository},
        CacheManager,
    },
    modules::room::{
        domain::RoomActor,
        infrastructure::{BindingQueries, RoomCommands, RoomQueries},
    },
    state::AppState,
};

#[derive(Clone)]
pub struct RoomAccessUseCase {
    room_queries: RoomQueries,
    room_commands: RoomCommands,
    binding_queries: BindingQueries,
    cache_manager: Arc<CacheManager>,
    path_tree: Arc<RwLock<RoomPathTree>>,
}

impl RoomAccessUseCase {
    pub fn from_state(state: &AppState) -> Self {
        Self::new(
            RoomRepository::new(state.db_pool.clone()),
            UserRoomBindingRepository::new(state.db_pool.clone()),
            state.cache_manager.clone(),
            state.room_path_tree.clone(),
        )
    }

    pub fn new(
        room_repository: RoomRepository,
        binding_repository: UserRoomBindingRepository,
        cache_manager: Arc<CacheManager>,
        path_tree: Arc<RwLock<RoomPathTree>>,
    ) -> Self {
        Self {
            room_queries: RoomQueries::new(room_repository.clone()),
            room_commands: RoomCommands::new(room_repository),
            binding_queries: BindingQueries::new(binding_repository),
            cache_manager,
            path_tree,
        }
    }

    #[instrument(skip(self, actor), fields(module = "room", use_case = "get_room", room_id = %id))]
    pub async fn get_room(&self, actor: &RoomActor, id: Uuid) -> Result<Room> {
        let room = self
            .room_queries
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;
        self.ensure_room_access(actor, room.roomid).await?;
        Ok(room)
    }

    #[instrument(skip(self, actor), fields(module = "room", use_case = "get_room_by_roomid", roomid = roomid))]
    pub async fn get_room_by_roomid(&self, actor: &RoomActor, roomid: i64) -> Result<Room> {
        let room = self
            .cache_manager
            .get_room(roomid)
            .await?
            .ok_or(AppError::NotFound)?;
        self.ensure_room_access(actor, roomid).await?;
        Ok(room)
    }

    #[instrument(skip(self, actor), fields(module = "room", use_case = "update_threshold", room_id = %id))]
    pub async fn update_threshold(
        &self,
        actor: &RoomActor,
        id: Uuid,
        threshold: f32,
    ) -> Result<Room> {
        let room = self
            .room_queries
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;
        self.ensure_room_access(actor, room.roomid).await?;
        let updated = self.room_commands.update_threshold(id, threshold).await?;
        self.cache_manager.invalidate_room(updated.roomid).await?;
        Ok(updated)
    }

    #[instrument(skip(self, new_room), fields(module = "room", use_case = "create_room", roomid = new_room.roomid))]
    pub async fn create_room(&self, new_room: NewRoom) -> Result<Room> {
        let room = self.room_commands.create(new_room).await?;
        self.cache_manager.invalidate_room(room.roomid).await?;
        Ok(room)
    }

    #[instrument(skip(self), fields(module = "room", use_case = "reset_send_flag", room_id = %id))]
    pub async fn reset_send_flag(&self, id: Uuid) -> Result<Room> {
        let room = self.room_commands.reset_send_flag(id).await?;
        self.cache_manager.invalidate_room(room.roomid).await?;
        Ok(room)
    }

    pub async fn delete_room(&self, id: Uuid) -> Result<usize> {
        self.room_commands.delete(id).await
    }

    #[instrument(
        skip(self, actor),
        fields(module = "room", use_case = "get_flagged_rooms")
    )]
    pub async fn get_flagged_rooms(&self, actor: &RoomActor) -> Result<Vec<Room>> {
        let rooms = self.room_queries.find_flagged().await?;

        if actor.is_admin {
            return Ok(rooms);
        }

        let bound_roomids = self.bound_roomids(actor).await?;
        Ok(rooms
            .into_iter()
            .filter(|room| bound_roomids.contains(&room.roomid))
            .collect())
    }

    #[instrument(skip(self, actor), fields(module = "room", use_case = "list_rooms", limit = limit, offset = offset))]
    pub async fn list_rooms(
        &self,
        actor: &RoomActor,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Room>> {
        if actor.is_admin {
            return self.room_queries.find_all(limit, offset).await;
        }

        let bound_roomids = self.bound_roomids(actor).await?;
        let roomids = bound_roomids.into_iter().collect::<Vec<_>>();
        self.room_queries
            .find_by_roomids_paged(&roomids, limit, offset)
            .await
    }

    #[instrument(skip(self, actor), fields(module = "room", use_case = "get_room_by_path", path = path))]
    pub async fn get_room_by_path(&self, actor: &RoomActor, path: &str) -> Result<Room> {
        let roomid = {
            let tree = self.path_tree.read().await;
            tree.find_roomid_by_path(path)
                .await
                .ok_or(AppError::NotFound)?
        };

        let room = self
            .cache_manager
            .get_room(roomid)
            .await?
            .ok_or(AppError::NotFound)?;

        // 绑定前路径树只暴露最小 roomid；完整电费/阈值详情仍必须走房间访问控制。
        self.ensure_room_access(actor, room.roomid).await?;
        Ok(room)
    }

    #[instrument(skip(self, actor), fields(module = "room", use_case = "get_room_by_hash", path_hash = hash))]
    pub async fn get_room_by_hash(&self, actor: &RoomActor, hash: i64, path: &str) -> Result<Room> {
        let roomid = {
            let tree = self.path_tree.read().await;
            tree.find_roomid_by_hash(hash, path)
                .await
                .ok_or(AppError::NotFound)?
        };

        let room = self
            .cache_manager
            .get_room(roomid)
            .await?
            .ok_or(AppError::NotFound)?;

        self.ensure_room_access(actor, room.roomid).await?;
        Ok(room)
    }

    async fn ensure_room_access(&self, actor: &RoomActor, roomid: i64) -> Result<()> {
        if actor.is_admin {
            return Ok(());
        }

        let user_id = actor
            .user_id
            .ok_or(AppError::Unauthorized("未认证".to_string()))?;

        let binding = self
            .binding_queries
            .find_by_user_and_room(user_id, roomid)
            .await?;

        if binding.is_none() {
            return Err(AppError::Forbidden);
        }

        Ok(())
    }

    async fn bound_roomids(&self, actor: &RoomActor) -> Result<HashSet<i64>> {
        let user_id = actor
            .user_id
            .ok_or(AppError::Unauthorized("未认证".to_string()))?;
        Ok(self
            .binding_queries
            .find_by_user_id(user_id)
            .await?
            .into_iter()
            .map(|binding| binding.roomid)
            .collect())
    }
}
