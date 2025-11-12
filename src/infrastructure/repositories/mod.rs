//! 数据仓储实现
//!
//! 实现数据访问抽象层

pub mod electricity_history_repository;
pub mod room_repository;

pub use electricity_history_repository::ElectricityHistoryRepository;
pub use room_repository::RoomRepository;
