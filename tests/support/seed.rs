use std::sync::atomic::{AtomicU64, Ordering};

use chrono::Utc;
use electricity_monitor_backend::{
    domain::models::{NewRoom, Room},
    infrastructure::repositories::RoomRepository,
    state::AppState,
    utils::hash::calculate_roompath_hash,
};

static ROOM_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_roomid() -> i32 {
    let millis = Utc::now().timestamp_millis().rem_euclid(100_000);
    let counter = ROOM_COUNTER
        .fetch_add(1, Ordering::Relaxed)
        .rem_euclid(9_000);
    800_000 + millis as i32 * 10 + counter as i32
}

pub async fn seed_room(state: &AppState) -> Room {
    let roomid = unique_roomid();
    let repo = RoomRepository::new(state.db_pool.clone());

    repo.create(NewRoom {
        roomid,
        electricity_fee: 25.0,
        threshold: 100.0,
        room_name: format!("测试房间-{roomid}"),
        primary_roompath: format!("测试/路径/{roomid}"),
        primary_roompath_hash: calculate_roompath_hash(&format!("测试/路径/{roomid}")),
        has_additional_paths: false,
        is_active: true,
        source_type: "test".to_string(),
        external_id: None,
        last_synced_at: None,
        last_recovered_at: None,
    })
    .await
    .expect("创建测试房间失败")
}

pub async fn delete_room(state: &AppState, room_id: uuid::Uuid) {
    let repo = RoomRepository::new(state.db_pool.clone());
    let _ = repo.delete(room_id).await;
}
