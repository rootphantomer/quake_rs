// API 密钥管理模块

use std::fs::{create_dir, File};
use std::path::Path;
use std::{fs, io};

use crate::models::{Output, Service};
use crate::client::Quake;
use std::io::Write;

/// ApiKey 结构体，用于管理 API 密钥和 GPT API 密钥的初始化、设置和获取操作
pub struct ApiKey;

impl ApiKey {
    /// 获取配置目录路径
    fn config_dir() -> String {
        let home = dirs::home_dir()
            .expect("Failed to get home directory")
            .to_str()
            .unwrap()
            .to_string();
        format!("{}/.config/quake", home)
    }

    /// 确保目录存在，不存在则创建
    fn ensure_dir(path: &str) {
        if !Path::new(path).exists() {
            if let Err(e) = create_dir(path) {
                Output::error(&format!("创建路径失败: {}. {}", path, e));
                std::process::exit(1);
            }
        }
    }

    /// 初始化 Quake API 密钥
    pub fn init(api_key: String) {
        if Self::set_api(api_key) {
            Output::success("成功初始化");
        } else {
            Output::error("错误: 无效的 API 密钥");
        }
    }

    /// 初始化 GPT API 密钥
    pub fn gptinit(gpt_key: String) {
        if Self::set_gptapi(gpt_key) {
            Output::success("成功初始化");
        } else {
            Output::error("错误: 无效的 API 密钥");
        }
    }

    /// 检查 Quake API 密钥是否可用
    fn check_api(apikey: String) -> bool {
        let (local, one_years_ago) = crate::models::getdate();
        let s = Service {
            ip_list: Vec::new(),
            query: String::from("port:80"),
            start: 1,
            size: 1,
            ignore_cache: false,
            latest: false,
            start_time: one_years_ago,
            end_time: local,
            shortcuts: Vec::new(),
        };
        Quake::new(apikey).search(s).is_ok()
    }

    /// 检查 GPT API 密钥是否可用（目前直接返回 true）
    fn check_gptapi(_apikey: String) -> bool {
        true
    }

    /// 将密钥写入指定文件
    fn write_key(filename: &str, apikey: &str) -> bool {
        let config_path = Self::config_dir();
        Self::ensure_dir(&config_path);
        let key_path = format!("{}/{}", config_path, filename);

        let mut file = match File::create(Path::new(&key_path)) {
            Ok(f) => f,
            Err(e) => {
                Output::error(&format!("文件创建失败: {}", e));
                std::process::exit(1);
            }
        };
        match file.write_all(apikey.as_bytes()) {
            Ok(_) => true,
            Err(e) => {
                Output::error(&format!("文件写入失败: {}", e));
                std::process::exit(1);
            }
        }
    }

    /// 设置 Quake API 密钥
    pub fn set_api(apikey: String) -> bool {
        if Self::check_api(apikey.clone()) {
            return Self::write_key("api_key", &apikey);
        }
        false
    }

    /// 获取 Quake API 密钥
    pub fn get_api() -> Result<String, io::Error> {
        let path = format!("{}/api_key", Self::config_dir());
        fs::read_to_string(path)
    }

    /// 设置 GPT API 密钥
    pub fn set_gptapi(apikey: String) -> bool {
        if Self::check_gptapi(apikey.clone()) {
            return Self::write_key("gptapi_key", &apikey);
        }
        false
    }

    /// 获取 GPT API 密钥
    pub fn get_gptapi() -> Result<String, io::Error> {
        let path = format!("{}/gptapi_key", Self::config_dir());
        fs::read_to_string(path)
    }
}
