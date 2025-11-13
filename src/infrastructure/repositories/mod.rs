//! 数据访问层 (Repositories)
//!
//! 提供数据持久化操作的抽象

pub mod electricity_history_repository;
pub mod room_repository;
pub mod user_repository;
pub mod user_room_binding_repository;

pub use electricity_history_repository::ElectricityHistoryRepository;
pub use room_repository::RoomRepository;
pub use user_repository::UserRepository;
pub use user_room_binding_repository::UserRoomBindingRepository;
