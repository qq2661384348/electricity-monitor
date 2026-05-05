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
    /// * `public_url` - 公开访问地址，由运行时配置生成
    ///
    /// # 返回
    /// 格式化的预警消息文本
    ///
    /// # 消息格式
    /// - 使用 emoji 增强视觉层次
    /// - 显示房间路径（primary_roompath）而非房间名称
    /// - 不显示内部 roomid，提升用户体验
    pub fn build_electricity_alert_message(room: &Room, public_url: &str) -> String {
        format!(
            "⚡ 【电量预警提醒】\n\n📍 房间位置: {}\n🔋 当前剩余: {:.2} kWh\n⚠️  预警阈值: {:.2} kWh\n\n💡 您的电量已低于预警阈值，请及时充值！\n\n访问{} 以更新你的数据",
            room.primary_roompath,
            room.electricity_fee,
            room.threshold,
            public_url.trim()
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
            primary_roompath: "南校区/1号楼/101".to_string(),
            primary_roompath_hash: 12345678,
            has_additional_paths: false,
            is_active: true,
            source_type: "manual".to_string(),
            external_id: None,
            last_synced_at: None,
            last_recovered_at: None,
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
        };

        let public_url = "https://pythonrust.icu:11451/";
        let message = MessageBuilder::build_electricity_alert_message(&room, public_url);

        // 检查 emoji 存在
        assert!(message.contains("⚡"));
        assert!(message.contains("📍"));
        assert!(message.contains("🔋"));
        assert!(message.contains("⚠️"));
        assert!(message.contains("💡"));

        // 检查房间路径（而非房间名称）
        assert!(message.contains("南校区/1号楼/101"));
        assert!(!message.contains("测试房间")); // 不应该包含 room_name

        // 检查电量数据
        assert!(message.contains("5.50"));
        assert!(message.contains("10.00"));
        assert!(message.contains(public_url));

        // 确认不显示 "roomid" 或 "房间ID" 字样
        assert!(!message.contains("roomid"));
        assert!(!message.contains("房间ID"));
    }

    #[test]
    fn test_build_api_request_body() {
        let body = MessageBuilder::build_api_request_body("123456", "测试消息");
        assert_eq!(body["user_id"], "123456");
        assert_eq!(body["message"], "测试消息");
    }
}
