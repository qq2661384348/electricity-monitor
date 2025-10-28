//! 数据库迁移工具
//! 
//! 从TOML配置文件读取数据库配置并执行Diesel迁移
//! 
//! 使用方法:
//!   cargo run --bin migrate                    # 使用development环境
//!   cargo run --bin migrate -- production      # 使用production环境
//!   cargo run --bin migrate -- --revert        # 回滚最后一次迁移

use std::env;
use std::process::Command;

fn main() {
    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    
    let mut environment = "development".to_string();
    let mut revert = false;
    
    for arg in args.iter().skip(1) {
        if arg == "--revert" || arg == "-r" {
            revert = true;
        } else if !arg.starts_with("--") {
            environment = arg.clone();
        }
    }
    
    println!("🔧 数据库迁移工具");
    println!("📁 环境: {}", environment);
    println!("🗄️  操作: {}", if revert { "回滚迁移" } else { "运行迁移" });
    println!();
    
    // 加载配置
    let config = match load_config(&environment) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("❌ 加载配置失败: {}", e);
            std::process::exit(1);
        }
    };
    
    // 构建DATABASE_URL
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.username, config.password, config.host, config.port, config.database
    );
    
    println!("🔗 数据库连接: postgres://{}:***@{}:{}/{}", 
        config.username, config.host, config.port, config.database);
    println!();
    
    // 执行diesel命令
    let diesel_cmd = if revert {
        "migration revert"
    } else {
        "migration run"
    };
    
    println!("⚙️  执行: diesel {}", diesel_cmd);
    println!();
    
    let status = Command::new("diesel")
        .args(diesel_cmd.split_whitespace())
        .env("DATABASE_URL", &database_url)
        .env("APP_ENV", &environment)
        .status();
    
    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                println!();
                println!("✅ 迁移完成!");
            } else {
                eprintln!();
                eprintln!("❌ 迁移失败，退出码: {:?}", exit_status.code());
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!();
            eprintln!("❌ 执行diesel命令失败: {}", e);
            eprintln!();
            eprintln!("💡 提示: 请确保已安装 diesel_cli:");
            eprintln!("   cargo install diesel_cli --no-default-features --features postgres");
            std::process::exit(1);
        }
    }
}

#[derive(Debug)]
struct DatabaseConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

fn load_config(environment: &str) -> Result<DatabaseConfig, String> {
    use std::fs;
    
    // 读取default.toml
    let default_path = "config/default.toml";
    let default_content = fs::read_to_string(default_path)
        .map_err(|e| format!("无法读取 {}: {}", default_path, e))?;
    
    // 解析默认配置
    let mut config = parse_database_config(&default_content)?;
    
    // 如果环境配置文件存在，覆盖相应值
    let env_path = format!("config/{}.toml", environment);
    if let Ok(env_content) = fs::read_to_string(&env_path) {
        if let Ok(env_database) = extract_toml_value(&env_content, "database", "database") {
            config.database = env_database;
        }
    }
    
    Ok(config)
}

fn parse_database_config(content: &str) -> Result<DatabaseConfig, String> {
    Ok(DatabaseConfig {
        host: extract_toml_value(content, "database", "host")?,
        port: extract_toml_value(content, "database", "port")?
            .parse()
            .map_err(|e| format!("端口解析失败: {}", e))?,
        username: extract_toml_value(content, "database", "username")?,
        password: extract_toml_value(content, "database", "password")?,
        database: extract_toml_value(content, "database", "database")?,
    })
}

fn extract_toml_value(content: &str, section: &str, key: &str) -> Result<String, String> {
    // 查找section
    let section_marker = format!("[{}]", section);
    let section_start = content.find(&section_marker)
        .ok_or_else(|| format!("未找到配置节: [{}]", section))?;
    
    // 提取section内容（直到下一个section或文件结束）
    let section_content = &content[section_start..];
    let section_end = section_content[section_marker.len()..]
        .find("\n[")
        .map(|pos| pos + section_marker.len())
        .unwrap_or(section_content.len());
    let section_text = &section_content[..section_end];
    
    // 查找key
    for line in section_text.lines() {
        let line = line.trim();
        if line.starts_with(&format!("{} =", key)) || line.starts_with(&format!("{}=", key)) {
            // 提取值
            if let Some(value_start) = line.find('=') {
                let value = line[value_start + 1..].trim();
                // 移除引号
                let value = value.trim_matches('"').trim_matches('\'');
                // 移除注释
                let value = value.split('#').next().unwrap_or(value).trim();
                return Ok(value.to_string());
            }
        }
    }
    
    Err(format!("未找到配置项: {}.{}", section, key))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_toml_value() {
        let content = r#"
[database]
host = "localhost"
port = 5432
username = "postgres"
password = "secret"
database = "test_db"
        "#;
        
        assert_eq!(extract_toml_value(content, "database", "host").unwrap(), "localhost");
        assert_eq!(extract_toml_value(content, "database", "port").unwrap(), "5432");
        assert_eq!(extract_toml_value(content, "database", "username").unwrap(), "postgres");
    }
}
