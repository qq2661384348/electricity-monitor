use uuid::Uuid;

use crate::{
    domain::models::{NewRoom, Room, UpdateThreshold, UserRoomBinding},
    errors::Result,
    infrastructure::repositories::{RoomRepository, UserRoomBindingRepository},
};

#[derive(Clone)]
pub struct RoomQueries {
    repository: RoomRepository,
}

impl RoomQueries {
    pub fn new(repository: RoomRepository) -> Self {
        Self { repository }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Room>> {
        self.repository.find_by_id(id).await
    }

    pub async fn find_by_roomid(&self, roomid: i32) -> Result<Option<Room>> {
        self.repository.find_by_roomid(roomid).await
    }

    pub async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Room>> {
        self.repository.find_all(limit, offset).await
    }

    pub async fn find_by_roomids_paged(
        &self,
        roomids: &[i32],
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Room>> {
        self.repository
            .find_by_roomids_paged(roomids, limit, offset)
            .await
    }

    pub async fn find_flagged(&self) -> Result<Vec<Room>> {
        self.repository.find_rooms_with_send_flag_true().await
    }
}

#[derive(Clone)]
pub struct RoomCommands {
    repository: RoomRepository,
}

impl RoomCommands {
    pub fn new(repository: RoomRepository) -> Self {
        Self { repository }
    }

    pub async fn create(&self, new_room: NewRoom) -> Result<Room> {
        self.repository.create(new_room).await
    }

    pub async fn update_threshold(&self, id: Uuid, threshold: f32) -> Result<Room> {
        self.repository
            .update_threshold(id, UpdateThreshold { threshold })
            .await
    }

    pub async fn reset_send_flag(&self, id: Uuid) -> Result<Room> {
        self.repository.reset_send_flag(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<usize> {
        self.repository.delete(id).await
    }
}

#[derive(Clone)]
pub struct BindingQueries {
    repository: UserRoomBindingRepository,
}

impl BindingQueries {
    pub fn new(repository: UserRoomBindingRepository) -> Self {
        Self { repository }
    }

    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<UserRoomBinding>> {
        self.repository.find_by_user_id(user_id).await
    }

    pub async fn find_by_user_and_room(
        &self,
        user_id: Uuid,
        roomid: i32,
    ) -> Result<Option<UserRoomBinding>> {
        self.repository.find_by_user_and_room(user_id, roomid).await
    }
}
