// 持久化层：文件读写与数据保存，所有函数为独立自由函数

use crate::display;
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::{env, fs};

pub fn save_domain_data(
    filename: &str,
    content: Value,
    data_type: Vec<&str>,
) -> io::Result<i32> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;
    let domains: Vec<String> = display::show_domain(content, false, false, data_type);
    let mut count = 0;
    for domain in domains {
        f.write_all(format!("{}\n", domain).as_bytes())?;
        count += 1;
    }
    Ok(count)
}

pub fn save_host_data(filename: &str, content: Value) -> io::Result<i32> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;
    let hosts = display::show_host(content, false);
    let mut count = 0;
    for host in hosts {
        f.write_all(format!("{}\n", host).as_bytes())?;
        count += 1;
    }
    Ok(count)
}

pub fn save_search_data(
    filename: &str,
    content: Value,
    filter: &str,
    data_type: Vec<&str>,
) -> io::Result<i32> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;
    let hosts = display::show(content, false, filter, data_type);
    let mut count = 0;
    for host in hosts {
        f.write_all(format!("{}\n", host).as_bytes())?;
        count += 1;
    }
    Ok(count)
}

pub fn save_scroll_data(
    filename: &str,
    content: Vec<Value>,
    filter: &str,
    data_type: Vec<&str>,
) -> io::Result<i32> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;
    let hosts = display::show_scroll(content, false, filter, data_type);
    let mut count = 0;
    for host in hosts {
        f.write_all(format!("{}\n", host).as_bytes())?;
        count += 1;
    }
    Ok(count)
}

pub fn save_host_by_scroll_data(filename: &str, content: Vec<Value>) -> io::Result<i32> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;
    let hosts = display::show_host_by_scroll(content, false);
    let mut count = 0;
    for host in hosts {
        f.write_all(format!("{}\n", host).as_bytes())?;
        count += 1;
    }
    Ok(count)
}

pub fn read_file_search(filename: &str) -> String {
    let mut file = fs::File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    //print!("{:?}",contents);
    let newline = match env::consts::OS {
        "windows" => "\r\n",  // Windows 使用 \r\n
        _ => "\n",            // 其他系统使用 \n
    };
    let contents_or = contents.replace(newline, " OR ");
    let contents_end = &contents_or[contents_or.len() - 4..contents_or.len()];
    if contents_end == " OR " {
        let query = &contents_or[0..contents_or.len() - 4];
        query.to_string()
    } else {
        let query = &contents_or;
        query.to_string()
    }
}

pub fn read_file_host(filename: &str) -> String {
    // 读取文件
    let mut file = fs::File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    //print!("{:?}", contents);
    //判断系统版本
    let newline = match env::consts::OS {
        "windows" => "\r\n",  // Windows 使用 \r\n
        _ => "\n",            // 其他系统使用 \n
    };
    let contents_hosts = contents.replace(newline, "\" OR ip:\"");
    let contents_end = &contents_hosts[contents_hosts.len() - 8..contents_hosts.len()];
    if contents_end == " OR ip:\"" {
        let query = &contents_hosts[0..contents_hosts.len() - 8];
        let query_host = &*("ip:\"".to_owned() + query);
        query_host.to_string()
    } else {
        let query = &contents_hosts;
        let query_host = &*("ip:\"".to_owned() + query + "\"");
        query_host.to_string()
    }
}
