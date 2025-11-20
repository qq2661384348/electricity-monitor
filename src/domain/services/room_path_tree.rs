//! 房间路径树
//!
//! 提供高性能的内存路径索引，支持逐层查询和路径哈希反查
//! 
//! ## 设计理念
//! - **Trie 树结构**: 4 层固定深度（校区/建筑/楼层/房间）
//! - **线程安全**: 使用 `Arc<RwLock>` 支持并发读写
//! - **内存优化**: 约 6000 房间占用 < 5MB
//! - **查询性能**: O(depth)，depth=4
//! 
//! ## 使用示例
//! ```rust,ignore
//! use crate::domain::services::room_path_tree::RoomPathTree;
//! use crate::domain::services::room_sync::crawler::models::RoomData;
//! 
//! // 构建路径树
//! let rooms = vec![...];
//! let tree = RoomPathTree::build_from_rooms(&rooms);
//! 
//! // 查询子节点
//! let children = tree.query_children("箭盘校区").await?;
//! 
//! // 路径哈希反查
//! let roomid = tree.find_roomid_by_path("箭盘校区/北区12栋/三楼/B12313");
//! ```

use anyhow::Result;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::services::room_sync::crawler::models::RoomData;
use crate::utils::hash::calculate_roompath_hash;

/// 路径分隔符常量
const PATH_SEPARATOR: char = '/';

/// 路径树节点
/// 
/// 使用 Trie 结构存储层级关系：
/// - 非叶子节点：校区/建筑/楼层（只有 name 和 children）
/// - 叶子节点：房间（额外存储 roomids）
/// 
/// **设计说明**：构建阶段使用 `Box<PathNode>` 避免借用冲突，
/// 完成后转换为 `Arc<PathNode>` 支持多线程共享。
#[derive(Debug, Clone)]
pub struct PathNode {
    /// 节点名称（如 "箭盘校区"、"北区12栋"）
    pub name: String,
    
    /// 子节点映射（name -> Node）
    pub children: HashMap<String, Arc<PathNode>>,
    
    /// 是否为叶子节点（Level 4 房间）
    pub is_leaf: bool,
    
    /// 叶子节点存储的 roomid 列表（支持 1:N 映射）
    pub roomids: Vec<i32>,
}

impl PathNode {
    /// 创建新节点
    pub fn new(name: String, is_leaf: bool) -> Self {
        Self {
            name,
            children: HashMap::new(),
            is_leaf,
            roomids: Vec::new(),
        }
    }
    
    /// 递归统计节点下的房间总数
    pub fn count_rooms(&self) -> usize {
        if self.is_leaf {
            self.roomids.len()
        } else {
            self.children
                .values()
                .map(|child| child.count_rooms())
                .sum()
        }
    }
}

/// 构建阶段的可变节点（避免 Arc 借用问题）
struct MutablePathNode {
    name: String,
    children: HashMap<String, Box<MutablePathNode>>,
    is_leaf: bool,
    roomids: Vec<i32>,
}

impl MutablePathNode {
    fn new(name: String, is_leaf: bool) -> Self {
        Self {
            name,
            children: HashMap::new(),
            is_leaf,
            roomids: Vec::new(),
        }
    }
    
    /// 获取或创建子节点（可变引用）
    fn get_or_create_child(&mut self, name: &str, is_leaf: bool) -> &mut MutablePathNode {
        self.children
            .entry(name.to_string())
            .or_insert_with(|| Box::new(MutablePathNode::new(name.to_string(), is_leaf)))
    }
    
    /// 转换为不可变的 PathNode（递归）
    fn into_path_node(self) -> PathNode {
        PathNode {
            name: self.name,
            children: self
                .children
                .into_iter()
                .map(|(k, v)| (k, Arc::new(v.into_path_node())))
                .collect(),
            is_leaf: self.is_leaf,
            roomids: self.roomids,
        }
    }
}

/// 房间路径树管理器
/// 
/// 线程安全的路径索引，支持：
/// 1. 从 RoomData 列表构建 Trie 树
/// 2. 逐层查询子节点
/// 3. 路径哈希反查 roomid
/// 4. 统计信息查询
#[derive(Debug, Clone)]
pub struct RoomPathTree {
    /// 树根节点（虚拟节点，name=""）
    root: Arc<RwLock<PathNode>>,
    
    /// 最后更新时间
    last_updated: Arc<RwLock<NaiveDateTime>>,
    
    /// 哈希索引: key=path_hash, value=路径-房间ID映射列表
    hash_index: Arc<RwLock<HashIndex>>,
}

type HashIndex = HashMap<i64, Vec<PathHashEntry>>;

#[derive(Debug, Clone, PartialEq)]
struct PathHashEntry {
    /// 房间路径
    roompath: String,
    /// 房间ID
    roomid: i32,
}

impl RoomPathTree {
    /// 创建空树
    pub fn new() -> Self {
        Self {
            root: Arc::new(RwLock::new(PathNode::new(String::new(), false))),
            last_updated: Arc::new(RwLock::new(chrono::Utc::now().naive_utc())),
            hash_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 从爬虫数据构建路径树
    /// 
    /// # 参数
    /// - `rooms`: 从 `RoomSyncService` 获取的房间数据列表
    /// 
    /// # 算法复杂度
    /// - 时间: O(n * d)，n=房间数，d=路径深度（4）
    /// - 空间: O(n)
    /// 
    /// # 示例
    /// ```rust,ignore
    /// let rooms = vec![
    ///     RoomData { roomid: 101, primary_roompath: "箭盘校区/北区12栋/三楼/B12313".into(), .. },
    ///     RoomData { roomid: 102, primary_roompath: "箭盘校区/北区12栋/三楼/B12314".into(), .. },
    /// ];
    /// 
    /// let tree = RoomPathTree::build_from_rooms(&rooms);
    /// ```
    pub fn build_from_rooms(rooms: &[RoomData]) -> Self {
        let mut mutable_root = MutablePathNode::new(String::new(), false);
        let mut hash_map = HashIndex::new();
        
        tracing::info!("开始构建路径树：共 {} 个房间", rooms.len());
        
        for room in rooms {
            let path = &room.primary_roompath;
            let parts: Vec<&str> = path.split(PATH_SEPARATOR).collect();
            
            if parts.is_empty() {
                tracing::warn!("跳过空路径：roomid={}", room.roomid);
                continue;
            }
            
            // 逐层构建节点（使用 MutablePathNode）
            let mut current = &mut mutable_root;
            
            for (level, part) in parts.iter().enumerate() {
                let is_leaf = level == parts.len() - 1;
                current = current.get_or_create_child(part, is_leaf);
            }
            
            // 叶子节点存储 roomid
            if current.is_leaf {
                current.roomids.push(room.roomid);
            }
            
            // 构建哈希索引
            let hash = calculate_roompath_hash(path);
            hash_map
                .entry(hash)
                .or_insert_with(Vec::new)
                .push(PathHashEntry {
                    roompath: room.primary_roompath.clone(),
                    roomid: room.roomid,
                });
        }
        
        // 转换为不可变树
        let root = mutable_root.into_path_node();
        
        // 统计信息
        let total_nodes = Self::count_nodes(&root);
        let total_rooms = root.count_rooms();
        
        tracing::info!(
            "路径树构建完成：总节点数={}，总房间数={}，哈希索引={}",
            total_nodes,
            total_rooms,
            hash_map.len()
        );
        
        // 直接返回构造好的实例
        Self {
            root: Arc::new(RwLock::new(root)),
            last_updated: Arc::new(RwLock::new(chrono::Utc::now().naive_utc())),
            hash_index: Arc::new(RwLock::new(hash_map)),
        }
    }
    
    /// 逐层查询子节点
    /// 
    /// # 参数
    /// - `parent_path`: 父路径（空字符串表示根节点）
    /// 
    /// # 返回
    /// 子节点列表（排序后）
    /// 
    /// # 示例
    /// ```rust,ignore
    /// let campuses = tree.query_children("").await?;  // ["箭盘校区", "东环校区"]
    /// let buildings = tree.query_children("箭盘校区").await?;  // ["北区12栋", "南区5栋"]
    /// ```
    pub async fn query_children(&self, parent_path: &str) -> Result<Vec<PathChildNode>> {
        let root = self.root.read().await;
        
        // 根路径查询
        if parent_path.is_empty() {
            let mut children: Vec<_> = root
                .children
                .values()
                .map(|node| PathChildNode {
                    name: node.name.clone(),
                    is_leaf: node.is_leaf,
                    room_count: node.count_rooms(),
                })
                .collect();
            
            children.sort_by(|a, b| a.name.cmp(&b.name));
            return Ok(children);
        }
        
        // 逐层查找目标节点
        let parts: Vec<&str> = parent_path.split(PATH_SEPARATOR).collect();
        let mut current = &*root;
        
        for part in &parts {
            match current.children.get(*part) {
                Some(node) => current = node,
                None => {
                    anyhow::bail!("路径不存在: {}", parent_path);
                }
            }
        }
        
        // 返回子节点
        let mut children: Vec<_> = current
            .children
            .values()
            .map(|node| PathChildNode {
                name: node.name.clone(),
                is_leaf: node.is_leaf,
                room_count: node.count_rooms(),
            })
            .collect();
        
        children.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(children)
    }
    
    /// 根据路径查找 roomid
    /// 
    /// # 参数
    /// - `path`: 完整路径（如 "箭盘校区/北区12栋/三楼/B12313"）
    /// 
    /// # 返回
    /// - `Some(roomid)`: 找到房间
    /// - `None`: 路径不存在或非叶子节点
    pub async fn find_roomid_by_path(&self, path: &str) -> Option<i32> {
        let root = self.root.read().await;
        let parts: Vec<&str> = path.split(PATH_SEPARATOR).collect();
        
        let mut current = &*root;
        
        for part in &parts {
            match current.children.get(*part) {
                Some(node) => current = node,
                None => return None,
            }
        }
        
        // 必须是叶子节点且有 roomid
        if current.is_leaf && !current.roomids.is_empty() {
            Some(current.roomids[0])
        } else {
            None
        }
    }
    
    /// 根据路径哈希查找 roomid（支持哈希冲突验证）
    /// 
    /// # 参数
    /// - `hash`: 路径哈希值
    /// - `path`: 完整路径（用于验证，防止哈希冲突）
    /// 
    /// # 返回
    /// - `Some(roomid)`: 找到房间
    /// - `None`: 哈希不存在或路径不匹配
    pub async fn find_roomid_by_hash(&self, hash: i64, path: &str) -> Option<i32> {
        let index = self.hash_index.read().await;
        
        match index.get(&hash) {
            Some(entries) => {
                // 精确匹配路径（防止哈希冲突）
                entries
                    .iter()
                    .find(|entry| entry.roompath == path)
                    .map(|entry| entry.roomid)
            }
            None => None,
        }
    }
    
    /// 获取最后更新时间
    pub async fn last_updated(&self) -> NaiveDateTime {
        *self.last_updated.read().await
    }
    
    /// 递归统计节点总数（用于调试）
    fn count_nodes(node: &PathNode) -> usize {
        1 + node
            .children
            .values()
            .map(|child| Self::count_nodes(child))
            .sum::<usize>()
    }
}

impl Default for RoomPathTree {
    fn default() -> Self {
        Self::new()
    }
}

/// 路径子节点（查询结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathChildNode {
    /// 节点名称
    pub name: String,
    
    /// 是否为叶子节点（房间）
    pub is_leaf: bool,
    
    /// 该节点下的房间总数
    pub room_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造测试数据
    fn mock_rooms() -> Vec<RoomData> {
        vec![
            RoomData {
                roomid: 101,
                roompaths: vec!["箭盘校区/北区12栋/三楼/B12313".to_string()],
                primary_roompath: "箭盘校区/北区12栋/三楼/B12313".to_string(),
                path_count: 1,
            },
            RoomData {
                roomid: 102,
                roompaths: vec!["箭盘校区/北区12栋/三楼/B12314".to_string()],
                primary_roompath: "箭盘校区/北区12栋/三楼/B12314".to_string(),
                path_count: 1,
            },
            RoomData {
                roomid: 103,
                roompaths: vec!["东环校区/南区5栋/二楼/A201".to_string()],
                primary_roompath: "东环校区/南区5栋/二楼/A201".to_string(),
                path_count: 1,
            },
        ]
    }

    #[test]
    fn test_path_node_creation() {
        let node = PathNode::new("测试节点".to_string(), false);
        assert_eq!(node.name, "测试节点");
        assert!(!node.is_leaf);
        assert_eq!(node.children.len(), 0);
        assert_eq!(node.roomids.len(), 0);
    }

    #[test]
    fn test_path_node_room_count() {
        let mut root = PathNode::new("root".to_string(), false);
        let mut child = PathNode::new("child".to_string(), true);
        child.roomids = vec![101, 102];
        root.children.insert("child".to_string(), Arc::new(child));
        
        assert_eq!(root.count_rooms(), 2);
    }

    #[tokio::test]
    async fn test_build_from_rooms() {
        let rooms = mock_rooms();
        let tree = RoomPathTree::build_from_rooms(&rooms);
        
        let root = tree.root.read().await;
        assert_eq!(root.children.len(), 2);  // 2个校区
        assert!(root.children.contains_key("箭盘校区"));
        assert!(root.children.contains_key("东环校区"));
    }

    #[tokio::test]
    async fn test_query_children_root() {
        let rooms = mock_rooms();
        let tree = RoomPathTree::build_from_rooms(&rooms);
        
        let children = tree.query_children("").await.unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "东环校区");
        assert_eq!(children[1].name, "箭盘校区");
    }

    #[tokio::test]
    async fn test_query_children_nested() {
        let rooms = mock_rooms();
        let tree = RoomPathTree::build_from_rooms(&rooms);
        
        let buildings = tree.query_children("箭盘校区").await.unwrap();
        assert_eq!(buildings.len(), 1);
        assert_eq!(buildings[0].name, "北区12栋");
        assert_eq!(buildings[0].room_count, 2);
    }

    #[tokio::test]
    async fn test_find_roomid_by_path() {
        let rooms = mock_rooms();
        let tree = RoomPathTree::build_from_rooms(&rooms);
        
        let roomid = tree.find_roomid_by_path("箭盘校区/北区12栋/三楼/B12313").await;
        assert_eq!(roomid, Some(101));
        
        let not_found = tree.find_roomid_by_path("不存在/路径").await;
        assert_eq!(not_found, None);
    }

    #[tokio::test]
    async fn test_find_roomid_by_hash() {
        let rooms = mock_rooms();
        let tree = RoomPathTree::build_from_rooms(&rooms);
        
        let path = "箭盘校区/北区12栋/三楼/B12313";
        let hash = calculate_roompath_hash(path);
        
        let roomid = tree.find_roomid_by_hash(hash, path).await;
        assert_eq!(roomid, Some(101));
        
        // 哈希匹配但路径不匹配
        let wrong_path = tree.find_roomid_by_hash(hash, "错误路径").await;
        assert_eq!(wrong_path, None);
    }

    #[tokio::test]
    async fn test_query_children_invalid_path() {
        let rooms = mock_rooms();
        let tree = RoomPathTree::build_from_rooms(&rooms);
        
        let result = tree.query_children("不存在的校区").await;
        assert!(result.is_err());
    }
}
