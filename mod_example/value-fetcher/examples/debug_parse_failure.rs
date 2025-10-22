//! 调试工具：分析数据解析失败的原因
//!
//! 功能：
//! - 对指定房间ID发起HTTP请求
//! - 显示原始响应内容
//! - 分析解析失败原因
//! - 提供修复建议

use regex::Regex;
use reqwest;
use std::time::Duration;

/// HTTP 客户端（简化版，用于调试）
struct DebugClient {
    client: reqwest::Client,
    url_prefix: String,
}

impl DebugClient {
    fn new(url_prefix: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()?;

        Ok(Self {
            client,
            url_prefix: url_prefix.to_string(),
        })
    }

    async fn fetch_raw(&self, room_id: u32) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.url_prefix, room_id);
        let response = self.client.get(&url).send().await?;
        let text = response.text().await?;
        Ok(text)
    }
}

/// 数据解析器（与生产环境相同）
struct DebugParser {
    regex: Regex,
}

impl DebugParser {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 不再使用正则
        Ok(Self {
            regex: Regex::new("dummy")?,
        })
    }

    fn parse(&self, raw_data: &str) -> Option<String> {
        // 与生产环境完全相同的逻辑
        if raw_data.contains(r#"\"BS\":\"-1\""#) {
            return Some("ROOM_NOT_FOUND".to_string());
        }

        // 查找 "Name":"剩余" 后面的 "Value"
        let name_marker = r#"\"Name\":\"剩余\""#;
        let name_pos = raw_data.find(name_marker)?;

        let rest_after_name = &raw_data[name_pos..];
        let value_marker = r#"\"Value\":\""#;
        let value_pos_relative = rest_after_name.find(value_marker)?;
        let value_start = name_pos + value_pos_relative + value_marker.len();

        let rest = &raw_data[value_start..];
        let value_end = rest.find(r#"\""#)?;
        let value_str = &rest[..value_end];

        if value_str.is_empty() {
            return None;
        }

        let first_char = value_str.chars().next()?;
        if first_char.is_ascii_digit() || first_char == '-' {
            Some(value_str.to_string())
        } else {
            None
        }
    }

    fn analyze(&self, raw_data: &str) -> ParseAnalysis {
        let mut analysis = ParseAnalysis::default();

        // 1. 检查是否为空
        if raw_data.is_empty() {
            analysis.issues.push("响应为空".to_string());
            return analysis;
        }

        // 2. 检查是否包含 "surplus"
        if raw_data.contains("surplus") {
            analysis.contains_surplus = true;

            // 3. 提取 surplus 附近的内容
            if let Some(pos) = raw_data.find("surplus") {
                let start = pos.saturating_sub(20);
                let end = (pos + 50).min(raw_data.len());
                analysis.surplus_context = Some(raw_data[start..end].to_string());
            }
        } else {
            analysis
                .issues
                .push("响应中不包含 'surplus' 字段".to_string());
        }

        // 4. 检查是否为JSON格式
        if raw_data.trim().starts_with('{') || raw_data.trim().starts_with('[') {
            analysis.is_json = true;
        } else {
            analysis.issues.push("响应不是JSON格式".to_string());
        }

        // 5. 检查是否包含错误信息
        let error_keywords = ["error", "Error", "错误", "fail", "Fail"];
        for keyword in &error_keywords {
            if raw_data.contains(keyword) {
                analysis.contains_error_keyword = true;
                analysis.issues.push(format!("包含错误关键词: {}", keyword));
                break;
            }
        }

        // 6. 尝试解析
        if let Some(value) = self.parse(raw_data) {
            analysis.parsed_value = Some(value);
        } else {
            analysis.issues.push("正则表达式匹配失败".to_string());
        }

        analysis
    }
}

/// 解析分析结果
#[derive(Default)]
struct ParseAnalysis {
    contains_surplus: bool,
    surplus_context: Option<String>,
    is_json: bool,
    contains_error_keyword: bool,
    parsed_value: Option<String>,
    issues: Vec<String>,
}

impl ParseAnalysis {
    fn print(&self, room_id: u32) {
        println!("  房间 {} 分析结果:", room_id);
        println!(
            "    - 包含 'surplus': {}",
            if self.contains_surplus { "✅" } else { "❌" }
        );
        println!(
            "    - JSON 格式: {}",
            if self.is_json { "✅" } else { "❌" }
        );
        println!(
            "    - 包含错误关键词: {}",
            if self.contains_error_keyword {
                "⚠️ 是"
            } else {
                "否"
            }
        );

        if let Some(context) = &self.surplus_context {
            println!("    - surplus 上下文: \"{}\"", context);
        }

        if let Some(value) = &self.parsed_value {
            println!("    - ✅ 解析成功: {}", value);
        } else {
            println!("    - ❌ 解析失败");
        }

        if !self.issues.is_empty() {
            println!("    - 🔍 发现的问题:");
            for issue in &self.issues {
                println!("      • {}", issue);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 🔍 数据解析失败调试工具 ===");
    println!();

    let url_prefix = "https://zywxhd02.gxust.edu.cn/Home/GetRoomInfo?roomid=";
    let test_room_ids = vec![3243, 1714, 635];

    println!("📡 URL 前缀: {}", url_prefix);
    println!("🎯 测试房间: {:?}", test_room_ids);
    println!();

    // 初始化客户端和解析器
    let client = DebugClient::new(url_prefix)?;
    let parser = DebugParser::new()?;

    // 逐个测试房间
    for (index, &room_id) in test_room_ids.iter().enumerate() {
        println!(
            "─── 测试 {}/{}: 房间 {} ───",
            index + 1,
            test_room_ids.len(),
            room_id
        );

        match client.fetch_raw(room_id).await {
            Ok(raw_data) => {
                println!("✅ HTTP 请求成功");
                println!("📦 原始响应长度: {} 字节", raw_data.len());
                println!();

                // 显示原始响应（前500字符）
                println!("📄 原始响应内容（前500字符）:");
                println!("┌─────────────────────────────────────────┐");
                let preview = if raw_data.len() > 500 {
                    format!("{}...", &raw_data[..500])
                } else {
                    raw_data.clone()
                };
                for line in preview.lines() {
                    println!("│ {}", line);
                }
                println!("└─────────────────────────────────────────┘");
                println!();

                // 测试实际解析结果
                let parsed = parser.parse(&raw_data);
                println!("🔬 实际解析结果:");
                match parsed {
                    Some(value) => {
                        if value == "ROOM_NOT_FOUND" {
                            println!("  ⚠️  房间不存在（特殊标记）");
                        } else {
                            println!("  ✅ 成功解析: {} 元", value);
                        }
                    }
                    None => {
                        println!("  ❌ 解析失败");
                    }
                }

                // 分析解析问题
                let analysis = parser.analyze(&raw_data);
                analysis.print(room_id);
                println!();
            }
            Err(e) => {
                println!("❌ HTTP 请求失败: {}", e);
                println!();
            }
        }
    }

    // 总结
    println!("=== 📊 调试总结 ===");
    println!();
    println!("🔍 常见解析失败原因:");
    println!("  1. 房间不存在 → API 返回错误信息而非电费数据");
    println!("  2. 'surplus' 字段格式异常 → 值为空字符串或非数字");
    println!("  3. API 返回格式变化 → JSON结构改变");
    println!("  4. 网络超时/错误 → 返回HTML错误页面");
    println!();
    println!("💡 建议:");
    println!("  - 检查 API 返回的实际内容");
    println!("  - 验证正则表达式是否需要更新");
    println!("  - 考虑添加更宽松的解析逻辑");
    println!("  - 区分 '房间不存在' 和 '真正的解析错误'");

    Ok(())
}
