// 数据模型层：包含数据结构体、输出工具和时间工具函数

use ansi_term::Colour::{Blue, Green, Red, Yellow};
use chrono::{Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// 派生序列化、反序列化和调试输出的 trait
#[derive(Serialize, Deserialize, Debug)]
/// 表示服务查询的结构体，包含查询所需的各种参数
pub struct Service {
    /// 查询语句，用于指定查询的条件
    pub query: String,
    /// 查询的起始位置，用于分页查询
    pub start: i32,
    /// 查询结果的数量，即每页返回的记录数
    pub size: i32,
    /// 是否忽略缓存，true 表示忽略缓存，直接从数据源查询
    pub ignore_cache: bool,
    /// 是否只查询最新数据，true 表示只返回最新的数据
    pub latest: bool,
    /// 查询的开始时间，格式通常为特定的时间字符串
    pub start_time: String,
    /// 查询的结束时间，格式通常为特定的时间字符串
    pub end_time: String,
    /// IP 地址列表，用于筛选特定 IP 相关的数据
    pub ip_list: Vec<Value>,
    /// 快捷方式列表，可能包含一些预设的查询条件或配置
    pub shortcuts: Vec<Value>,
}

/*
  TODO: Comment
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Scroll {
    pub query: String,
    pub size: i32,
    pub ignore_cache: bool,
    pub latest: bool,
    pub pagination_id: String,
    pub start_time: String,
    pub end_time: String,
    pub ip_list: Vec<Value>,
    pub shortcuts: Vec<Value>,
}

/*
  TODO: Comment
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Host {
    pub query: String,
    pub start: i32,
    pub size: i32,
    pub ignore_cache: bool,
}

/*
  TODO: Comment
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct ScrollHost {
    pub query: String,
    pub size: i32,
    pub pagination_id: String,
    pub ignore_cache: bool,
}

/*
  TODO: Comment
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct AggService {
    pub query: String,
    pub start: i32,
    pub size: i32,
    pub ignore_cache: bool,
    pub aggregation_list: Vec<String>,
}

/// 输出工具结构体，提供带颜色标记的控制台输出方法
pub struct Output;

impl Output {
    /// 输出错误信息（红色）
    pub fn error(msg: &str) {
        println!("{} {}", Red.bold().paint("[!]"), msg);
    }
    /// 输出提示信息（蓝色）
    pub fn info(msg: &str) {
        println!("{} {}", Blue.bold().paint("[+]"), msg);
    }
    /// 输出成功信息（绿色）
    pub fn success(msg: &str) {
        println!("{} {}", Green.bold().paint("[+]"), msg);
    }
    /// 输出警告信息（黄色）
    pub fn warning(msg: &str) {
        println!("{} {}", Yellow.bold().paint("[-]"), msg);
    }
}

/// 获取指定时间，一年前的日期
///
/// # 参数
/// - `manual_date`: 手动指定的日期字符串，格式为 "%Y-%m-%d"
///
/// # 返回值
/// 一年前日期的字符串表示
pub fn getdate_for_manual(manual_date: &str) -> String {
    let manual_date = NaiveDate::parse_from_str(manual_date, "%Y-%m-%d").unwrap();
    let one_years_ago = manual_date - Duration::days(365);
    one_years_ago.format("%Y-%m-%d").to_string()
}

/// 获取当前时间和当前时间一年前的时间
///
/// # 返回值
/// 元组 (当前时间字符串, 一年前时间字符串)
pub fn getdate() -> (String, String) {
    let local = Local::now();
    let one_years_ago = local - Duration::days(365);
    (
        local.format("%Y-%m-%d %H:%M:%S").to_string(),
        one_years_ago.format("%Y-%m-%d %H:%M:%S").to_string(),
    )
}
