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

use electricity_monitor_backend::AppConfig;

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
    println!(
        "🗄️  操作: {}",
        if revert {
            "回滚迁移"
        } else {
            "运行迁移"
        }
    );
    println!();

    // 加载配置
    let config = match AppConfig::load_for_environment(&environment) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("❌ 加载配置失败: {}", e);
            std::process::exit(1);
        }
    };

    let database_url = config.database.connection_url();

    println!(
        "🔗 数据库连接: postgres://{}:***@{}:{}/{}",
        config.database.username,
        config.database.host,
        config.database.port,
        config.database.database
    );
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
