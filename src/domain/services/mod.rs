//! 领域服务
//!
//! 封装跨实体的业务逻辑

pub mod electricity_service;
pub mod notification_service;
pub mod rate_limiter;

pub use electricity_service::{ElectricityData, ElectricityService};
pub use notification_service::NotificationService;
pub use rate_limiter::{RateLimiter, RateLimitOperation};
