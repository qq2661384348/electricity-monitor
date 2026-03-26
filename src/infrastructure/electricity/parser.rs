//! 电费数据解析器
//!
//! 使用高性能字符串搜索从 HTTP 响应中提取电费数值

use super::error::Result;

/// 电费解析器
///
/// 使用 `str::find()` 高效提取 JSON 响应中的 "Value" 字段
pub struct ElectricityParser;

impl ElectricityParser {
    /// 创建解析器
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// 解析电费值
    ///
    /// # 算法
    ///
    /// 1. 快速检测 BS=-1（房间不存在）
    /// 2. 使用 find() 定位 "Value" 字段
    /// 3. 手动提取数值部分
    /// 4. 简单验证数字格式
    pub fn parse(&self, raw_data: &str) -> Option<f32> {
        // 1. 快速检测 API 业务错误：{"BS":"-1","Msg":"失败",...}
        if raw_data.contains(r#"\"BS\":\"-1\""#) {
            return None;
        }

        // 2. 查找 "Name":"剩余" 后面的 "Value" 字段
        let name_marker = r#"\"Name\":\"剩余\""#;
        let name_pos = raw_data.find(name_marker)?;

        // 从"剩余"后面开始查找 "Value"
        let rest_after_name = &raw_data[name_pos..];
        let value_marker = r#"\"Value\":\""#;
        let value_pos_relative = rest_after_name.find(value_marker)?;
        let value_start = name_pos + value_pos_relative + value_marker.len();

        // 3. 从 Value 后面提取到下一个转义引号 \"
        let rest = &raw_data[value_start..];
        let value_end = rest.find(r#"\""#)?;
        let value_str = &rest[..value_end];

        // 4. 解析为 f32
        value_str.parse::<f32>().ok()
    }
}

impl Default for ElectricityParser {
    fn default() -> Self {
        // new()总是返回Ok(Self)，永远不会失败
        Self::new().expect("ElectricityParser::new() 不应该失败")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = ElectricityParser::new();
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parse_valid_data() {
        let parser = ElectricityParser::new().unwrap();
        let raw_data = r#""{\"BS\":\"1\",\"Msg\":\"成功\",\"total\":0,\"component\":[{\"Name\":\"状态\",\"Value\":\"正常\"},{\"Name\":\"剩余\",\"Value\":\"45.67\"}],\"url\":null}""#;
        let result = parser.parse(raw_data);
        assert_eq!(result, Some(45.67));
    }

    #[test]
    fn test_parse_negative_value() {
        let parser = ElectricityParser::new().unwrap();
        let raw_data = r#""{\"BS\":\"1\",\"Msg\":\"成功\",\"total\":0,\"component\":[{\"Name\":\"状态\",\"Value\":\"正常\"},{\"Name\":\"剩余\",\"Value\":\"-224.37\"}],\"url\":null}""#;
        let result = parser.parse(raw_data);
        assert_eq!(result, Some(-224.37));
    }

    #[test]
    fn test_parse_room_not_found() {
        let parser = ElectricityParser::new().unwrap();
        let raw_data =
            r#""{\"BS\":\"-1\",\"Msg\":\"失败\",\"total\":0,\"component\":null,\"url\":null}""#;
        let result = parser.parse(raw_data);
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_parse_real_api_response() {
        if std::env::var("RUN_EXTERNAL_INTEGRATION_TESTS").is_err() {
            println!("跳过真实电费 API 测试：设置 RUN_EXTERNAL_INTEGRATION_TESTS=1 以启用");
            return;
        }

        let api_url = "https://zywxhd02.gxust.edu.cn/Home/GetRoomInfo?roomid=4330";

        // 发起HTTP请求
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        let response = match client.get(api_url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                panic!("外部网络测试模式下请求真实电费 API 失败: {}", e);
            }
        };

        let raw_data = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                panic!("外部网络测试模式下读取真实电费 API 响应失败: {}", e);
            }
        };

        println!("真实API响应: {}", raw_data);

        // 测试解析器能否成功解析
        let parser = ElectricityParser::new().unwrap();
        let result = parser.parse(&raw_data);

        // 验证：应该能解析出一个数值（不验证具体值）
        assert!(
            result.is_some(),
            "解析器应该能从真实API响应中提取电费值，实际响应: {}",
            raw_data
        );

        let value = result.unwrap();
        println!("成功解析电费值: {}", value);
    }
}
