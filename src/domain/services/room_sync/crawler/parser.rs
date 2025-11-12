//! JSON解析器
//!
//! 使用sonic-rs进行高性能JSON解析，处理特殊情况（BOM、双重编码）

use anyhow::{Context, Result};
use sonic_rs::{JsonContainerTrait, JsonValueTrait, Value};

/// 安全解析JSON字符串
/// 
/// 处理以下特殊情况：
/// 1. UTF-8 BOM（字节序标记）
/// 2. 双重JSON编码（字符串中的JSON）
/// 3. 前后空白字符
/// 
/// # 参数
/// - `json_str`: 待解析的JSON字符串
/// 
/// # 返回
/// sonic_rs::Value对象
/// 
/// # 示例
/// ```
/// use electricity_monitor_backend::domain::services::room_sync::crawler::parser::safe_parse;
/// 
/// // 正常JSON
/// let result = safe_parse(r#"{"key": "value"}"#);
/// assert!(result.is_ok());
/// 
/// // 带BOM的JSON
/// let result = safe_parse("\u{FEFF}{\"key\": \"value\"}");
/// assert!(result.is_ok());
/// 
/// // 双重编码的JSON
/// let result = safe_parse(r#""{\"key\": \"value\"}""#);
/// assert!(result.is_ok());
/// ```
pub fn safe_parse(json_str: &str) -> Result<Value> {
    // 移除BOM和前后空白
    let cleaned = json_str.trim_start_matches('\u{FEFF}').trim();
    
    // 尝试直接解析
    match sonic_rs::from_str(cleaned) {
        Ok(value) => Ok(value),
        Err(e) => {
            // 可能是双重编码，尝试解析字符串
            if let Ok(val) = sonic_rs::from_str::<Value>(cleaned) {
                if let Some(inner) = val.as_str() {
                    // 递归解析内部JSON
                    return sonic_rs::from_str(inner)
                        .with_context(|| format!("双重编码JSON解析失败: {}", e));
                }
            }
            Err(e.into())
        }
    }
}

/// 解析楼层节点
/// 
/// 从楼层节点中提取房间路径
/// 
/// # 参数
/// - `floor`: 楼层节点对象
/// - `base_path`: 基础路径（校区/楼栋）
/// 
/// # 返回
/// 房间路径列表
pub fn parse_floor_rooms(floor: &Value, base_path: &str) -> Result<Vec<(String, String)>> {
    let floor_name = floor["Name"]
        .as_str()
        .context("楼层Name字段不存在或非字符串")?;
    
    let rooms = floor["Items"]
        .as_array()
        .context("楼层Items字段不存在或非数组")?;
    
    let mut result = Vec::new();
    
    for room in rooms {
        let room_name = room["Name"]
            .as_str()
            .context("房间Name字段不存在")?;
        
        let roomid = room["Value"]
            .as_str()
            .context("房间Value字段不存在")?;
        
        // 构造完整路径
        let roompath = format!("{}/{}/{}", base_path, floor_name, room_name);
        
        result.push((roompath, roomid.to_string()));
    }
    
    Ok(result)
}

/// 解析楼栋节点
/// 
/// # 参数
/// - `building`: 楼栋节点对象
/// - `base_path`: 基础路径（校区）
/// 
/// # 返回
/// 房间路径列表
pub fn parse_building_rooms(building: &Value, base_path: &str) -> Result<Vec<(String, String)>> {
    let building_name = building["Name"]
        .as_str()
        .context("楼栋Name字段不存在或非字符串")?;
    
    let floors = building["Items"]
        .as_array()
        .context("楼栋Items字段不存在或非数组")?;
    
    let mut result = Vec::new();
    let full_base_path = format!("{}/{}", base_path, building_name);
    
    for floor in floors {
        let rooms = parse_floor_rooms(floor, &full_base_path)?;
        result.extend(rooms);
    }
    
    Ok(result)
}

/// 解析校区节点
/// 
/// # 参数
/// - `campus`: 校区节点对象
/// 
/// # 返回
/// 房间路径列表
pub fn parse_campus_rooms(campus: &Value) -> Result<Vec<(String, String)>> {
    let campus_name = campus["Name"]
        .as_str()
        .context("校区Name字段不存在或非字符串")?;
    
    let buildings = campus["Items"]
        .as_array()
        .context("校区Items字段不存在或非数组")?;
    
    let mut result = Vec::new();
    
    for building in buildings {
        let rooms = parse_building_rooms(building, campus_name)?;
        result.extend(rooms);
    }
    
    Ok(result)
}

/// 解析完整的房间树JSON
/// 
/// # 参数
/// - `json_str`: JSON字符串
/// 
/// # 返回
/// (roompath, roomid)元组列表
pub fn parse_room_tree(json_str: &str) -> Result<Vec<(String, String)>> {
    let root = safe_parse(json_str)
        .context("JSON解析失败")?;
    
    let campuses = root
        .as_array()
        .context("根节点应该是数组")?;
    
    let mut result = Vec::new();
    
    for campus in campuses {
        let rooms = parse_campus_rooms(campus)
            .with_context(|| format!("解析校区失败: {:?}", campus))?;
        result.extend(rooms);
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_parse_normal() {
        let json = r#"{"key": "value"}"#;
        let result = safe_parse(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_parse_with_bom() {
        let json = "\u{FEFF}{\"key\": \"value\"}";
        let result = safe_parse(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_parse_double_encoded() {
        let json = r#""{\"key\": \"value\"}""#;
        let result = safe_parse(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_parse_with_whitespace() {
        let json = "  \n\t{\"key\": \"value\"}  \n";
        let result = safe_parse(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_room_tree() {
        let json = r#"[
            {
                "Name": "桂林",
                "Items": [
                    {
                        "Name": "05栋",
                        "Items": [
                            {
                                "Name": "5楼",
                                "Items": [
                                    {"Name": "0501", "Value": "101"},
                                    {"Name": "0502", "Value": "102"}
                                ]
                            }
                        ]
                    }
                ]
            }
        ]"#;
        
        let result = parse_room_tree(json);
        assert!(result.is_ok());
        
        let rooms = result.unwrap();
        assert_eq!(rooms.len(), 2);
        assert_eq!(rooms[0].0, "桂林/05栋/5楼/0501");
        assert_eq!(rooms[0].1, "101");
    }
}
