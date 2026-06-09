// use log::{debug, error, info};
use crate::api::ApiKey;
use crate::models::{AggService, Host, Output, Scroll, ScrollHost, Service};
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::{Map, Number, Value};
use std::fs;

//BaseUrl is the basis for all of our api requests.
const BASE_URL: &str = "https://quake.360.net";
// Removed unused constant to fix the warning
pub struct Quake {
    api_key: String,
}

impl Quake {
    pub fn new(api_key: String) -> Quake {
        Quake { api_key }
    }


    pub fn query_host(query_string: &str, start: i32, size: i32) -> Value {
        Output::info(&format!("Search with {}", query_string));
        let res = ApiKey::get_api().expect("Failed to read apikey:\t");

        let h = Host {
            query: String::from(query_string),
            start,
            size,
            ignore_cache: false,
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
        let mut url = String::new();
        url.push_str(BASE_URL);
        url.push_str("/api/v3/search/quake_host");
        let client = reqwest::blocking::Client::new();
        let resp = match client.post(&url).headers(self.header()).json(&host).send() {
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
        let res = resp.text().unwrap();
        let response: Value = serde_json::from_str(&res)?;

        let code = response["code"].to_string();
        let message = response["message"].as_str().unwrap();
        if code != "0" {
            Output::error(&format!("Query failed: {}", message));
            std::process::exit(1);
        }
        Ok(response)
    }

    pub fn query_host_by_scroll(query_string: &str, size: i32) -> Vec<Value> {
        Output::info(&format!("Search with {}", query_string));
        let res = ApiKey::get_api().expect("Failed to read apikey:\t");
        let response = match Quake::new(res).search_host_by_scroll(query_string, size) {
            Ok(response) => response,
            Err(e) => {
                Output::error(&format!("Query failed: {}", e));
                std::process::exit(1);
            }
        };
        response
    }

    pub fn search_host_by_scroll(
        &self,
        query_string: &str,
        size: i32,
    ) -> Result<Vec<Value>, serde_json::Error> {
        let sh = Self::init_scroll_host(query_string, size, "");
        let res = Self::get_scroll_data_by_host(self, sh);
        let response: Value = serde_json::from_str(&res)?;
        let message = response["message"].as_str().unwrap();
        let code = response["code"].to_string();
        if code != "0" {
            Output::error(&format!("Query failed: {}", message));
            std::process::exit(1);
        }
        let data_array = response["data"].as_array().unwrap();
        let pagination_id = response["meta"]["pagination_id"].as_str().unwrap();
        let mut data_len = data_array.len();
        let mut all_data = Vec::new();

        all_data.extend(data_array.iter().cloned());

        while data_len != 0 && (data_len as i32) >= size {
            let s_scroll = Self::init_scroll_host(query_string, size, pagination_id);
            let res_scroll = Self::get_scroll_data_by_host(self, s_scroll);
            let responses: Value = serde_json::from_str(&res_scroll)?;
            let data_array_for_while = responses["data"].as_array().unwrap();
            // all_data.append(&mut data_array_for_while);
            all_data.extend(data_array_for_while.iter().cloned());
            data_len = data_array_for_while.len();
        }
        Ok(all_data)
    }

    pub fn get_scroll_data_by_host(&self, scrollhost: ScrollHost) -> String {
        let mut url = String::new();
        url.push_str(BASE_URL);
        url.push_str("/api/v3/scroll/quake_host");
        let client = reqwest::blocking::Client::new();
        let post_data: Map<String, Value> = Self::get_scrollhost_post_data(scrollhost);
        let resp: Response = match client
            .post(&url)
            .headers(self.header())
            .json(&post_data)
            .send()
        {
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
        let res = match resp.text() {
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
        res
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
            query: "".to_string(),
            size,
            ignore_cache: false,
            pagination_id: "".to_string(),
        };
        if !query_string.is_empty() {
            sh.query = query_string.to_string();
        } else {
            Output::info("Search failed");
            std::process::exit(1);
        }
        if !pagination_id.is_empty() {
            sh.pagination_id = pagination_id.to_string();
        }
        sh
    }

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
            query: "".to_string(),
            start,
            size,
            ignore_cache: false,
            latest: false,
            start_time: "".to_string(),
            end_time: "".to_string(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        if cdn == 1 {
            s.shortcuts
                .push(Value::String("612f5a5ad6b3bdb87961727f".to_string()));
        }
        if mg == 1 {
            s.shortcuts
                .push(Value::String("610ce2adb1a2e3e1632e67b1".to_string()));
        }
        if zxsj == 1 {
            s.ignore_cache = true;
            s.latest = true;
        }
        if wxqq == 1 {
            s.shortcuts
                .push(Value::String("62bc12b70537d96695680ce5".to_string()));
        }
        if sjqc == 1 {
            s.shortcuts
                .push(Value::String("610ce2fbda6d29df72ac56eb".to_string()));
        }
        let (local, one_years_ago) = crate::models::getdate();
        if time_start.is_empty() && time_end.is_empty() {
            s.start_time = one_years_ago;
            s.end_time = local;
        } else if !time_start.is_empty() && time_end.is_empty() {
            s.start_time = time_start.to_string();
            s.end_time = local;
        } else if time_start.is_empty() && !time_end.is_empty() {
            s.start_time = crate::models::getdate_for_manual(time_end);
            s.end_time = time_end.to_string();
        } else if !time_start.is_empty() && !time_end.is_empty() {
            s.start_time = time_start.to_string();
            s.end_time = time_end.to_string();
        }
        if !file_name.is_empty() {
            let ips: String = match fs::read_to_string(file_name) {
                Ok(res) => res,
                Err(err) => {
                    Output::error(&format!(
                        "Failed to read {} : {}",
                        file_name,
                        err
                    ));
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
        //print!("{:?}", s);
        let response: Value = match Quake::new(res).search(s) {
            Ok(response) => response,
            Err(e) => {
                Output::error(&format!("Query failed: {}", e));
                std::process::exit(1);
            }
        };
        response
    }

    pub fn search(&self, service: Service) -> Result<Value, serde_json::Error> {
        let mut url = String::new();
        url.push_str(BASE_URL);
        url.push_str("/api/v3/search/quake_service");
        let client = reqwest::blocking::Client::new();
        let post_data: Map<String, Value> = Self::get_service_post_data(service);
        //print!("{:?}", post_data);
        //print!("{:?}",self.header());
        let resp: Response = match client
            .post(&url)
            .headers(self.header())
            .json(&post_data)
            .send()
        {
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
        let res = match resp.text() {
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
        //print!("{:?}",res);
        let response: Value = serde_json::from_str(&res)?;
        let code = response["code"].to_string();
        let message = response["message"].as_str().unwrap();
        if code != "0" {
            Output::error(&format!("Query failed: {}", message));
            std::process::exit(1);
        }
        Ok(response)
    }
    // Removed unused method `query_gpt` to fix the warning.
    pub fn get_scroll_data(&self, scroll: Scroll) -> String {
        let mut url = String::new();
        url.push_str(BASE_URL);
        url.push_str("/api/v3/scroll/quake_service");
        let client = reqwest::blocking::Client::new();
        let post_data: Map<String, Value> = Self::get_scroll_post_data(scroll);
        let resp: Response = match client
            .post(&url)
            .headers(self.header())
            .json(&post_data)
            .send()
        {
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
        let res = match resp.text() {
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
        res
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
            query: "".to_string(),
            size,
            ignore_cache: false,
            latest: false,
            pagination_id: "".to_string(),
            start_time: "".to_string(),
            end_time: "".to_string(),
            ip_list: vec![],
            shortcuts: vec![],
        };
        if cdn == 1 {
            s.shortcuts
                .push(Value::String("612f5a5ad6b3bdb87961727f".to_string()));
        }
        if mg == 1 {
            s.shortcuts
                .push(Value::String("610ce2adb1a2e3e1632e67b1".to_string()));
        }
        if zxsj == 1 {
            s.ignore_cache = true;
            s.latest = true;
        }
        if wxqq == 1 {
            s.shortcuts
                .push(Value::String("62bc12b70537d96695680ce5".to_string()));
        }
        if sjqc == 1 {
            s.shortcuts
                .push(Value::String("610ce2fbda6d29df72ac56eb".to_string()));
        }
        let (local, one_years_ago) = crate::models::getdate();
        if time_start.is_empty() && time_end.is_empty() {
            s.start_time = one_years_ago;
            s.end_time = local;
        } else if !time_start.is_empty() && time_end.is_empty() {
            s.start_time = time_start.to_string();
            s.end_time = local;
        } else if time_start.is_empty() && !time_end.is_empty() {
            s.start_time = crate::models::getdate_for_manual(time_end);
            s.end_time = time_end.to_string();
        } else if !time_start.is_empty() && !time_end.is_empty() {
            s.start_time = time_start.to_string();
            s.end_time = time_end.to_string();
        }
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
        let response = match Quake::new(res).scroll(
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
        };
        //println!("{:?}",response);
        response
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
        let res = Self::get_scroll_data(self, scroll);
        let response: Value = serde_json::from_str(&res)?;
        //println!("{:?}",res);
        //println!("{:?}",response["code"]);
        let code = response["code"].to_string();
        let message = response["message"].as_str().unwrap();
        if code != "0" {
            Output::error(&format!("Query failed: {}", message));
            std::process::exit(1);
        }
        let data_array = response["data"].as_array().unwrap();
        let _pagination_id = response["meta"]["pagination_id"].as_str().unwrap();
        let mut _data_len = data_array.len();
        let mut all_data = Vec::new();

        // all_data.append(&mut data_array);
        all_data.extend(data_array.iter().cloned());
        //println!("{:?}",all_data);
        // while data_len != 0 && (data_len as i32) >= size {
        //     let s_scroll = Self::init_scroll(
        //         query_string,
        //         size,
        //         time_start,
        //         time_end,
        //         cdn,
        //         mg,
        //         zxsj,
        //         wxqq,
        //         sjqc,
        //         pagination_id,
        //     );
        //     let res_scroll = Self::get_scroll_data(self, s_scroll);
        //     let responses: Value = serde_json::from_str(&res_scroll)?;
        //     let data_array_for_while = responses["data"].as_array().unwrap();
        //     // all_data.append(&mut data_array_for_while);
        //     all_data.extend(data_array_for_while.iter().cloned());
        //     data_len = data_array_for_while.len();
        // }
        Ok(all_data)
    }

    pub fn aggservice(&self, agg: &AggService) -> Result<Value, serde_json::Error> {
        let mut url = String::new();
        url.push_str(BASE_URL);
        url.push_str("/api/v3/aggregation/quake_service");
        let client = reqwest::blocking::Client::new();
        let resp = match client.post(&url).headers(self.header()).json(&agg).send() {
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
        let res = resp.text().unwrap();
        let response: Value = serde_json::from_str(&res)?;
        let code = response["code"].to_string();
        let message = response["message"].as_str().unwrap();
        if code != "0" {
            Output::error(&format!("Query failed: {}", message));
            std::process::exit(1);
        }
        Ok(response)
    }

    // Interface for obtaining user information
    // https://quake.360.cn/quake/#/help?id=5fdb2a58dd0705216cbaa480&title=%E7%94%A8%E6%88%B7%E4%BF%A1%E6%81%AF%E6%8E%A5%E5%8F%A3
    // URL: https://quake.360.cn/api/v3/user/info
    // Parameters: None
    // Method: GET
    // Return: Result<Value, serde_json::Error>
    pub fn info(&self) -> Result<Value, serde_json::Error> {
        let mut url: String = String::new();
        url.push_str(BASE_URL);
        url.push_str("/api/v3/user/info");
        let clinet = reqwest::blocking::Client::new();
        let resp = match clinet.get(&url).headers(self.header()).send() {
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
        let res = resp.text().unwrap();
        let response: Value = serde_json::from_str(&res)?;

        let code = response["code"].to_string();
        let message = response["message"].as_str().unwrap();
        if code != "0" {
            Output::error(&format!("Query failed: {}", message));
            std::process::exit(1);
        }
        Ok(response)
    }

    fn get_scroll_post_data(s: Scroll) -> Map<String, Value> {
        let mut data: Map<String, Value> = Map::new();
        data.insert("size".to_string(), Value::Number(Number::from(s.size)));
        data.insert("ignore_cache".to_string(), Value::Bool(s.ignore_cache));
        data.insert("latest".to_string(), Value::Bool(s.latest));
        data.insert("start_time".to_string(), Value::String(s.start_time));
        data.insert("end_time".to_string(), Value::String(s.end_time));
        data.insert("shortcuts".to_string(), Value::Array(s.shortcuts));
        if !s.ip_list.is_empty() {
            data.insert("query".to_string(), Value::String("".to_string()));
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
            data.insert("query".to_string(), Value::String("".to_string()));
            data.insert("ip_list".to_string(), Value::Array(s.ip_list));
        } else {
            data.insert("query".to_string(), Value::String(s.query));
        }
        data
    }

    fn header(&self) -> HeaderMap {
        let mut header = HeaderMap::new();
        header.insert(
            "X-QuakeToken",
            HeaderValue::from_str(self.api_key.as_str()).unwrap(),
        );
        header
    }
}
