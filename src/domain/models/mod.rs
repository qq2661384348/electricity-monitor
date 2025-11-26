//! 领域模型
//!
//! 定义核心业务实体

pub mod electricity_history;
pub mod room;
pub mod room_aggregate;
pub mod room_path;
pub mod room_sync_log;
pub mod user;
pub mod user_room_binding;

pub use electricity_history::{ElectricityHistory, NewElectricityHistory};
pub use room::{NewRoom, ResetSendFlag, Room, UpdateElectricityFee, UpdateLastRecovered, UpdateThreshold};
pub use room_aggregate::RoomAggregate;
pub use room_path::{NewRoomPath, RoomPath};
pub use room_sync_log::*;
pub use user::{NewUser, UpdateUserRole, User};
pub use user_room_binding::{
    NewUserRoomBinding, UpdateLastNotified, UpdateNotificationEnabled, UserRoomBinding, UserRoomBindingWithRoomInfo,
};
