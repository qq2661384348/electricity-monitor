//! Cargo 构建脚本
//!
//! 说明：
//! - 使用 `--features static-build` 时启用静态链接（Docker/Linux）
//! - 默认模式使用系统安装的库（Windows 开发）
//!
//! Windows 开发环境需要：
//! 1. PostgreSQL 16+ 安装（包含 libpq）
//! 2. OpenSSL 安装（推荐 https://slproweb.com/products/Win32OpenSSL.html）
//!
//! 环境变量：
//! - PQ_LIB_DIR: PostgreSQL 库目录（可选，自动检测）
//! - OPENSSL_DIR: OpenSSL 安装目录（Windows 必需）
//!
//! 参考: https://www.edu4rdshl.dev/posts/rust-binaries-with-diesel-and-postgres-static-linking-on-2025/

use std::env;
use std::path::PathBuf;

fn main() {
    // 触发重新构建的条件
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=PQ_LIB_DIR");
    println!("cargo:rerun-if-env-changed=OPENSSL_DIR");
    
    // Windows 平台特殊配置
    #[cfg(target_os = "windows")]
    configure_windows();
}

/// Windows 平台配置
#[cfg(target_os = "windows")]
fn configure_windows() {
    // 1. PostgreSQL 路径检测
    let pg_home = find_postgres_home();
    
    if let Some(ref pg) = pg_home {
        if env::var("PQ_LIB_DIR").is_err() {
            let lib_dir = pg.join("lib");
            println!("cargo:rustc-link-search=native={}", lib_dir.display());
        }
        
        // 2. 使用 PostgreSQL 自带的 OpenSSL（推荐方案）
        // PostgreSQL 16+ 包含完整的 OpenSSL 开发文件
        if env::var("OPENSSL_DIR").is_err() {
            let include_dir = pg.join("include").join("openssl");
            let lib_dir = pg.join("lib");
            
            if include_dir.exists() && lib_dir.join("libssl.lib").exists() {
                println!("cargo:warning=使用 PostgreSQL 自带的 OpenSSL: {}", pg.display());
                // 注意：这些环境变量在 build.rs 中设置不会影响 openssl-sys 的构建
                // 需要在 .cargo/config.toml 中设置
            }
        }
    }
    
    // 3. 如果没有找到 PostgreSQL，检查独立 OpenSSL 安装
    if pg_home.is_none() && env::var("OPENSSL_DIR").is_err() {
        let openssl_paths = vec![
            r"C:\Program Files\OpenSSL-Win64",
            r"C:\OpenSSL-Win64",
        ];
        
        for path in &openssl_paths {
            let p = PathBuf::from(path);
            if p.join("include").exists() && p.join("lib").exists() {
                println!("cargo:warning=检测到 OpenSSL: {}", path);
                return;
            }
        }
        
        println!("cargo:warning=未检测到 OpenSSL 安装");
        println!("cargo:warning=推荐方案：安装 PostgreSQL 16+（自带 OpenSSL）");
        println!("cargo:warning=或从 https://slproweb.com/products/Win32OpenSSL.html 下载完整版");
    }
}

/// 查找 PostgreSQL 安装目录
#[cfg(target_os = "windows")]
fn find_postgres_home() -> Option<PathBuf> {
    // 优先检查环境变量
    if let Ok(home) = env::var("POSTGRES_HOME") {
        let path = PathBuf::from(&home);
        if path.exists() {
            return Some(path);
        }
    }
    
    // 检查默认安装路径
    let default_paths = vec![
        r"C:\Program Files\PostgreSQL\17",
        r"C:\Program Files\PostgreSQL\16",
        r"C:\Program Files\PostgreSQL\15",
        r"C:\Program Files\PostgreSQL\14",
    ];
    
    for path_str in default_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            return Some(path);
        }
    }
    
    None
}
