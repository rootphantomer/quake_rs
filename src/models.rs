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

#[cfg(test)]
mod tests {
    use super::*;

    // ========== getdate_for_manual 测试 ==========

    #[test]
    fn test_getdate_for_manual_normal_date() {
        let result = getdate_for_manual("2024-01-01");
        assert_eq!(result, "2023-01-01");
    }

    #[test]
    fn test_getdate_for_manual_leap_year() {
        // 2024-03-01 减去 365 天 → 2023-03-02 (因为 2024 是闰年)
        let result = getdate_for_manual("2024-03-01");
        assert_eq!(result, "2023-03-02");
    }

    #[test]
    fn test_getdate_for_manual_year_boundary() {
        // 2023-01-01 减去 365 天 → 2022-01-01
        let result = getdate_for_manual("2023-01-01");
        assert_eq!(result, "2022-01-01");
    }

    #[test]
    fn test_getdate_for_manual_specific_date() {
        // 2020 是闰年，2020-12-25 减去 365 天 → 2019-12-26
        let result = getdate_for_manual("2020-12-25");
        assert_eq!(result, "2019-12-26");
    }

    // ========== getdate 测试 ==========

    #[test]
    fn test_getdate_returns_two_strings() {
        let (now, one_year_ago) = getdate();
        assert!(!now.is_empty());
        assert!(!one_year_ago.is_empty());
    }

    #[test]
    fn test_getdate_format() {
        let (now, one_year_ago) = getdate();
        // 验证格式: YYYY-MM-DD HH:MM:SS
        let parts: Vec<&str> = now.split(' ').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].split('-').count(), 3);
        assert_eq!(parts[1].split(':').count(), 3);
        // 一年前的格式也应相同
        let parts_ago: Vec<&str> = one_year_ago.split(' ').collect();
        assert_eq!(parts_ago.len(), 2);
    }

    #[test]
    fn test_getdate_one_year_diff() {
        let (now, one_year_ago) = getdate();
        let now_date =
            NaiveDate::parse_from_str(now.split(' ').next().unwrap(), "%Y-%m-%d").unwrap();
        let ago_date =
            NaiveDate::parse_from_str(one_year_ago.split(' ').next().unwrap(), "%Y-%m-%d").unwrap();
        let diff = now_date - ago_date;
        assert!(diff.num_days() >= 364 && diff.num_days() <= 366);
    }

    // ========== Serde 序列化/反序列化测试 ==========

    #[test]
    fn test_service_serde_roundtrip() {
        let s = Service {
            query: "port:80".to_string(),
            start: 0,
            size: 10,
            ignore_cache: true,
            latest: false,
            start_time: "2023-01-01 00:00:00".to_string(),
            end_time: "2024-01-01 00:00:00".to_string(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        let json = serde_json::to_string(&s).unwrap();
        let deserialized: Service = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.query, "port:80");
        assert_eq!(deserialized.start, 0);
        assert_eq!(deserialized.size, 10);
        assert!(deserialized.ignore_cache);
        assert!(!deserialized.latest);
    }

    #[test]
    fn test_scroll_serde_roundtrip() {
        let s = Scroll {
            query: "port:443".to_string(),
            size: 20,
            ignore_cache: false,
            latest: true,
            pagination_id: "abc123".to_string(),
            start_time: "2023-06-01 00:00:00".to_string(),
            end_time: "2024-06-01 00:00:00".to_string(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        let json = serde_json::to_string(&s).unwrap();
        let deserialized: Scroll = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pagination_id, "abc123");
        assert_eq!(deserialized.size, 20);
    }

    #[test]
    fn test_host_serde_roundtrip() {
        let h = Host {
            query: "ip:1.1.1.1".to_string(),
            start: 0,
            size: 5,
            ignore_cache: true,
        };
        let json = serde_json::to_string(&h).unwrap();
        let deserialized: Host = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.query, "ip:1.1.1.1");
        assert_eq!(deserialized.size, 5);
    }

    #[test]
    fn test_scroll_host_serde_roundtrip() {
        let sh = ScrollHost {
            query: "ip:8.8.8.8".to_string(),
            size: 10,
            pagination_id: "page2".to_string(),
            ignore_cache: false,
        };
        let json = serde_json::to_string(&sh).unwrap();
        let deserialized: ScrollHost = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pagination_id, "page2");
        assert!(!deserialized.ignore_cache);
    }

    #[test]
    fn test_agg_service_serde_roundtrip() {
        let a = AggService {
            query: "app:\"*蜜罐*\" AND ip:1.1.1.1".to_string(),
            start: 0,
            size: 5,
            ignore_cache: false,
            aggregation_list: vec!["app".to_string()],
        };
        let json = serde_json::to_string(&a).unwrap();
        let deserialized: AggService = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.aggregation_list, vec!["app"]);
        assert_eq!(deserialized.query, "app:\"*蜜罐*\" AND ip:1.1.1.1");
    }

    #[test]
    fn test_service_with_ip_list() {
        let s = Service {
            query: String::new(),
            start: 0,
            size: 100,
            ignore_cache: false,
            latest: false,
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![
                Value::String("1.1.1.1".to_string()),
                Value::String("8.8.8.8".to_string()),
            ],
            shortcuts: vec![],
        };
        let json = serde_json::to_string(&s).unwrap();
        let deserialized: Service = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.ip_list.len(), 2);
        assert_eq!(
            deserialized.ip_list[0],
            Value::String("1.1.1.1".to_string())
        );
    }

    // ========== Output 结构体测试 ==========

    #[test]
    fn test_output_methods_exist() {
        // 验证 Output 方法可以被调用（不验证输出内容）
        Output::error("test error");
        Output::info("test info");
        Output::success("test success");
        Output::warning("test warning");
    }
}
