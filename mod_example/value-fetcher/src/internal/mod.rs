//! 内部实现模块
//!
//! 包含所有内部组件，通过 Facade 模式对外提供服务。
//! 所有类型仅在 crate 内部可见。

pub(crate) mod executor;
pub(crate) mod http;
pub(crate) mod parser;
pub(crate) mod traits;
pub(crate) mod url;

// 内部导出（仅 crate 内可用）
pub(crate) use executor::{RoomBatchFetcher, RoomResult};
pub(crate) use http::ReqwestAsyncClient;
pub(crate) use parser::ElectricityParser;
pub(crate) use url::UrlBuilder;
