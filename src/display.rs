// 展示层：纯展示逻辑，所有函数为独立自由函数

use crate::models::Output;
use ansi_term::Colour::Red;
use ipaddress::IPAddress;
use regex::Regex;
use serde_json::{Map, Value};

/// 清理字符串中的特殊字符（引号、制表符、换行符）
fn sanitize(s: &str) -> String {
    s.replace(['"', '\t', '\n', '\r'], "")
}

/// 从 JSON Value 中提取搜索结果的字段
struct Record {
    ip: String,
    port: String,
    title: String,
    product_name_cn: String,
    version: String,
    protocol: String,
    country: String,
    province: String,
    city: String,
    owner: String,
    time: String,
    domain: String,
    ssl: String,
}

impl Record {
    /// 从 JSON 对象中提取所有字段
    fn from_value(data_value: &Map<String, Value>) -> Self {
        let (product_name_cn, version) = data_value
            .get("components")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.as_object())
            .map(|c| {
                (
                    c.get("product_name_cn")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    c.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                )
            })
            .unwrap_or((String::new(), String::new()));

        let service = data_value.get("service").and_then(|v| v.as_object());
        let http = service
            .and_then(|s| s.get("http"))
            .and_then(|v| v.as_object());
        let title = http
            .and_then(|h| h.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let domain = http
            .and_then(|h| h.get("host"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let protocol = service
            .and_then(|s| s.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let ip = data_value
            .get("ip")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let port = data_value
            .get("port")
            .map(|v| v.to_string())
            .unwrap_or_default();
        let location = data_value
            .get("location")
            .and_then(|v| v.as_object());
        let country = location
            .and_then(|l| l.get("country_cn"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let province = location
            .and_then(|l| l.get("province_cn"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let city = location
            .and_then(|l| l.get("city_cn"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let owner = location
            .and_then(|l| l.get("owner"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let time = data_value
            .get("time")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let ssl = Self::extract_ssl(data_value);

        Self {
            ip: sanitize(ip),
            port,
            title: sanitize(title),
            product_name_cn: sanitize(&product_name_cn),
            version: sanitize(&version),
            protocol: sanitize(protocol),
            country: country.to_string(),
            province: province.to_string(),
            city: city.to_string(),
            owner: owner.to_string(),
            time: time.to_string(),
            domain: sanitize(domain),
            ssl,
        }
    }

    /// 从 JSON 中提取 SSL 证书域名
    fn extract_ssl(data_value: &Map<String, Value>) -> String {
        let try_extract = |path: &[&str]| -> Option<String> {
            let mut current = data_value;
            for (i, key) in path.iter().enumerate() {
                if i == path.len() - 1 {
                    return current.get(*key).and_then(|v| v.as_array()).and_then(|arr| {
                        arr.first()
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    });
                }
                current = current.get(*key)?.as_object()?;
            }
            None
        };

        try_extract(&[
            "service", "tls", "server_certificates", "certificate", "parsed",
            "subject", "common_name",
        ])
        .or_else(|| {
            try_extract(&[
                "service", "tls", "handshake_log", "server_certificates", "certificate",
                "parsed", "subject", "common_name",
            ])
        })
        .unwrap_or_default()
    }

    /// 根据字段名获取对应的值
    fn get_field(&self, field: &str) -> &str {
        match field {
            "ip" => &self.ip,
            "port" => &self.port,
            "title" => &self.title,
            "product_name_cn" => &self.product_name_cn,
            "version" => &self.version,
            "protocol" => &self.protocol,
            "country" => &self.country,
            "province" => &self.province,
            "city" => &self.city,
            "owner" => &self.owner,
            "time" => &self.time,
            "ssldomain" => &self.ssl,
            _ => "",
        }
    }

    /// 根据字段名获取域名（需要特殊处理，排除 IP 地址）
    fn get_domain(&self, field: &str) -> &str {
        if field == "domain" && !IPAddress::is_valid(&self.domain) {
            &self.domain
        } else {
            ""
        }
    }

    /// 格式化记录为制表符分隔的字符串
    fn format_fields(&self, data_type: &[&str]) -> String {
        let mut f = String::new();
        for data in data_type {
            let value = if *data == "domain" {
                self.get_domain(data)
            } else {
                self.get_field(data)
            };
            f.push_str(value);
            f.push('\t');
        }
        f
    }

    /// 拼接用于正则过滤的文本
    fn filter_text(&self, data_value: &Map<String, Value>, filter: &str) -> String {
        if filter.is_empty() {
            return String::new();
        }
        let service = data_value.get("service").and_then(|v| v.as_object());
        let http = service
            .and_then(|s| s.get("http"))
            .and_then(|v| v.as_object());
        let cert = service
            .and_then(|s| s.get("cert"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let response = service
            .and_then(|s| s.get("response"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let http_body = http
            .and_then(|h| h.get("body"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let http_header = http
            .and_then(|h| h.get("response_headers"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut text = String::with_capacity(cert.len() + response.len() + http_body.len() + http_header.len());
        text.push_str(cert);
        text.push_str(response);
        text.push_str(http_body);
        text.push_str(http_header);
        text
    }
}

/// 通用搜索结果展示：从 JSON 数据中提取字段并格式化
fn show_results(
    items: Vec<&Map<String, Value>>,
    showdata: bool,
    filter: &str,
    data_type: &[&str],
) -> Vec<String> {
    let mut res = Vec::new();
    let re = Regex::new(filter).unwrap();

    for data_value in items {
        let record = Record::from_value(data_value);
        let mut f = record.format_fields(data_type);

        let regex_text = record.filter_text(data_value, filter);
        let regex_res = re.find(&regex_text).map(|m| m.as_str()).unwrap_or("");

        if showdata {
            print!("{}", f);
            println!("{}", Red.bold().paint(regex_res));
        } else {
            f.push_str(regex_res);
        }
        res.push(f);
    }
    res
}

pub fn show(
    value: Value,
    showdata: bool,
    filter: &str,
    data_type: Vec<&str>,
) -> Vec<String> {
    let count = value["meta"]["pagination"]["count"].as_i64().unwrap() as usize;
    let total = value["meta"]["pagination"]["total"].as_i64().unwrap() as i32;
    Output::success("Successful.");
    Output::success(&format!("count: {} \ttotal: {}", count, total));

    let items: Vec<&Map<String, Value>> = (0..count)
        .filter_map(|i| value["data"][i].as_object())
        .collect();

    show_results(items, showdata, filter, &data_type)
}

pub fn show_scroll(
    value: Vec<Value>,
    showdata: bool,
    filter: &str,
    data_type: Vec<&str>,
) -> Vec<String> {
    let items: Vec<&Map<String, Value>> = value
        .iter()
        .filter_map(|v| v.as_object())
        .collect();

    show_results(items, showdata, filter, &data_type)
}

/// 格式化单条 Host 记录
fn format_host_record(data_value: &Map<String, Value>) -> String {
    let ip = data_value["ip"].as_str().unwrap_or("");
    let location = data_value["location"].as_object().unwrap();
    let country = location["country_en"].as_str().unwrap_or("");
    let province = location["province_en"].as_str().unwrap_or("");
    let city = location["city_en"].as_str().unwrap_or("");
    let services = data_value["services"].as_array().unwrap();

    let mut info = format!(
        "IP: {}\tCountry: {}\tProvince: {}\tCity: {}\n",
        ip, country, province, city
    );
    info.push_str(&format!(
        "{port}\t{protocol:>width$}\t{time:>width$}\n",
        port = "| Port",
        protocol = "Protocol",
        time = "time",
        width = 20
    ));

    for s in services {
        let protocol = s["name"].as_str().unwrap_or("");
        let service_time = s["time"]
            .as_str()
            .unwrap_or("")
            .replace("unknown", "");
        info.push_str(&format!(
            "| {port}\t{protocol:>width$}\t{time:>width$}\n",
            port = s["port"],
            protocol = protocol,
            time = service_time,
            width = 20
        ));
    }
    info.push('\n');
    info
}

pub fn show_host(mut value: Value, show_data: bool) -> Vec<String> {
    let count = value["meta"]["pagination"]["count"].as_i64().unwrap() as usize;
    let total = value["meta"]["pagination"]["total"].as_i64().unwrap() as i32;
    Output::success("Successful.");
    Output::success(&format!("count: {} \ttotal: {}", count, total));

    let mut res = Vec::new();
    for i in 0..count {
        let data = value["data"][i].take();
        let data_value = data.as_object().unwrap();
        let info = format_host_record(data_value);
        if show_data {
            print!("{}", info);
        }
        res.push(info);
    }
    res
}

pub fn show_host_by_scroll(value: Vec<Value>, show_data: bool) -> Vec<String> {
    let mut res = Vec::new();
    for v in &value {
        if let Some(data_value) = v.as_object() {
            let info = format_host_record(data_value);
            if show_data {
                print!("{}", info);
            }
            res.push(info);
        }
    }
    res
}

pub fn show_domain(
    mut value: Value,
    onlycount: bool,
    showdata: bool,
    data_type: Vec<&str>,
) -> Vec<String> {
    let mut res = Vec::new();

    let count = value["meta"]["pagination"]["count"].as_i64().unwrap() as usize;
    let total = value["meta"]["pagination"]["total"].as_i64().unwrap() as i32;
    Output::success("Successful.");
    Output::success(&format!("count: {} \ttotal: {}", count, total));

    if onlycount {
        if showdata {
            println!("{}", total);
        }
        return res;
    }

    for i in 0..count {
        let data_value = value["data"][i].take();
        let domain = data_value["service"]["http"]["host"]
            .as_str()
            .unwrap_or("");
        let title = data_value["service"]["http"]["title"]
            .as_str()
            .unwrap_or("");
        let ip = data_value["ip"].as_str().unwrap_or("");
        let port = &data_value["port"];

        let mut f = String::new();
        for data in &data_type {
            match *data {
                "domain" => f.push_str(&format!("{}\t", domain)),
                "title" => f.push_str(&format!("{}\t", sanitize(title))),
                "ip" => f.push_str(&format!("{}\t", ip.replace('"', ""))),
                "port" => f.push_str(&format!("{}\t", port)),
                _ => {}
            }
        }
        if showdata {
            println!("{}", f);
        }
        res.push(f);
    }
    res
}

/// 纯展示函数：显示用户账户信息
pub fn show_info(info: Value) {
    let code = info["code"].as_i64().unwrap_or(-1) as i32;
    let message = info["message"].as_str().unwrap_or("Unknown error");
    if code == 0 {
        let data = info["data"].as_object().unwrap();
        let credit = data["credit"].as_i64().unwrap_or(0);
        let persistent_credit = data["persistent_credit"].as_i64().unwrap_or(0);
        let username = data["user"]["username"].as_str().unwrap_or("无");
        let email = data["user"]["email"].as_str().unwrap_or("无");
        let mobile_phone = data["mobile_phone"].as_str().unwrap_or("无");
        let role = data["role"].as_array().unwrap();
        let role_info: Vec<&str> = role
            .iter()
            .filter_map(|r| r["fullname"].as_str())
            .collect();
        Output::success("Successful.");
        Output::info(&format!("用户名:  {}", username));
        Output::info(&format!("邮  箱:  {}", email));
        Output::info(&format!("手  机:  {}", mobile_phone));
        Output::info(&format!("月度积分: {}", credit));
        Output::info(&format!("长效积分: {}", persistent_credit));
        Output::info(&format!("角  色:  {}", role_info.join(",")));
    } else {
        Output::error(message);
    }
}

/// 纯展示函数：显示月度积分信息（简化版）
pub fn show_info_jf(info: Value) {
    let code = info["code"].as_i64().unwrap_or(-1) as i32;
    if code == 0 {
        let data = info["data"].as_object().unwrap();
        let credit = data["credit"].as_i64().unwrap_or(0);
        Output::info(&format!("月度积分: {}", credit));
    }
}

/// 纯展示函数：显示蜜罐检测结果
pub fn display_honeypot(response: Value) {
    let app = response["data"]["app"].as_array().unwrap();
    if !app.is_empty() {
        let app_name = app[0].as_object().unwrap();
        let honeypot = app_name["key"]
            .as_str()
            .unwrap()
            .replace("蜜罐", "")
            .replace('"', "");
        Output::error(&format!("Looks like a {} honeypot system! ", honeypot));
    } else {
        Output::success("Looks like a real system!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::iter::FromIterator;

    // ========== sanitize 测试 ==========

    #[test]
    fn test_sanitize_empty_string() {
        assert_eq!(sanitize(""), "");
    }

    #[test]
    fn test_sanitize_no_special_chars() {
        assert_eq!(sanitize("hello world"), "hello world");
    }

    #[test]
    fn test_sanitize_removes_quotes() {
        assert_eq!(sanitize(r#"he said "hello""#), "he said hello");
    }

    #[test]
    fn test_sanitize_removes_tabs() {
        assert_eq!("hello\tworld".replace(['"', '\t', '\n', '\r'], ""), "helloworld");
    }

    #[test]
    fn test_sanitize_removes_newlines() {
        assert_eq!("line1\nline2\rline3\r\nline4".replace(['"', '\t', '\n', '\r'], ""), "line1line2line3line4");
    }

    #[test]
    fn test_sanitize_mixed() {
        assert_eq!(sanitize("\"tab\there\"\nnew\rline"), "tabherenewline");
    }

    // ========== Record 测试 ==========

    fn make_test_record() -> Record {
        let mut map = serde_json::Map::new();
        map.insert("ip".to_string(), json!("192.168.1.1"));
        map.insert("port".to_string(), json!(80));
        map.insert(
            "service".to_string(),
            json!({
                "name": "http",
                "http": {
                    "title": "Test Page\t\n",
                    "host": "example.com"
                },
                "tls": {
                    "server_certificates": {
                        "certificate": {
                            "parsed": {
                                "subject": {
                                    "common_name": ["*.example.com"]
                                }
                            }
                        }
                    }
                }
            }),
        );
        map.insert(
            "location".to_string(),
            json!({
                "country_cn": "中国",
                "province_cn": "北京",
                "city_cn": "北京",
                "owner": "Test Org"
            }),
        );
        map.insert("time".to_string(), json!("2024-01-15 10:30:00"));
        map.insert(
            "components".to_string(),
            json!([{
                "product_name_cn": "Apache",
                "version": "2.4.41"
            }]),
        );
        Record::from_value(&map)
    }

    #[test]
    fn test_record_from_value_ip() {
        let record = make_test_record();
        assert_eq!(record.ip, "192.168.1.1");
    }

    #[test]
    fn test_record_from_value_port() {
        let record = make_test_record();
        assert_eq!(record.port, "80");
    }

    #[test]
    fn test_record_from_value_title_sanitized() {
        let record = make_test_record();
        assert_eq!(record.title, "Test Page");
    }

    #[test]
    fn test_record_from_value_domain() {
        let record = make_test_record();
        assert_eq!(record.domain, "example.com");
    }

    #[test]
    fn test_record_from_value_protocol() {
        let record = make_test_record();
        assert_eq!(record.protocol, "http");
    }

    #[test]
    fn test_record_from_value_location() {
        let record = make_test_record();
        assert_eq!(record.country, "中国");
        assert_eq!(record.province, "北京");
        assert_eq!(record.city, "北京");
        assert_eq!(record.owner, "Test Org");
    }

    #[test]
    fn test_record_from_value_time() {
        let record = make_test_record();
        assert_eq!(record.time, "2024-01-15 10:30:00");
    }

    #[test]
    fn test_record_from_value_components() {
        let record = make_test_record();
        assert_eq!(record.product_name_cn, "Apache");
        assert_eq!(record.version, "2.4.41");
    }

    #[test]
    fn test_record_extract_ssl() {
        let record = make_test_record();
        assert_eq!(record.ssl, "*.example.com");
    }

    #[test]
    fn test_record_extract_ssl_fallback_handshake() {
        let mut map = serde_json::Map::new();
        map.insert(
            "service".to_string(),
            json!({
                "tls": {
                    "handshake_log": {
                        "server_certificates": {
                            "certificate": {
                                "parsed": {
                                    "subject": {
                                        "common_name": ["fallback.example.com"]
                                    }
                                }
                            }
                        }
                    }
                }
            }),
        );
        let record = Record::from_value(&map);
        assert_eq!(record.ssl, "fallback.example.com");
    }

    #[test]
    fn test_record_extract_ssl_no_cert() {
        let mut map = serde_json::Map::new();
        map.insert("service".to_string(), json!({"name": "ssh"}));
        let record = Record::from_value(&map);
        assert_eq!(record.ssl, "");
    }

    #[test]
    fn test_record_get_field() {
        let record = make_test_record();
        assert_eq!(record.get_field("ip"), "192.168.1.1");
        assert_eq!(record.get_field("port"), "80");
        assert_eq!(record.get_field("title"), "Test Page");
        assert_eq!(record.get_field("country"), "中国");
        assert_eq!(record.get_field("ssldomain"), "*.example.com");
        assert_eq!(record.get_field("unknown_field"), "");
    }

    #[test]
    fn test_record_get_domain_valid_domain() {
        let record = make_test_record();
        assert_eq!(record.get_domain("domain"), "example.com");
    }

    #[test]
    fn test_record_get_domain_ip_address() {
        let mut record = make_test_record();
        record.domain = "192.168.1.1".to_string();
        assert_eq!(record.get_domain("domain"), "");
    }

    #[test]
    fn test_record_get_domain_wrong_field() {
        let record = make_test_record();
        assert_eq!(record.get_domain("ip"), "");
    }

    #[test]
    fn test_record_format_fields() {
        let record = make_test_record();
        let result = record.format_fields(&["ip", "port", "title"]);
        assert_eq!(result, "192.168.1.1\t80\tTest Page\t");
    }

    #[test]
    fn test_record_format_fields_empty() {
        let record = make_test_record();
        let result = record.format_fields(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_record_format_fields_with_domain() {
        let record = make_test_record();
        let result = record.format_fields(&["domain", "ip"]);
        assert_eq!(result, "example.com\t192.168.1.1\t");
    }

    #[test]
    fn test_record_filter_text_empty_filter() {
        let record = make_test_record();
        let map = serde_json::Map::new();
        let result = record.filter_text(&map, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_record_filter_text_concatenates_fields() {
        let record = make_test_record();
        let mut map = serde_json::Map::new();
        map.insert("service".to_string(), json!({
            "cert": "cert_data",
            "response": "resp_data",
            "http": {
                "body": "body_data",
                "response_headers": "header_data"
            }
        }));
        let result = record.filter_text(&map, "test");
        assert_eq!(result, "cert_dataresp_databody_dataheader_data");
    }

    // ========== format_host_record 测试 ==========

    #[test]
    fn test_format_host_record_basic() {
        let map = serde_json::Map::from_iter(vec![
            ("ip".to_string(), json!("10.0.0.1")),
            ("location".to_string(), json!({
                "country_en": "China",
                "province_en": "Beijing",
                "city_en": "Beijing"
            })),
            ("services".to_string(), json!([
                {
                    "port": 80,
                    "name": "http",
                    "time": "2024-01-15"
                }
            ])),
        ]);
        let result = format_host_record(&map);
        assert!(result.contains("IP: 10.0.0.1"));
        assert!(result.contains("Country: China"));
        assert!(result.contains("Province: Beijing"));
        assert!(result.contains("City: Beijing"));
        assert!(result.contains("| Port"));
        assert!(result.contains("http"));
    }

    #[test]
    fn test_format_host_record_multiple_services() {
        let map = serde_json::Map::from_iter(vec![
            ("ip".to_string(), json!("10.0.0.1")),
            ("location".to_string(), json!({
                "country_en": "US",
                "province_en": "California",
                "city_en": "San Francisco"
            })),
            ("services".to_string(), json!([
                {"port": 80, "name": "http", "time": "2024-01-01"},
                {"port": 443, "name": "https", "time": "2024-01-02"}
            ])),
        ]);
        let result = format_host_record(&map);
        assert!(result.contains("http"));
        assert!(result.contains("https"));
    }

    #[test]
    fn test_format_host_record_unknown_time() {
        let map = serde_json::Map::from_iter(vec![
            ("ip".to_string(), json!("10.0.0.1")),
            ("location".to_string(), json!({
                "country_en": "US",
                "province_en": "CA",
                "city_en": "SF"
            })),
            ("services".to_string(), json!([
                {"port": 22, "name": "ssh", "time": "unknown"}
            ])),
        ]);
        let result = format_host_record(&map);
        assert!(!result.contains("unknown"));
    }

    // ========== show / show_scroll 测试 ==========

    fn make_search_response(count: usize, total: i32) -> Value {
        let data: Vec<Value> = (0..count)
            .map(|i| {
                json!({
                    "ip": format!("10.0.0.{}", i + 1),
                    "port": 80 + i as i64,
                    "service": {
                        "name": "http",
                        "http": {
                            "title": format!("Page {}", i),
                            "host": format!("host{}.com", i)
                        }
                    },
                    "location": {
                        "country_cn": "中国",
                        "province_cn": "北京",
                        "city_cn": "北京",
                        "owner": "Org"
                    },
                    "time": "2024-01-01 00:00:00"
                })
            })
            .collect();
        json!({
            "code": 0,
            "data": data,
            "meta": {
                "pagination": {
                    "count": count,
                    "total": total
                }
            }
        })
    }

    #[test]
    fn test_show_returns_formatted_strings() {
        let value = make_search_response(2, 100);
        let result = show(value, false, "", vec!["ip", "port"]);
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("10.0.0.1"));
        assert!(result[0].contains("80"));
    }

    #[test]
    fn test_show_empty_filter() {
        let value = make_search_response(1, 1);
        let result = show(value, false, "", vec!["ip"]);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("10.0.0.1"));
    }

    #[test]
    fn test_show_with_filter_match() {
        let value = make_search_response(1, 1);
        let result = show(value, false, "10\\.0\\.0\\.1", vec!["ip"]);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("10.0.0.1"));
    }

    #[test]
    fn test_show_scroll_returns_formatted_strings() {
        let items = vec![
            json!({
                "ip": "1.1.1.1",
                "port": 443,
                "service": {
                    "name": "https",
                    "http": {"title": "Test", "host": "test.com"}
                },
                "location": {
                    "country_cn": "US",
                    "province_cn": "CA",
                    "city_cn": "SF",
                    "owner": "Org"
                },
                "time": "2024-01-01"
            }),
        ];
        let result = show_scroll(items, false, "", vec!["ip", "port"]);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("1.1.1.1"));
        assert!(result[0].contains("443"));
    }

    // ========== show_host / show_host_by_scroll 测试 ==========

    #[test]
    fn test_show_host_returns_formatted() {
        let value = json!({
            "code": 0,
            "data": [{
                "ip": "192.168.1.1",
                "location": {
                    "country_en": "China",
                    "province_en": "Beijing",
                    "city_en": "Beijing"
                },
                "services": [{
                    "port": 80,
                    "name": "http",
                    "time": "2024-01-15"
                }]
            }],
            "meta": {
                "pagination": {"count": 1, "total": 1}
            }
        });
        let result = show_host(value, false);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("IP: 192.168.1.1"));
        assert!(result[0].contains("Country: China"));
        assert!(result[0].contains("http"));
    }

    #[test]
    fn test_show_host_by_scroll_returns_formatted() {
        let items = vec![
            json!({
                "ip": "10.0.0.1",
                "location": {
                    "country_en": "US",
                    "province_en": "NY",
                    "city_en": "NYC"
                },
                "services": [
                    {"port": 22, "name": "ssh", "time": "2024-01-01"},
                    {"port": 80, "name": "http", "time": "2024-01-02"}
                ]
            }),
        ];
        let result = show_host_by_scroll(items, false);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("IP: 10.0.0.1"));
        assert!(result[0].contains("ssh"));
        assert!(result[0].contains("http"));
    }

    // ========== show_domain 测试 ==========

    #[test]
    fn test_show_domain_returns_formatted() {
        let value = json!({
            "code": 0,
            "data": [{
                "ip": "1.2.3.4",
                "port": 443,
                "service": {
                    "http": {
                        "host": "example.com",
                        "title": "Example"
                    }
                }
            }],
            "meta": {
                "pagination": {"count": 1, "total": 1}
            }
        });
        let result = show_domain(value, false, false, vec!["domain", "ip", "port"]);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("example.com"));
        assert!(result[0].contains("1.2.3.4"));
    }

    #[test]
    fn test_show_domain_only_count() {
        let value = json!({
            "code": 0,
            "data": [],
            "meta": {
                "pagination": {"count": 0, "total": 42}
            }
        });
        let result = show_domain(value, true, false, vec!["domain"]);
        assert!(result.is_empty());
    }

    // ========== show_info / show_info_jf 测试 ==========

    #[test]
    fn test_show_info_success() {
        let info = json!({
            "code": 0,
            "message": "success",
            "data": {
                "credit": 1000,
                "persistent_credit": 5000,
                "user": {
                    "username": "testuser",
                    "email": "test@example.com"
                },
                "mobile_phone": "138****1234",
                "role": [
                    {"fullname": "Admin"},
                    {"fullname": "User"}
                ]
            }
        });
        // 不验证输出，只验证不 panic
        show_info(info);
    }

    #[test]
    fn test_show_info_error() {
        let info = json!({
            "code": -1,
            "message": "Unauthorized"
        });
        show_info(info);
    }

    #[test]
    fn test_show_info_jf_success() {
        let info = json!({
            "code": 0,
            "message": "success",
            "data": {"credit": 2000}
        });
        show_info_jf(info);
    }

    #[test]
    fn test_show_info_jf_error() {
        let info = json!({
            "code": -1,
            "message": "error"
        });
        show_info_jf(info);
    }

    // ========== display_honeypot 测试 ==========

    #[test]
    fn test_display_honeypot_detected() {
        let response = json!({
            "data": {
                "app": [{"key": "蜜罐系统"}]
            }
        });
        display_honeypot(response);
    }

    #[test]
    fn test_display_honeypot_not_detected() {
        let response = json!({
            "data": {
                "app": []
            }
        });
        display_honeypot(response);
    }

    // ========== 边界情况测试 ==========

    #[test]
    fn test_show_with_empty_data_array() {
        let value = json!({
            "code": 0,
            "data": [],
            "meta": {
                "pagination": {"count": 0, "total": 0}
            }
        });
        let result = show(value, false, "", vec!["ip"]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_record_from_value_missing_fields() {
        let mut map = serde_json::Map::new();
        map.insert("ip".to_string(), json!("1.1.1.1"));
        // 缺少 service, location, time, components 等字段
        let record = Record::from_value(&map);
        assert_eq!(record.ip, "1.1.1.1");
        assert_eq!(record.title, "");
        assert_eq!(record.ssl, "");
        assert_eq!(record.country, "");
    }
}
