//! 领域模型
//!
//! 定义核心业务实体

pub mod room;
pub mod room_aggregate;
pub mod room_path;
pub mod room_sync_log;

pub use room::{NewRoom, ResetSendFlag, Room, UpdateElectricityFee, UpdateThreshold};
pub use room_aggregate::RoomAggregate;
pub use room_path::{NewRoomPath, RoomPath};
pub use room_sync_log::*;
