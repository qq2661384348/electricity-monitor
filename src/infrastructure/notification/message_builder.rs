//! 消息构建器

use crate::domain::models::Room;
use sonic_rs::json;

/// 消息构建器
pub struct MessageBuilder;

impl MessageBuilder {
    /// 构建验证码消息
    /// 
    /// # 参数
    /// * `code` - 6位验证码
    /// 
    /// # 返回
    /// 格式化的验证码消息文本
    pub fn build_verification_code_message(code: &str) -> String {
        format!(
            "【电力监控系统】\n\n您的验证码是: {}\n\n验证码有效期为5分钟，请及时使用。\n如非本人操作，请忽略此消息。",
            code
        )
    }
    
    /// 构建电费预警消息
    /// 
    /// # 参数
    /// * `room` - 房间信息
    /// 
    /// # 返回
    /// 格式化的预警消息文本
    pub fn build_electricity_alert_message(room: &Room) -> String {
        format!(
            "【电量预警提醒】\n\n房间: {}\n房间ID: {}\n当前剩余电量: {:.2} kWh\n预警阈值: {:.2} kWh\n\n您的电量已低于预警阈值，请及时充值！",
            room.room_name,
            room.roomid,
            room.electricity_fee,
            room.threshold
        )
    }
    
    /// 构建QQ API请求体
    /// 
    /// # 参数
    /// * `user_id` - QQ号
    /// * `message` - 消息文本
    /// 
    /// # 返回
    /// JSON格式的请求体
    pub fn build_api_request_body(user_id: &str, message: &str) -> sonic_rs::Value {
        json!({
            "user_id": user_id,
            "message": message
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_build_verification_code_message() {
        let message = MessageBuilder::build_verification_code_message("123456");
        assert!(message.contains("123456"));
        assert!(message.contains("验证码"));
        assert!(message.contains("5分钟"));
    }

    #[test]
    fn test_build_electricity_alert_message() {
        let room = Room {
            id: Uuid::new_v4(),
            roomid: 101,
            electricity_fee: 5.5,
            send_flag: true,
            threshold: 10.0,
            room_name: "测试房间".to_string(),
            primary_roompath: "测试/路径".to_string(),
            primary_roompath_hash: 12345678,
            has_additional_paths: false,
            is_active: true,
            source_type: "manual".to_string(),
            external_id: None,
            last_synced_at: None,
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
        };
        
        let message = MessageBuilder::build_electricity_alert_message(&room);
        assert!(message.contains("测试房间"));
        assert!(message.contains("101"));
        assert!(message.contains("5.50"));
        assert!(message.contains("10.00"));
    }

    #[test]
    fn test_build_api_request_body() {
        let body = MessageBuilder::build_api_request_body("123456", "测试消息");
        assert_eq!(body["user_id"], "123456");
        assert_eq!(body["message"], "测试消息");
    }
}
