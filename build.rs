//! Cargo 构建脚本 - 自动配置 PostgreSQL 链接
//!
//! 功能：
//! 1. 检测 PostgreSQL 安装路径
//! 2. 自动配置链接库和依赖
//! 3. Windows 平台特殊处理

use std::env;
use std::path::{Path, PathBuf};

fn main() {
    // 1. 获取 PostgreSQL 库路径
    let pg_lib_dir = get_postgres_lib_dir();
    
    println!("cargo:rerun-if-env-changed=PQ_LIB_DIR");
    println!("cargo:rerun-if-env-changed=POSTGRES_HOME");
    
    // 2. 配置链接搜索路径
    if let Some(lib_dir) = pg_lib_dir {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        
        // 3. Windows 平台特殊配置
        if cfg!(target_os = "windows") {
            configure_windows_linking();
        }
    }
}

/// 获取 PostgreSQL 库目录
fn get_postgres_lib_dir() -> Option<PathBuf> {
    // 优先级1: 环境变量 PQ_LIB_DIR（必须是有效路径且包含libpq）
    if let Ok(dir) = env::var("PQ_LIB_DIR") {
        let path = PathBuf::from(&dir);
        // 验证路径存在且包含 libpq 库
        if path.exists() && is_valid_postgres_lib(&path) {
            println!("cargo:warning=使用 PQ_LIB_DIR: {}", path.display());
            return Some(path);
        } else {
            println!("cargo:warning=PQ_LIB_DIR 路径无效或缺少 libpq: {}", dir);
            println!("cargo:warning=将尝试自动检测 PostgreSQL...");
        }
    }
    
    // 优先级2: 环境变量 POSTGRES_HOME/lib
    if let Ok(home) = env::var("POSTGRES_HOME") {
        let path = PathBuf::from(home).join("lib");
        if path.exists() {
            println!("cargo:warning=使用 POSTGRES_HOME: {}", path.display());
            return Some(path);
        }
    }
    
    // 优先级3: Windows 默认安装路径
    #[cfg(target_os = "windows")]
    {
        let default_paths = vec![
            r"C:\Program Files\PostgreSQL\16\lib",
            r"C:\Program Files\PostgreSQL\15\lib",
            r"C:\Program Files\PostgreSQL\14\lib",
        ];
        
        for path_str in default_paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                println!("cargo:warning=自动检测到 PostgreSQL: {}", path.display());
                return Some(path);
            }
        }
    }
    
    // 优先级4: Linux 默认路径
    #[cfg(target_os = "linux")]
    {
        let default_paths = vec![
            "/usr/lib/postgresql/16/lib",
            "/usr/lib/postgresql/15/lib",
            "/usr/lib/x86_64-linux-gnu",
            "/usr/lib",
        ];
        
        for path_str in default_paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Some(path);
            }
        }
    }
    
    println!("cargo:warning=未找到 PostgreSQL 库目录，请设置 PQ_LIB_DIR 或 POSTGRES_HOME 环境变量");
    None
}

/// Windows 平台链接配置
#[cfg(target_os = "windows")]
fn configure_windows_linking() {
    // PostgreSQL 核心库
    println!("cargo:rustc-link-lib=libpq");
    println!("cargo:rustc-link-lib=libpgcommon");
    println!("cargo:rustc-link-lib=libpgport");
    
    // OpenSSL 依赖
    println!("cargo:rustc-link-lib=libssl");
    println!("cargo:rustc-link-lib=libcrypto");
    
    // 其他依赖
    println!("cargo:rustc-link-lib=libintl");
    
    // Windows 系统库
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=secur32");
    println!("cargo:rustc-link-lib=ws2_32");
    println!("cargo:rustc-link-lib=advapi32");
    println!("cargo:rustc-link-lib=crypt32");
}

/// Linux 平台链接配置
#[cfg(not(target_os = "windows"))]
fn configure_windows_linking() {
    // Linux 下 diesel 会自动处理
}

/// 验证是否为有效的 PostgreSQL 库目录
fn is_valid_postgres_lib(path: &Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        // Windows 下检查是否存在 libpq.lib
        path.join("libpq.lib").exists()
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Linux 下检查是否存在 libpq.so 或 libpq.a
        path.join("libpq.so").exists() || 
        path.join("libpq.a").exists() ||
        path.parent().and_then(|p| p.parent()).map(|p| {
            p.join("lib").join("libpq.so").exists()
        }).unwrap_or(false)
    }
}
