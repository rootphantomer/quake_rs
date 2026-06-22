// 持久化层：文件读写与数据保存

use crate::display;
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::{env, fs};

/// 通用的文件保存辅助函数：打开文件并逐行写入
fn save_lines(filename: &str, lines: &[String]) -> io::Result<i32> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;
    let mut count = 0;
    for line in lines {
        f.write_all(format!("{}\n", line).as_bytes())?;
        count += 1;
    }
    Ok(count)
}

/// 获取当前系统的换行符
fn newline() -> &'static str {
    match env::consts::OS {
        "windows" => "\r\n",
        _ => "\n",
    }
}

pub fn save_domain_data(filename: &str, content: Value, data_type: Vec<&str>) -> io::Result<i32> {
    let domains = display::show_domain(content, false, false, data_type);
    save_lines(filename, &domains)
}

pub fn save_host_data(filename: &str, content: Value) -> io::Result<i32> {
    let hosts = display::show_host(content, false);
    save_lines(filename, &hosts)
}

pub fn save_search_data(
    filename: &str,
    content: Value,
    filter: &str,
    data_type: Vec<&str>,
) -> io::Result<i32> {
    let results = display::show(content, false, filter, data_type);
    save_lines(filename, &results)
}

pub fn save_scroll_data(
    filename: &str,
    content: Vec<Value>,
    filter: &str,
    data_type: Vec<&str>,
) -> io::Result<i32> {
    let results = display::show_scroll(content, false, filter, data_type);
    save_lines(filename, &results)
}

pub fn save_host_by_scroll_data(filename: &str, content: Vec<Value>) -> io::Result<i32> {
    let hosts = display::show_host_by_scroll(content, false);
    save_lines(filename, &hosts)
}

/// 读取文件中的查询行，用 " OR " 连接
pub fn read_file_search(filename: &str) -> String {
    let contents = fs::read_to_string(filename).unwrap();
    let contents = contents.trim_end();
    contents.replace(newline(), " OR ")
}

/// 读取文件中的 IP 地址行，构建 ip:"..." OR ip:"..." 格式
pub fn read_file_host(filename: &str) -> String {
    let contents = fs::read_to_string(filename).unwrap();
    let contents = contents.trim_end();
    let query = contents.replace(newline(), "\" OR ip:\"");
    format!("ip:\"{}\"", query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ========== newline 测试 ==========

    #[test]
    fn test_newline_returns_string() {
        let nl = newline();
        assert!(!nl.is_empty());
    }

    #[test]
    fn test_newline_not_windows() {
        // 在 macOS/Linux 上应返回 "\n"
        if env::consts::OS != "windows" {
            assert_eq!(newline(), "\n");
        }
    }

    // ========== save_lines 测试 ==========

    #[test]
    fn test_save_lines_basic() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let lines = vec![
            "line1".to_string(),
            "line2".to_string(),
            "line3".to_string(),
        ];
        let count = save_lines(path, &lines).unwrap();
        assert_eq!(count, 3);
        let contents = fs::read_to_string(path).unwrap();
        assert!(contents.contains("line1\n"));
        assert!(contents.contains("line2\n"));
        assert!(contents.contains("line3\n"));
    }

    #[test]
    fn test_save_lines_empty() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let lines: Vec<String> = vec![];
        let count = save_lines(path, &lines).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_save_lines_append_mode() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        // 第一次写入
        save_lines(path, &["first".to_string()]).unwrap();
        // 第二次追加
        save_lines(path, &["second".to_string()]).unwrap();
        let contents = fs::read_to_string(path).unwrap();
        assert!(contents.contains("first"));
        assert!(contents.contains("second"));
    }

    // ========== read_file_search 测试 ==========

    /// 辅助函数：创建临时文件并写入内容，返回（文件路径, NamedTempFile）以保持文件存活
    fn create_temp_file(content: &str) -> (String, NamedTempFile) {
        let mut tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        write!(tmp, "{}", content).unwrap();
        (path, tmp)
    }

    #[test]
    fn test_read_file_search_single_line() {
        let (path, _tmp) = create_temp_file("port:80\n");
        let result = read_file_search(&path);
        assert_eq!(result, "port:80");
    }

    #[test]
    fn test_read_file_search_multiple_lines() {
        let (path, _tmp) = create_temp_file("port:80\nport:443\nport:8080\n");
        let result = read_file_search(&path);
        assert_eq!(result, "port:80 OR port:443 OR port:8080");
    }

    #[test]
    fn test_read_file_search_trims_trailing_or() {
        let (path, _tmp) = create_temp_file("port:80\nport:443\n");
        let result = read_file_search(&path);
        assert!(!result.ends_with(" OR "));
    }

    // ========== read_file_host 测试 ==========

    #[test]
    fn test_read_file_host_single_ip() {
        let (path, _tmp) = create_temp_file("1.1.1.1\n");
        let result = read_file_host(&path);
        assert_eq!(result, "ip:\"1.1.1.1\"");
    }

    #[test]
    fn test_read_file_host_multiple_ips() {
        let (path, _tmp) = create_temp_file("1.1.1.1\n8.8.8.8\n192.168.1.1\n");
        let result = read_file_host(&path);
        assert_eq!(
            result,
            "ip:\"1.1.1.1\" OR ip:\"8.8.8.8\" OR ip:\"192.168.1.1\""
        );
    }

    #[test]
    fn test_read_file_host_wraps_in_ip_prefix() {
        let (path, _tmp) = create_temp_file("10.0.0.1\n");
        let result = read_file_host(&path);
        assert!(result.starts_with("ip:\""));
        assert!(result.ends_with("\""));
    }

    // ========== save_* 函数集成测试 ==========

    #[test]
    fn test_save_domain_data() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let content = serde_json::json!({
            "code": 0,
            "data": [{
                "ip": "1.2.3.4",
                "port": 443,
                "service": {"http": {"host": "example.com", "title": "Test"}}
            }],
            "meta": {"pagination": {"count": 1, "total": 1}}
        });
        let count = save_domain_data(path, content, vec!["domain", "ip"]).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_save_host_data() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let content = serde_json::json!({
            "code": 0,
            "data": [{
                "ip": "10.0.0.1",
                "location": {"country_en": "US", "province_en": "CA", "city_en": "SF"},
                "services": [{"port": 80, "name": "http", "time": "2024-01-01"}]
            }],
            "meta": {"pagination": {"count": 1, "total": 1}}
        });
        let count = save_host_data(path, content).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_save_search_data() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let content = serde_json::json!({
            "code": 0,
            "data": [{
                "ip": "1.1.1.1",
                "port": 80,
                "service": {"name": "http", "http": {"title": "Test", "host": "test.com"}},
                "location": {"country_cn": "US", "province_cn": "CA", "city_cn": "SF", "owner": "Org"},
                "time": "2024-01-01"
            }],
            "meta": {"pagination": {"count": 1, "total": 1}}
        });
        let count = save_search_data(path, content, "", vec!["ip", "port"]).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_save_scroll_data() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let content = vec![serde_json::json!({
            "ip": "2.2.2.2",
            "port": 443,
            "service": {"name": "https", "http": {"title": "Test", "host": "test.com"}},
            "location": {"country_cn": "US", "province_cn": "CA", "city_cn": "SF", "owner": "Org"},
            "time": "2024-01-01"
        })];
        let count = save_scroll_data(path, content, "", vec!["ip"]).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_save_host_by_scroll_data() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let content = vec![serde_json::json!({
            "ip": "10.0.0.1",
            "location": {"country_en": "US", "province_en": "NY", "city_en": "NYC"},
            "services": [{"port": 22, "name": "ssh", "time": "2024-01-01"}]
        })];
        let count = save_host_by_scroll_data(path, content).unwrap();
        assert_eq!(count, 1);
    }
}
