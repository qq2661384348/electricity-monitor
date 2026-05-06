//! 数据库迁移工具
//!
//! 从 TOML 配置文件读取数据库配置并执行内嵌 Diesel 迁移
//!
//! 使用方法:
//!   cargo run --bin migrate                    # 使用development环境
//!   cargo run --bin migrate -- production      # 使用production环境
//!   cargo run --bin migrate -- --revert        # 回滚最后一次迁移

use std::env;
use std::error::Error;

use diesel::{Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use electricity_monitor_backend::AppConfig;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

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

    println!(
        "⚙️  执行内嵌迁移: {}",
        if revert {
            "revert last migration"
        } else {
            "run pending migrations"
        }
    );
    println!();

    match run_embedded_migrations(&database_url, revert) {
        Ok(()) => {
            println!();
            println!("✅ 迁移完成!");
        }
        Err(e) => {
            eprintln!();
            eprintln!("❌ 迁移失败: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_embedded_migrations(
    database_url: &str,
    revert: bool,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let mut connection = PgConnection::establish(database_url)?;

    if revert {
        let reverted = connection.revert_last_migration(MIGRATIONS)?;
        println!("↩️  已回滚迁移: {:?}", reverted);
        return Ok(());
    }

    let applied = connection.run_pending_migrations(MIGRATIONS)?;
    if applied.is_empty() {
        println!("ℹ️  没有待执行迁移");
    } else {
        for migration in applied {
            println!("✅ 已执行迁移: {:?}", migration);
        }
    }

    Ok(())
}
