//! 通知模块
//!
//! 提供QQ机器人通知功能，包括：
//! - QQ客户端：与QQ机器人API通信
//! - 消息构建器：构建各种类型的消息
//! - 错误处理：统一的错误类型

pub mod error;
pub mod message_builder;
pub mod qq_client;

pub use error::{NotificationError, Result};
pub use message_builder::MessageBuilder;
pub use qq_client::{QQClient, QQMessageData, QQMessageResponse};
