//! 电费数据解析器
//!
//! 从外部 HTTP 响应中提取电费数值。

use super::error::Result;
use serde_json::Value;

/// 电费解析器
///
/// 兼容新 Upay `Data[0].ResNum` 响应和旧 `component[].Value` 响应。
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
    /// 1. 优先解析新 Upay JSON 响应中的 `Data[0].ResNum`
    /// 2. 快速检测旧接口 BS=-1（房间不存在）
    /// 3. 兼容旧接口 `"Name":"剩余"` 后面的 `"Value"` 字段
    pub fn parse(&self, raw_data: &str) -> Option<f32> {
        if let Some(value) = self.parse_upay_res_num(raw_data) {
            return Some(value);
        }

        // 1. 快速检测 API 业务错误：{"BS":"-1","Msg":"失败",...}
        if raw_data.contains(r#"\"BS\":\"-1\""#) || raw_data.contains(r#""BS":"-1""#) {
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

    fn parse_upay_res_num(&self, raw_data: &str) -> Option<f32> {
        let value = parse_json_value(raw_data)?;
        let data = value.get("Data")?.as_array()?;
        let first = data.first()?;
        let res_num = first.get("ResNum")?;

        if let Some(value) = res_num.as_str() {
            return value.parse::<f32>().ok();
        }

        res_num.as_f64().map(|value| value as f32)
    }
}

fn parse_json_value(raw_data: &str) -> Option<Value> {
    let value: Value = serde_json::from_str(raw_data.trim()).ok()?;
    if let Some(inner) = value.as_str() {
        return serde_json::from_str(inner).ok();
    }
    Some(value)
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
    fn test_parse_upay_res_num_response() {
        let parser = ElectricityParser::new().unwrap();
        let raw_data = r#"{
            "Data": [
                {
                    "SchoolName": "文昌校区",
                    "ApartName": "北区4栋公寓",
                    "RoomID": "982318536531644416",
                    "RoomName": "107",
                    "ResNum": "73.80",
                    "UsedNum": "0.00",
                    "Updatedt": "2026-06-09 12:00:24"
                }
            ]
        }"#;

        let result = parser.parse(raw_data);

        assert_eq!(result, Some(73.80));
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

        let api_url =
            "https://upayadmin.gyruibo.cn/UpayManage/Home/GetRoom?roomid=982318536498089984";

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
