// use log::{debug, error, info};
use crate::api::ApiKey;
use crate::models::{AggService, Host, Output, Scroll, ScrollHost, Service};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::{Map, Number, Value};
use std::fs;

const BASE_URL: &str = "https://quake.360.net";

/// 快捷方式常量
const SHORTCUT_CDN: &str = "612f5a5ad6b3bdb87961727f";
const SHORTCUT_MG: &str = "610ce2adb1a2e3e1632e67b1";
const SHORTCUT_WXQQ: &str = "62bc12b70537d96695680ce5";
const SHORTCUT_SJQC: &str = "610ce2fbda6d29df72ac56eb";

pub struct Quake {
    api_key: String,
}

impl Quake {
    pub fn new(api_key: String) -> Quake {
        Quake { api_key }
    }

    /// 构建认证请求头
    fn header(&self) -> HeaderMap {
        let mut header = HeaderMap::new();
        header.insert(
            "X-QuakeToken",
            HeaderValue::from_str(&self.api_key).unwrap(),
        );
        header
    }

    /// 发送 POST 请求并返回响应文本
    fn send_post(&self, path: &str, body: &impl serde::Serialize) -> String {
        let url = format!("{}{}", BASE_URL, path);
        let client = Client::new();
        let resp = match client.post(&url).headers(self.header()).json(body).send() {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    Output::error("Connect Timeout!!");
                } else {
                    Output::error(&format!("Connect error!!!\r\n{}", e));
                }
                std::process::exit(1);
            }
        };
        match resp.text() {
            Ok(text) => text,
            Err(e) => {
                if e.is_timeout() {
                    Output::error("Connect Timeout!!");
                } else {
                    Output::error(&format!("Connect error!!!\r\n{}", e));
                }
                std::process::exit(1);
            }
        }
    }

    /// 发送 GET 请求并返回响应文本
    fn send_get(&self, path: &str) -> String {
        let url = format!("{}{}", BASE_URL, path);
        let client = Client::new();
        let resp = match client.get(&url).headers(self.header()).send() {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    Output::error("Connect Timeout!!");
                } else {
                    Output::error(&format!("Connect error!!!\r\n{}", e));
                }
                std::process::exit(1);
            }
        };
        match resp.text() {
            Ok(text) => text,
            Err(e) => {
                if e.is_timeout() {
                    Output::error("Connect Timeout!!");
                } else {
                    Output::error(&format!("Connect error!!!\r\n{}", e));
                }
                std::process::exit(1);
            }
        }
    }

    /// 检查 API 响应码，非 0 时输出错误并退出
    fn check_response(response: &Value) {
        let code = response["code"].to_string();
        if code != "0" {
            let message = response["message"].as_str().unwrap_or("Unknown error");
            Output::error(&format!("Query failed: {}", message));
            std::process::exit(1);
        }
    }

    /// 根据快捷方式标志应用 shortcuts
    fn apply_shortcuts(service: &mut impl Shortcutable, cdn: i32, mg: i32, wxqq: i32, sjqc: i32) {
        if cdn == 1 {
            service.add_shortcut(SHORTCUT_CDN);
        }
        if mg == 1 {
            service.add_shortcut(SHORTCUT_MG);
        }
        if wxqq == 1 {
            service.add_shortcut(SHORTCUT_WXQQ);
        }
        if sjqc == 1 {
            service.add_shortcut(SHORTCUT_SJQC);
        }
    }

    /// 根据时间参数设置时间范围
    fn apply_time_range(
        start_time: &mut String,
        end_time: &mut String,
        time_start: &str,
        time_end: &str,
    ) {
        let (local, one_years_ago) = crate::models::getdate();
        if time_start.is_empty() && time_end.is_empty() {
            *start_time = one_years_ago;
            *end_time = local;
        } else if !time_start.is_empty() && time_end.is_empty() {
            *start_time = time_start.to_string();
            *end_time = local;
        } else if time_start.is_empty() && !time_end.is_empty() {
            *start_time = crate::models::getdate_for_manual(time_end);
            *end_time = time_end.to_string();
        } else {
            *start_time = time_start.to_string();
            *end_time = time_end.to_string();
        }
    }

    // ========== Host 查询 ==========

    pub fn query_host(query_string: &str, start: i32, size: i32) -> Value {
        Output::info(&format!("Search with {}", query_string));
        let res = ApiKey::get_api().expect("Failed to read apikey:\t");

        let h = Host {
            query: String::from(query_string),
            start,
            size,
            ignore_cache: true,
        };
        let response: Value = match Quake::new(res).search_host(&h) {
            Ok(response) => response,
            Err(e) => {
                Output::error(&format!("Query failed: {}", e));
                std::process::exit(1);
            }
        };
        response
    }

    pub fn search_host(&self, host: &Host) -> Result<Value, serde_json::Error> {
        let res = self.send_post("/api/v3/search/quake_host", host);
        let response: Value = serde_json::from_str(&res)?;
        Self::check_response(&response);
        Ok(response)
    }

    pub fn query_host_by_scroll(query_string: &str, size: i32) -> Vec<Value> {
        Output::info(&format!("Search with {}", query_string));
        let res = ApiKey::get_api().expect("Failed to read apikey:\t");
        match Quake::new(res).search_host_by_scroll(query_string, size) {
            Ok(response) => response,
            Err(e) => {
                Output::error(&format!("Query failed: {}", e));
                std::process::exit(1);
            }
        }
    }

    pub fn search_host_by_scroll(
        &self,
        query_string: &str,
        size: i32,
    ) -> Result<Vec<Value>, serde_json::Error> {
        let sh = Self::init_scroll_host(query_string, size, "");
        let res = self.send_post(
            "/api/v3/scroll/quake_host",
            &Self::get_scrollhost_post_data(sh),
        );
        let response: Value = serde_json::from_str(&res)?;
        Self::check_response(&response);

        let data_array = response["data"].as_array().unwrap();
        let pagination_id = response["meta"]["pagination_id"].as_str().unwrap();
        let mut all_data: Vec<Value> = data_array.to_vec();
        let mut data_len = data_array.len();

        while data_len != 0 && (data_len as i32) >= size {
            let s_scroll = Self::init_scroll_host(query_string, size, pagination_id);
            let res_scroll = self.send_post(
                "/api/v3/scroll/quake_host",
                &Self::get_scrollhost_post_data(s_scroll),
            );
            let responses: Value = serde_json::from_str(&res_scroll)?;
            let data_array_for_while = responses["data"].as_array().unwrap();
            all_data.extend(data_array_for_while.iter().cloned());
            data_len = data_array_for_while.len();
        }
        Ok(all_data)
    }

    fn get_scrollhost_post_data(s: ScrollHost) -> Map<String, Value> {
        let mut data: Map<String, Value> = Map::new();
        data.insert("size".to_string(), Value::Number(Number::from(s.size)));
        data.insert("ignore_cache".to_string(), Value::Bool(s.ignore_cache));
        data.insert("query".to_string(), Value::String(s.query));
        if !s.pagination_id.is_empty() {
            data.insert("pagination_id".to_string(), Value::String(s.pagination_id));
        }
        data
    }

    pub fn init_scroll_host(query_string: &str, size: i32, pagination_id: &str) -> ScrollHost {
        let mut sh = ScrollHost {
            query: String::new(),
            size,
            ignore_cache: true,
            pagination_id: String::new(),
        };
        if query_string.is_empty() {
            Output::info("Search failed");
            std::process::exit(1);
        }
        sh.query = query_string.to_string();
        if !pagination_id.is_empty() {
            sh.pagination_id = pagination_id.to_string();
        }
        sh
    }

    // ========== Service 查询 ==========

    #[allow(clippy::too_many_arguments)]
    pub fn query(
        query_string: &str,
        file_name: &str,
        start: i32,
        size: i32,
        time_start: &str,
        time_end: &str,
        cdn: i32,
        mg: i32,
        zxsj: i32,
        wxqq: i32,
        sjqc: i32,
    ) -> Value {
        let res = ApiKey::get_api().expect("Failed to read apikey:\t");
        let mut s = Service {
            query: String::new(),
            start,
            size,
            ignore_cache: true,
            latest: true,
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        if zxsj == 1 {
            s.ignore_cache = true;
            s.latest = true;
        }
        Self::apply_shortcuts(&mut s, cdn, mg, wxqq, sjqc);
        Self::apply_time_range(&mut s.start_time, &mut s.end_time, time_start, time_end);

        if !file_name.is_empty() {
            let ips: String = match fs::read_to_string(file_name) {
                Ok(res) => res,
                Err(err) => {
                    Output::error(&format!("Failed to read {} : {}", file_name, err));
                    std::process::exit(1);
                }
            };
            s.ip_list = ips.lines().map(|s| Value::String(s.to_string())).collect();
        }
        if !query_string.is_empty() {
            s.query = query_string.to_string();
            Output::info(&format!("Search with {}", query_string));
        } else {
            Output::info(&format!("Search for {} IPs", s.ip_list.len()));
        }
        Output::info(&format!(
            "Data time again {} to {}.",
            s.start_time, s.end_time
        ));

        match Quake::new(res).search(s) {
            Ok(response) => response,
            Err(e) => {
                Output::error(&format!("Query failed: {}", e));
                std::process::exit(1);
            }
        }
    }

    pub fn search(&self, service: Service) -> Result<Value, serde_json::Error> {
        let res = self.send_post(
            "/api/v3/search/quake_service",
            &Self::get_service_post_data(service),
        );
        let response: Value = serde_json::from_str(&res)?;
        Self::check_response(&response);
        Ok(response)
    }

    pub fn get_scroll_data(&self, scroll: Scroll) -> String {
        self.send_post(
            "/api/v3/scroll/quake_service",
            &Self::get_scroll_post_data(scroll),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn init_scroll(
        query_string: &str,
        size: i32,
        time_start: &str,
        time_end: &str,
        cdn: i32,
        mg: i32,
        zxsj: i32,
        wxqq: i32,
        sjqc: i32,
        pagination_id: &str,
    ) -> Scroll {
        let mut s = Scroll {
            query: String::new(),
            size,
            ignore_cache: true,
            latest: true,
            pagination_id: String::new(),
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        if zxsj == 1 {
            s.ignore_cache = true;
            s.latest = true;
        }
        Self::apply_shortcuts(&mut s, cdn, mg, wxqq, sjqc);
        Self::apply_time_range(&mut s.start_time, &mut s.end_time, time_start, time_end);

        if !query_string.is_empty() {
            s.query = query_string.to_string();
        } else {
            Output::info(&format!("Search for {} IPs", s.ip_list.len()));
        }
        if !pagination_id.is_empty() {
            s.pagination_id = pagination_id.to_string();
        }
        s
    }

    #[allow(clippy::too_many_arguments)]
    pub fn query_for_scroll(
        query_string: &str,
        size: i32,
        time_start: &str,
        time_end: &str,
        cdn: i32,
        mg: i32,
        zxsj: i32,
        wxqq: i32,
        sjqc: i32,
    ) -> Vec<Value> {
        let res = ApiKey::get_api().expect("Failed to read apikey:\t");
        match Quake::new(res).scroll(
            query_string,
            size,
            time_start,
            time_end,
            cdn,
            mg,
            zxsj,
            wxqq,
            sjqc,
        ) {
            Ok(response) => response,
            Err(e) => {
                Output::error(&format!("Query failed: {}", e));
                std::process::exit(1);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn scroll(
        &self,
        query_string: &str,
        size: i32,
        time_start: &str,
        time_end: &str,
        cdn: i32,
        mg: i32,
        zxsj: i32,
        wxqq: i32,
        sjqc: i32,
    ) -> Result<Vec<Value>, serde_json::Error> {
        let scroll = Self::init_scroll(
            query_string,
            size,
            time_start,
            time_end,
            cdn,
            mg,
            zxsj,
            wxqq,
            sjqc,
            "",
        );
        let res = self.get_scroll_data(scroll);
        let response: Value = serde_json::from_str(&res)?;
        Self::check_response(&response);

        let data_array = response["data"].as_array().unwrap();
        Ok(data_array.to_vec())
    }

    // ========== 聚合与用户信息 ==========

    pub fn aggservice(&self, agg: &AggService) -> Result<Value, serde_json::Error> {
        let res = self.send_post("/api/v3/aggregation/quake_service", agg);
        let response: Value = serde_json::from_str(&res)?;
        Self::check_response(&response);
        Ok(response)
    }

    pub fn info(&self) -> Result<Value, serde_json::Error> {
        let res = self.send_get("/api/v3/user/info");
        let response: Value = serde_json::from_str(&res)?;
        Self::check_response(&response);
        Ok(response)
    }

    // ========== 数据转换 ==========

    fn get_scroll_post_data(s: Scroll) -> Map<String, Value> {
        let mut data: Map<String, Value> = Map::new();
        data.insert("size".to_string(), Value::Number(Number::from(s.size)));
        data.insert("ignore_cache".to_string(), Value::Bool(s.ignore_cache));
        data.insert("latest".to_string(), Value::Bool(s.latest));
        data.insert("start_time".to_string(), Value::String(s.start_time));
        data.insert("end_time".to_string(), Value::String(s.end_time));
        data.insert("shortcuts".to_string(), Value::Array(s.shortcuts));
        if !s.ip_list.is_empty() {
            data.insert("query".to_string(), Value::String(String::new()));
            data.insert("ip_list".to_string(), Value::Array(s.ip_list));
        } else {
            data.insert("query".to_string(), Value::String(s.query));
        }
        if !s.pagination_id.is_empty() {
            data.insert("pagination_id".to_string(), Value::String(s.pagination_id));
        }
        data
    }

    fn get_service_post_data(s: Service) -> Map<String, Value> {
        let mut data: Map<String, Value> = Map::new();
        data.insert("start".to_string(), Value::Number(Number::from(s.start)));
        data.insert("size".to_string(), Value::Number(Number::from(s.size)));
        data.insert("ignore_cache".to_string(), Value::Bool(s.ignore_cache));
        data.insert("latest".to_string(), Value::Bool(s.latest));
        data.insert("start_time".to_string(), Value::String(s.start_time));
        data.insert("end_time".to_string(), Value::String(s.end_time));
        data.insert("shortcuts".to_string(), Value::Array(s.shortcuts));
        if !s.ip_list.is_empty() {
            data.insert("query".to_string(), Value::String(String::new()));
            data.insert("ip_list".to_string(), Value::Array(s.ip_list));
        } else {
            data.insert("query".to_string(), Value::String(s.query));
        }
        data
    }
}

/// Trait 用于统一 Service 和 Scroll 的 shortcuts 操作
trait Shortcutable {
    fn add_shortcut(&mut self, id: &str);
}

impl Shortcutable for Service {
    fn add_shortcut(&mut self, id: &str) {
        self.shortcuts.push(Value::String(id.to_string()));
    }
}

impl Shortcutable for Scroll {
    fn add_shortcut(&mut self, id: &str) {
        self.shortcuts.push(Value::String(id.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Scroll, ScrollHost, Service};

    // ========== get_service_post_data 测试 ==========

    #[test]
    fn test_get_service_post_data_basic() {
        let s = Service {
            query: "port:80".to_string(),
            start: 0,
            size: 10,
            ignore_cache: true,
            latest: false,
            start_time: "2023-01-01".to_string(),
            end_time: "2024-01-01".to_string(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        let data = Quake::get_service_post_data(s);
        assert_eq!(data["query"], "port:80");
        assert_eq!(data["start"], 0);
        assert_eq!(data["size"], 10);
        assert_eq!(data["ignore_cache"], true);
        assert_eq!(data["latest"], false);
        assert_eq!(data["start_time"], "2023-01-01");
        assert_eq!(data["end_time"], "2024-01-01");
    }

    #[test]
    fn test_get_service_post_data_with_ip_list() {
        let s = Service {
            query: String::new(),
            start: 0,
            size: 10,
            ignore_cache: false,
            latest: true,
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![
                Value::String("1.1.1.1".to_string()),
                Value::String("8.8.8.8".to_string()),
            ],
            shortcuts: vec![],
        };
        let data = Quake::get_service_post_data(s);
        // 当 ip_list 非空时，query 应为空字符串
        assert_eq!(data["query"], "");
        assert!(data["ip_list"].as_array().unwrap().len() == 2);
    }

    #[test]
    fn test_get_service_post_data_with_shortcuts() {
        let s = Service {
            query: "test".to_string(),
            start: 0,
            size: 10,
            ignore_cache: false,
            latest: false,
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![],
            shortcuts: vec![
                Value::String("shortcut1".to_string()),
                Value::String("shortcut2".to_string()),
            ],
        };
        let data = Quake::get_service_post_data(s);
        let shortcuts = data["shortcuts"].as_array().unwrap();
        assert_eq!(shortcuts.len(), 2);
        assert_eq!(shortcuts[0], "shortcut1");
        assert_eq!(shortcuts[1], "shortcut2");
    }

    // ========== get_scroll_post_data 测试 ==========

    #[test]
    fn test_get_scroll_post_data_basic() {
        let s = Scroll {
            query: "port:443".to_string(),
            size: 20,
            ignore_cache: false,
            latest: true,
            pagination_id: "abc123".to_string(),
            start_time: "2023-06-01".to_string(),
            end_time: "2024-06-01".to_string(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        let data = Quake::get_scroll_post_data(s);
        assert_eq!(data["query"], "port:443");
        assert_eq!(data["size"], 20);
        assert_eq!(data["latest"], true);
        assert_eq!(data["pagination_id"], "abc123");
    }

    #[test]
    fn test_get_scroll_post_data_empty_pagination() {
        let s = Scroll {
            query: "test".to_string(),
            size: 10,
            ignore_cache: false,
            latest: false,
            pagination_id: String::new(),
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        let data = Quake::get_scroll_post_data(s);
        // 空 pagination_id 不应出现在 map 中
        assert!(data.get("pagination_id").is_none());
    }

    #[test]
    fn test_get_scroll_post_data_with_ip_list() {
        let s = Scroll {
            query: String::new(),
            size: 10,
            ignore_cache: false,
            latest: false,
            pagination_id: String::new(),
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![Value::String("1.1.1.1".to_string())],
            shortcuts: vec![],
        };
        let data = Quake::get_scroll_post_data(s);
        assert_eq!(data["query"], "");
        assert!(data["ip_list"].as_array().unwrap().len() == 1);
    }

    // ========== get_scrollhost_post_data 测试 ==========

    #[test]
    fn test_get_scrollhost_post_data_basic() {
        let sh = ScrollHost {
            query: "ip:8.8.8.8".to_string(),
            size: 50,
            pagination_id: "page2".to_string(),
            ignore_cache: true,
        };
        let data = Quake::get_scrollhost_post_data(sh);
        assert_eq!(data["query"], "ip:8.8.8.8");
        assert_eq!(data["size"], 50);
        assert_eq!(data["ignore_cache"], true);
        assert_eq!(data["pagination_id"], "page2");
    }

    #[test]
    fn test_get_scrollhost_post_data_empty_pagination() {
        let sh = ScrollHost {
            query: "test".to_string(),
            size: 10,
            pagination_id: String::new(),
            ignore_cache: false,
        };
        let data = Quake::get_scrollhost_post_data(sh);
        assert!(data.get("pagination_id").is_none());
    }

    // ========== init_scroll_host 测试 ==========

    #[test]
    fn test_init_scroll_host_basic() {
        let sh = Quake::init_scroll_host("ip:1.1.1.1", 20, "");
        assert_eq!(sh.query, "ip:1.1.1.1");
        assert_eq!(sh.size, 20);
        assert!(sh.ignore_cache);
        assert!(sh.pagination_id.is_empty());
    }

    #[test]
    fn test_init_scroll_host_with_pagination() {
        let sh = Quake::init_scroll_host("port:80", 10, "page5");
        assert_eq!(sh.pagination_id, "page5");
        assert_eq!(sh.query, "port:80");
    }

    // ========== apply_shortcuts 测试 ==========

    #[test]
    fn test_apply_shortcuts_none() {
        let mut s = Service {
            query: "test".to_string(),
            start: 0,
            size: 10,
            ignore_cache: false,
            latest: false,
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        Quake::apply_shortcuts(&mut s, 0, 0, 0, 0);
        assert!(s.shortcuts.is_empty());
    }

    #[test]
    fn test_apply_shortcuts_cdn() {
        let mut s = Service {
            query: "test".to_string(),
            start: 0,
            size: 10,
            ignore_cache: false,
            latest: false,
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        Quake::apply_shortcuts(&mut s, 1, 0, 0, 0);
        assert_eq!(s.shortcuts.len(), 1);
        assert_eq!(s.shortcuts[0], Value::String(SHORTCUT_CDN.to_string()));
    }

    #[test]
    fn test_apply_shortcuts_all() {
        let mut s = Service {
            query: "test".to_string(),
            start: 0,
            size: 10,
            ignore_cache: false,
            latest: false,
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        Quake::apply_shortcuts(&mut s, 1, 1, 1, 1);
        assert_eq!(s.shortcuts.len(), 4);
    }

    #[test]
    fn test_apply_shortcuts_on_scroll() {
        let mut s = Scroll {
            query: "test".to_string(),
            size: 10,
            ignore_cache: false,
            latest: false,
            pagination_id: String::new(),
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        Quake::apply_shortcuts(&mut s, 0, 1, 0, 1);
        assert_eq!(s.shortcuts.len(), 2);
        assert_eq!(s.shortcuts[0], Value::String(SHORTCUT_MG.to_string()));
        assert_eq!(s.shortcuts[1], Value::String(SHORTCUT_SJQC.to_string()));
    }

    // ========== apply_time_range 测试 ==========

    #[test]
    fn test_apply_time_range_both_empty() {
        let mut start = String::new();
        let mut end = String::new();
        Quake::apply_time_range(&mut start, &mut end, "", "");
        // 当两个参数都为空时，应设置为默认时间范围
        assert!(!start.is_empty());
        assert!(!end.is_empty());
    }

    #[test]
    fn test_apply_time_range_start_only() {
        let mut start = String::new();
        let mut end = String::new();
        Quake::apply_time_range(&mut start, &mut end, "2023-06-01", "");
        assert_eq!(start, "2023-06-01");
        assert!(!end.is_empty()); // end 应为当前时间
    }

    #[test]
    fn test_apply_time_range_end_only() {
        let mut start = String::new();
        let mut end = String::new();
        Quake::apply_time_range(&mut start, &mut end, "", "2024-01-01");
        assert!(!start.is_empty()); // start 应为 end 一年前
        assert_eq!(end, "2024-01-01");
    }

    #[test]
    fn test_apply_time_range_both_set() {
        let mut start = String::new();
        let mut end = String::new();
        Quake::apply_time_range(&mut start, &mut end, "2023-01-01", "2024-01-01");
        assert_eq!(start, "2023-01-01");
        assert_eq!(end, "2024-01-01");
    }

    // ========== init_scroll 测试 ==========

    #[test]
    fn test_init_scroll_basic() {
        let s = Quake::init_scroll("port:80", 10, "", "", 0, 0, 0, 0, 0, "");
        assert_eq!(s.query, "port:80");
        assert_eq!(s.size, 10);
        assert!(s.ignore_cache);
        assert!(s.pagination_id.is_empty());
    }

    #[test]
    fn test_init_scroll_with_pagination() {
        let s = Quake::init_scroll("test", 5, "", "", 0, 0, 0, 0, 0, "page3");
        assert_eq!(s.pagination_id, "page3");
    }

    #[test]
    fn test_init_scroll_with_shortcuts_and_time() {
        let s = Quake::init_scroll(
            "port:443",
            20,
            "2023-01-01",
            "2024-01-01",
            1,
            1,
            0,
            0,
            1,
            "",
        );
        assert_eq!(s.shortcuts.len(), 3);
        assert_eq!(s.start_time, "2023-01-01");
        assert_eq!(s.end_time, "2024-01-01");
    }

    // ========== Shortcutable trait 测试 ==========

    #[test]
    fn test_shortcutable_trait_service() {
        let mut s = Service {
            query: "test".to_string(),
            start: 0,
            size: 10,
            ignore_cache: false,
            latest: false,
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        s.add_shortcut("test_id");
        assert_eq!(s.shortcuts.len(), 1);
        assert_eq!(s.shortcuts[0], Value::String("test_id".to_string()));
    }

    #[test]
    fn test_shortcutable_trait_scroll() {
        let mut s = Scroll {
            query: "test".to_string(),
            size: 10,
            ignore_cache: false,
            latest: false,
            pagination_id: String::new(),
            start_time: String::new(),
            end_time: String::new(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        s.add_shortcut("test_id");
        assert_eq!(s.shortcuts.len(), 1);
        assert_eq!(s.shortcuts[0], Value::String("test_id".to_string()));
    }
}
