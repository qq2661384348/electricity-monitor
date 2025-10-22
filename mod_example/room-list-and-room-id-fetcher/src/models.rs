//! 数据模型定义
//!
//! 定义与 API 交互的数据结构和最终输出格式

use serde::{Deserialize, Serialize};

/// API 响应根结构
///
/// API 返回的 JSON 格式：
/// ```json
/// {
///     "component": [
///         { "RoomDepId": "xxx", "DepName": "xxx" },
///         ...
///     ]
/// }
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct ApiResponse {
    /// 组件列表（可能为空）
    pub component: Option<Vec<RoomComponent>>,
}

/// 房间组件（通用结构，适配4层级）
///
/// 用于表示：
/// - Level 1: 校区（Campus）
/// - Level 2: 建筑（Building）
/// - Level 3: 楼层（Floor）
/// - Level 4: 房间（Room）
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RoomComponent {
    /// 房间部门ID（层级标识符）
    pub room_dep_id: String,

    /// 部门名称（显示名称）
    pub dep_name: String,
}

/// 最终输出房间信息
///
/// 输出格式：
/// ```json
/// {
///     "roompath": "东环校区/东环北区12栋/三楼/B12313",
///     "roomid": "3241"
/// }
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct RoomInfo {
    /// 房间完整路径（校区/建筑/楼层/房间）
    pub roompath: String,

    /// 房间唯一标识符
    pub roomid: String,
}
