// 展示层：纯展示逻辑，所有函数为独立自由函数

use crate::models::Output;
use ansi_term::Colour::Red;
use ipaddress::IPAddress;
use regex::Regex;
use serde_json::{Map, Value};

pub fn show(
    value: Value,
    showdata: bool,
    filter: &str,
    mut data_type: Vec<&str>,
) -> Vec<String> {
    let count = value["meta"]["pagination"]["count"].as_i64().unwrap() as usize;
    let total = value["meta"]["pagination"]["total"].as_i64().unwrap() as i32;
    let mut res: Vec<String> = Vec::new();
    Output::success("Successful.");
    Output::success(&format!("count: {} \ttotal: {}", count, total));
    let re = Regex::new(filter).unwrap();
    for i in 0..count {
        let data_value = value["data"][i].as_object().unwrap();
        let mut product_name_cn="".to_string();
        let mut version="".to_string();
        let key = "components";
        if data_value.contains_key(key) {
            product_name_cn = data_value["components"][0]["product_name_cn"]
            .as_str()
            .unwrap_or("")
            .replace("\"", "")
            .replace("\t", "")
            .replace("\n", "")
            .replace("\r", "");
            version = data_value["components"][0]["version"]
            .as_str()
            .unwrap_or("")
            .replace("\"", "")
            .replace("\t", "")
            .replace("\n", "")
            .replace("\r", "");
        }

        let title = data_value["service"]["http"]["title"]
            .as_str()
            .unwrap_or("")
            .replace("\"", "")
            .replace("\t", "")
            .replace("\n", "")
            .replace("\r", "");
        let domain = data_value["service"]["http"]["host"]
            .as_str()
            .unwrap_or("")
            .replace("\"", "")
            .replace("\t", "")
            .replace("\n", "")
            .replace("\r", "");
        let name = data_value["service"]["name"]
            .as_str()
            .unwrap_or("")
            .replace("\"", "")
            .replace("\t", "")
            .replace("\n", "")
            .replace("\r", "");
        let ip = data_value["ip"].as_str().unwrap().replace("\"", "");
        let port = &data_value["port"];
        let country = data_value["location"]["country_cn"].as_str().unwrap_or("");
        let province = data_value["location"]["province_cn"].as_str().unwrap_or("");
        let city = data_value["location"]["city_cn"].as_str().unwrap_or("");
        let owner = data_value["location"]["owner"].as_str().unwrap_or("");
        let time = data_value["time"].as_str().unwrap_or("");
        let ssl: &str = match data_value["service"]["tls"]["server_certificates"]
            ["certificate"]["parsed"]["subject"]["common_name"]
            .as_array()
        {
            Some(ssl) => ssl[0].as_str().unwrap_or(""),
            None => match data_value["service"]["tls"]["handshake_log"]
                ["server_certificates"]["certificate"]["parsed"]["subject"]["common_name"]
                .as_array()
            {
                Some(ssl) => ssl[0].as_str().unwrap_or(""),
                None => "",
            },
        };
        let mut regex_data = String::new();
        if !filter.is_empty() {
            let cert = data_value["service"]["cert"].as_str().unwrap_or("");
            let response = data_value["service"]["response"].as_str().unwrap_or("");
            let http_body = data_value["service"]["http"]["body"].as_str().unwrap_or("");
            let http_header = data_value["service"]["http"]["response_headers"]
                .as_str()
                .unwrap_or("");
            regex_data.push_str(cert);
            regex_data.push_str(response);
            regex_data.push_str(http_body);
            regex_data.push_str(http_header);
        }
        let regex_res = match re.find(regex_data.as_str()) {
            Some(res) => res.as_str(),
            None => "",
        };
        let mut f: String = String::new();
        for data in data_type.iter_mut() {
            if data == &"title" {
                f.push_str(&format!("{}\t", title));
            }
            if data == &"product_name_cn" {
                f.push_str(&format!("{}\t", product_name_cn));
            }
            if data == &"version" {
                f.push_str(&format!("{}\t", version));
            }
            if data == &"protocol" {
                f.push_str(&format!("{}\t", name));
            }
            if data == &"ip" {
                f.push_str(&format!("{}\t", ip));
            }
            if data == &"port" {
                f.push_str(&format!("{}\t", port));
            }
            if data == &"country" {
                f.push_str(&format!("{}\t", country));
            }
            if data == &"province" {
                f.push_str(&format!("{}\t", province));
            }
            if data == &"city" {
                f.push_str(&format!("{}\t", city));
            }
            if data == &"owner" {
                f.push_str(&format!("{}\t", owner));
            }
            if data == &"time" {
                f.push_str(&format!("{}\t", time));
            }
            if data == &"domain" {
                if !IPAddress::is_valid(domain.clone()) {
                    f.push_str(&format!("{}\t", domain));
                } else {
                    f.push_str(&format!("{}\t", ""));
                }
            }
            if data == &"ssldomain" {
                f.push_str(&format!("{}\t", ssl))
            }
        }
        if showdata {
            print!("{}", f);
            println!("{}", Red.bold().paint(regex_res).to_string().as_str());
        } else {
            f.push_str(regex_res);
        }
        res.push(f);
    }
    res
}

    #[allow(clippy::needless_range_loop)]
pub fn show_scroll(
    value: Vec<Value>,
    showdata: bool,
    filter: &str,
    mut data_type: Vec<&str>,
) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    let re = Regex::new(filter).unwrap();
    let count = value.len();
    for i in 0..count {
        let data_value: &Map<String, Value> = value[i].as_object().unwrap();
        let title = data_value["service"]["http"]["title"]
            .as_str()
            .unwrap_or("")
            .replace("\"", "")
            .replace("\t", "")
            .replace("\n", "")
            .replace("\r", "");
        let domain = data_value["service"]["http"]["host"]
            .as_str()
            .unwrap_or("")
            .replace("\"", "")
            .replace("\t", "")
            .replace("\n", "")
            .replace("\r", "");
        let ip = data_value["ip"].as_str().unwrap().replace("\"", "");
        let port = &data_value["port"];
        let country = data_value["location"]["country_cn"].as_str().unwrap_or("");
        let province = data_value["location"]["province_cn"].as_str().unwrap_or("");
        let city = data_value["location"]["city_cn"].as_str().unwrap_or("");
        let owner = data_value["location"]["owner"].as_str().unwrap_or("");
        let time = data_value["time"].as_str().unwrap_or("");
        let ssl: &str = match data_value["service"]["tls"]["server_certificates"]
            ["certificate"]["parsed"]["subject"]["common_name"]
            .as_array()
        {
            Some(ssl) => ssl[0].as_str().unwrap_or(""),
            None => match data_value["service"]["tls"]["handshake_log"]
                ["server_certificates"]["certificate"]["parsed"]["subject"]["common_name"]
                .as_array()
            {
                Some(ssl) => ssl[0].as_str().unwrap_or(""),
                None => "",
            },
        };
        let mut regex_data = String::new();
        if !filter.is_empty() {
            let cert = data_value["service"]["cert"].as_str().unwrap_or("");
            let response = data_value["service"]["response"].as_str().unwrap_or("");
            let http_body = data_value["service"]["http"]["body"].as_str().unwrap_or("");
            let http_header = data_value["service"]["http"]["response_headers"]
                .as_str()
                .unwrap_or("");
            regex_data.push_str(cert);
            regex_data.push_str(response);
            regex_data.push_str(http_body);
            regex_data.push_str(http_header);
        }
        let regex_res = match re.find(regex_data.as_str()) {
            Some(res) => res.as_str(),
            None => "",
        };
        let mut f: String = String::new();
        for data in data_type.iter_mut() {
            if data == &"title" {
                f.push_str(&format!("{}\t", title));
            }
            if data == &"ip" {
                f.push_str(&format!("{}\t", ip));
            }
            if data == &"port" {
                f.push_str(&format!("{}\t", port));
            }
            if data == &"country" {
                f.push_str(&format!("{}\t", country));
            }
            if data == &"province" {
                f.push_str(&format!("{}\t", province));
            }
            if data == &"city" {
                f.push_str(&format!("{}\t", city));
            }
            if data == &"owner" {
                f.push_str(&format!("{}\t", owner));
            }
            if data == &"time" {
                f.push_str(&format!("{}\t", time));
            }
            if data == &"domain" {
                if !IPAddress::is_valid(domain.clone()) {
                    f.push_str(&format!("{}\t", domain));
                } else {
                    f.push_str(&format!("{}\t", ""));
                }
            }
            if data == &"ssldomain" {
                f.push_str(&format!("{}\t", ssl))
            }
        }
        if showdata {
            print!("{}", f);
            println!("{}", Red.bold().paint(regex_res).to_string().as_str());
        } else {
            f.push_str(regex_res);
        }
        res.push(f);
    }
    res
}

pub fn show_host(mut value: Value, show_data: bool) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    let count = value["meta"]["pagination"]["count"].as_i64().unwrap() as usize;
    let total = value["meta"]["pagination"]["total"].as_i64().unwrap() as i32;
    Output::success("Successful.");
    Output::success(&format!("count: {} \ttotal: {}", count, total));
    for i in 0..count {
        // ip
        let data = value["data"][i].take();
        let ip = data["ip"].as_str().unwrap().replace("\"", "");
        let location = data["location"].as_object().unwrap();
        let country = location["country_en"].as_str().unwrap_or("");
        let province = location["province_en"].as_str().unwrap_or("");
        let city = location["city_en"].as_str().unwrap_or("");
        let service = data["services"].as_array().unwrap();
        let mut info = String::new();
        info.push_str(&format!(
            "IP: {}\tCountry: {}\tProvince: {}\tCity: {}\n",
            ip, country, province, city
        ));
        info.push_str(&format!(
            "{port}\t{protocol:>width$}\t{time:>width$}\n",
            port = "| Port",
            protocol = "Protocol",
            time = "time",
            width = 20
        ));
        for s in service {
            let protocol = s["name"].as_str().unwrap().replace("\"", "");
            let service_time = s["time"]
                .as_str()
                .unwrap()
                .replace("\"", "")
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
        if show_data {
            println!("{}", info);
        }
        res.push(info);
    }
    res
}

    #[allow(clippy::needless_range_loop)]
pub fn show_host_by_scroll(value: Vec<Value>, show_data: bool) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    let count = value.len();
    for i in 0..count {
        let data_value: &Map<String, Value> = value[i].as_object().unwrap();
        let ip = data_value["ip"].as_str().unwrap().replace("\"", "");
        let location = data_value["location"].as_object().unwrap();
        let country = location["country_en"].as_str().unwrap_or("");
        let province = location["province_en"].as_str().unwrap_or("");
        let city = location["city_en"].as_str().unwrap_or("");
        let service = data_value["services"].as_array().unwrap();
        let mut info = String::new();
        info.push_str(&format!(
            "IP: {}\tCountry: {}\tProvince: {}\tCity: {}\n",
            ip, country, province, city
        ));
        info.push_str(&format!(
            "{port}\t{protocol:>width$}\t{time:>width$}\n",
            port = "| Port",
            protocol = "Protocol",
            time = "time",
            width = 20
        ));
        for s in service {
            let protocol = s["name"].as_str().unwrap().replace("\"", "");
            let service_time = s["time"]
                .as_str()
                .unwrap()
                .replace("\"", "")
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
        if show_data {
            println!("{}", info);
        }
        res.push(info);
    }
    res
}

pub fn show_domain(
    mut value: Value,
    onlycount: bool,
    showdata: bool,
    mut data_type: Vec<&str>,
) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();

    let count = value["meta"]["pagination"]["count"].as_i64().unwrap() as usize;
    let total = value["meta"]["pagination"]["total"].as_i64().unwrap() as i32;
    Output::success("Successful.");
    Output::success(&format!("count: {} \ttotal: {}", count, total));
    if !onlycount {
        for i in 0..count {
            let data_value = value["data"][i].take();
            let domain = data_value["service"]["http"]["host"]
                .as_str()
                .unwrap_or("")
                .replace("\"", "");

            let title = data_value["service"]["http"]["title"]
                .as_str()
                .unwrap_or("")
                .replace("\"", "")
                .replace("\t", "")
                .replace("\n", "")
                .replace("\r", "");
            let ip = data_value["ip"].as_str().unwrap().replace("\"", "");
            let port = &data_value["port"];
            let mut f = String::new();
            for data in data_type.iter_mut() {
                if data == &"domain" {
                    f.push_str(&format!("{}\t", domain));
                }
                if data == &"title" {
                    f.push_str(&format!("{}\t", title));
                }
                if data == &"ip" {
                    f.push_str(&format!("{}\t", ip));
                }
                if data == &"port" {
                    f.push_str(&format!("{}\t", port));
                }
            }
            if showdata {
                println!("{}", f);
            }
            res.push(f);
        }
    } else {
        if showdata {
            println!("{}", total);
        }
    }
    res
}

/// 纯展示函数：显示用户账户信息
/// 参数为已获取的 info 数据（Value 类型）
pub fn show_info(info: Value) {
    let code = info["code"].as_i64().unwrap_or(-1) as i32;
    let message = info["message"].as_str().unwrap();
    let data = info["data"].as_object().unwrap();
    if code == 0 {
        let credit = data["credit"].as_i64().unwrap_or(0);
        let persistent_credit = data["persistent_credit"].as_i64().unwrap_or(0);
        let username = data["user"]["username"].as_str().unwrap_or("无");
        let email = data["user"]["email"].as_str().unwrap_or("无");
        let mobile_phone = data["mobile_phone"].as_str().unwrap_or("无");
        let role = data["role"].as_array().unwrap();
        let mut role_info = String::new();
        Output::success("Successful.");
        Output::info(&format!("用户名:  {}", username));
        Output::info(&format!("邮  箱:  {}", email));
        Output::info(&format!("手  机:  {}", mobile_phone));
        Output::info(&format!("月度积分: {}", credit));
        Output::info(&format!("长效积分: {}", persistent_credit));
        for r in role {
            let r = r["fullname"].as_str().unwrap_or("");
            role_info.push_str(r);
            role_info.push(',');
        }
        role_info.remove(role_info.len() - 1);
        Output::info(&format!("角  色:  {}", role_info));
    } else {
        Output::error(message);
    }
}

/// 纯展示函数：显示月度积分信息（简化版）
/// 参数为已获取的 info 数据（Value 类型）
pub fn show_info_jf(info: Value) {
    let code = info["code"].as_i64().unwrap_or(-1) as i32;
    info["message"].as_str().unwrap();
    let data = info["data"].as_object().unwrap();
    if code == 0 {
        let credit = data["credit"].as_i64().unwrap_or(0);
        Output::info(&format!("月度积分: {}", credit));}
}

/// 纯展示函数：显示蜜罐检测结果
/// 参数为已获取的 aggservice 响应数据（Value 类型）
pub fn display_honeypot(response: Value) {
    let app = response["data"]["app"].as_array().unwrap();
    if !app.is_empty() {
        let app_name = app[0].as_object().unwrap();
        let honeypot = app_name["key"]
            .as_str()
            .unwrap()
            .replace("蜜罐", "")
            .replace("\"", "");
        Output::error(&format!("Looks like a {} honeypot system! ", honeypot));
    } else {
        Output::success("Looks like a real system!");
    }
}
