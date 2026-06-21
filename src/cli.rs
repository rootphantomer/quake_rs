// CLI 层：命令行参数解析与子命令路由
// 从 common.rs 迁移而来，移除过渡期 re-export

use crate::api::ApiKey;
use crate::client::Quake;
use crate::display;
use crate::gpt::Gpt;
use crate::models::{AggService, Output};
use crate::persistence;
use clap::{Arg, ArgAction, Command};
use regex::Regex;
use serde_json::Value;
use std::error::Error;

/// 从 clap 匹配结果中解析 i32 参数，失败时返回默认值
fn parse_i32(matches: &clap::ArgMatches, name: &str, default: i32) -> i32 {
    matches
        .get_one::<String>(name)
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(default)
}

/// 从 clap 匹配结果中获取字符串参数，失败时返回默认值
fn get_str<'a>(matches: &'a clap::ArgMatches, name: &str, default: &'a str) -> String {
    matches
        .get_one::<String>(name)
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

pub struct ArgParse;

impl ArgParse {
    pub fn parse() {
        let matches = Command::new("Quake Command-Line Application")
            .version("3.1.7")
            .author("Author: 360 Quake Team  <quake@360.cn>")
            .about("Dose awesome things.")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(
                Command::new("init")
                    .about("Initialize the Quake command-line")
                    .arg(
                        Arg::new("Api_Key")
                            .index(1)
                            .action(ArgAction::Set)
                            .help("Initialize the Quake command-line")
                    )
            ).subcommand(
                Command::new("gptinit")
                    .about("Initialize the gtpapi")
                    .arg(
                        Arg::new("gpt_Key")
                            .action(ArgAction::Set)
                            .help("Initialize the gptapi")
                    )
            )
            .subcommand(
                Command::new("info")
                    .about("Shows general information about your account")
            )
            .subcommand(
                Command::new("host")
                    .about("View all available information for an IP address")
                    .arg(
                        Arg::new("ip")
                            .index(1)
                            .action(ArgAction::Set)
                            .help(" View all available information for an IP address")
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .action(ArgAction::Set)
                            .help("Save the host information in the given file (append if file exists).")
                            .value_name("FILENAME")
                    )
                    .arg(
                        Arg::new("query_host_file")
                            .short('q')
                            .long("query_host_file")
                            .action(ArgAction::Set)
                            .help("Quake Host file(Only support --size); Example: quake search -q hosts.txt")
                            .value_name("FILENAME")
                    )
                    .arg(
                        Arg::new("size")
                            .long("size")
                            .action(ArgAction::Set)
                            .value_name("NUMBER")
                            .help("The size of the number of responses, up to a maximum of 100 (Default 10).")
                    )
                    .arg(
                        Arg::new("start")
                            .long("start")
                            .action(ArgAction::Set)
                            .value_name("NUMBER")
                            .help("Starting position of the query (Default 0).")
                    )
                    .arg(
                        Arg::new("type")
                            .short('t')
                            .long("type")
                            .action(ArgAction::Set)
                            .value_name("TYPE")
                            .help("Fields displayed:ip,port,title,country,province,city,owner,time,ssldomain. (Default ip,port)")
                    )
            )
            .subcommand(
                Command::new("search")
                    .about("Search the Quake database")
                    .arg(
                        Arg::new("query_string")
                            .index(1)
                            .action(ArgAction::Set)
                            .help("Quake Querystring; Example: quake search 'port:80'")
                    )
                    .arg(
                        Arg::new("query_file")
                            .short('q')
                            .long("query_file")
                            .action(ArgAction::Set)
                            .help("Quake Querystring file; Example: quake search -q test.txt")
                            .value_name("FILENAME")
                    )
                    .arg(
                        Arg::new("time_start")
                            .short('s')
                            .long("start_time")
                            .action(ArgAction::Set)
                            .help("Search start time\r\n\
                            Example: quake search 'port:80' -s 2020-01-01")
                            .value_name("TIME START")
                    )
                    .arg(
                        Arg::new("time_end")
                            .short('e')
                            .long("end_time")
                            .action(ArgAction::Set)
                            .help("Search end time\r\n\
                            Example: quake search 'port:80' -e 2020-01-01")
                            .value_name("TIME END")
                    )
                    .arg(
                        Arg::new("upload")
                            .short('u')
                            .long("upload")
                            .action(ArgAction::Set)
                            .help("Uploading *.txt files containing only IP addresses, with no more than 1000 IPs.\r\n\
                            Example: quake search -u ips.txt")
                            .value_name("IP File")
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .action(ArgAction::Set)
                            .help("Save the host information in the given file (append if file exists).")
                            .value_name("FILENAME")
                    )
                    .arg(
                        Arg::new("size")
                            .long("size")
                            .action(ArgAction::Set)
                            .value_name("NUMBER")
                            .help("The size of the number of responses, up to a maximum of 100 (Default 10).")
                    )
                    .arg(
                        Arg::new("start")
                            .long("start")
                            .action(ArgAction::Set)
                            .value_name("NUMBER")
                            .help("Starting position of the query (Default 0).")
                    )
                    .arg(
                        Arg::new("type")
                            .short('t')
                            .long("type")
                            .action(ArgAction::Set)
                            .value_name("TYPE")
                            .help("Fields displayed:ip,port,title,product_name_cn,version,protocol,country,province,city,owner,time,ssldomain,domain. (Default ip,port)")
                    )
                    .arg(
                        Arg::new("filter")
                            .short('f')
                            .long("filter")
                            .action(ArgAction::Set)
                            .value_name("TYPE")
                            .help("Filter search results with more regular expressions.\r\n\
                            Example: quake search 'app:\"exchange 2010\"' -t ip,port,title -f \"X-OWA-Version: (.*)\"")
                    )
                    .arg(Arg::new("cdn")
                        .short('c')
                        .long("cdn")
                        .action(ArgAction::Set)
                        .value_name("NUMBER")
                        .help("Exclude cdn data when parameter is 1,Not excluded by default"))
                    .arg(Arg::new("honey_jar")
                        .short('m')
                        .long("honey_jar")
                        .action(ArgAction::Set)
                        .value_name("NUMBER")
                        .help("Exclude honey_jar data when parameter is 1,Not excluded by default"))
                    .arg(Arg::new("latest_data")
                        .short('l')
                        .long("latest_data")
                        .action(ArgAction::Set)
                        .value_name("NUMBER")
                        .help("Display latest data when parameter is 1,Not up to date by default"))
                    .arg(Arg::new("filter_request")
                        .short('r')
                        .long("filter_request")
                        .action(ArgAction::Set)
                        .value_name("NUMBER")
                        .help("When the parameter is 1, invalid requests are filtered, such as 400, 401, 403 and other request data, the default is not filtered"))
                    .arg(Arg::new("deduplication")
                        .short('d')
                        .long("deduplication")
                        .action(ArgAction::Set)
                        .value_name("NUMBER")
                        .help("When the parameter is 1, data deduplication is performed, and no deduplication is performed by default."))
            )
            .subcommand(
                Command::new("domain")
                    .about("View all available information for a domain.")
                    .arg(Arg::new("cdn")
                        .short('n')
                        .long("cdn")
                        .action(ArgAction::Set)
                        .value_name("NUMBER")
                        .help("Exclude cdn data when parameter is 1,Not excluded by default"))
                    .arg(Arg::new("honey_jar")
                        .short('m')
                        .long("honey_jar")
                        .action(ArgAction::Set)
                        .value_name("NUMBER")
                        .help("Exclude honey_jar data when parameter is 1,Not excluded by default"))
                    .arg(Arg::new("latest_data")
                        .short('l')
                        .long("latest_data")
                        .action(ArgAction::Set)
                        .value_name("NUMBER")
                        .help("Display latest data when parameter is 1,Not up to date by default"))

                    .arg(
                        Arg::new("domain_name")
                            .index(1)
                            .action(ArgAction::Set)
                            .value_name("DOMAIN_NAME")
                            .help("The domain name to be queried.")
                    )
                    .arg(
                        Arg::new("count")
                            .short('c')
                            .long("count")
                            .action(ArgAction::Set)
                            .value_name("NUMBER")
                            .help("Count of results")
                    )
                    .arg(
                        Arg::new("size")
                            .long("size")
                            .action(ArgAction::Set)
                            .value_name("NUMBER")
                            .help("The size of the number of responses, up to a maximum of 100 (Default 10).")
                    )
                    .arg(
                        Arg::new("start")
                            .long("start")
                            .action(ArgAction::Set)
                            .value_name("NUMBER")
                            .help("Starting position of the query (Default 0).")
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .action(ArgAction::Set)
                            .value_name("FILENAME")
                            .help("Output result to file.")
                    )
                    .arg(
                        Arg::new("type")
                            .short('t')
                            .long("type")
                            .action(ArgAction::Set)
                            .value_name("TYPE")
                            .help("Fields displayed:domain,ip,port,title. (Default domain, ip, port)")
                    )
                    .arg(Arg::new("filter_request")
                        .short('r')
                        .long("filter_request")
                        .action(ArgAction::Set)
                        .value_name("NUMBER")
                        .help("When the parameter is 1, invalid requests are filtered, such as 400, 401, 403 and other request data, the default is not filtered"))
                    .arg(Arg::new("deduplication")
                        .short('d')
                        .long("deduplication")
                        .action(ArgAction::Set)
                        .value_name("NUMBER")
                        .help("When the parameter is 1, data deduplication is performed, and no deduplication is performed by default."))
            )
            .subcommand(
                Command::new("honeypot")
                    .about("Check whether the IP is a honeypot or not.")
                    .arg(
                        Arg::new("ip")
                            .index(1)
                            .action(ArgAction::Set)
                            .value_name("ip")
                            .help("The ip address to be queried.")
                    )
            ) .subcommand(
                Command::new("gpt")
                    .about("Artificial intelligence engine, directly say what you want to check without grammar")
                    .arg(
                        Arg::new("gpt_match")
                            .index(1)
                            .action(ArgAction::Set)
                            .value_name("GPT_MATVH")
                            .help("what to say")
                    ).arg(
                        Arg::new("size")
                            .long("size")
                            .action(ArgAction::Set)
                            .value_name("NUMBER")
                            .help("The size of the number of responses, up to a maximum of 100 (Default 10).")
                    )
            )
            .get_matches();

        match matches.subcommand() {
            Some(("init", init_match)) => {
                if let Some(api_key) = init_match.get_one::<String>("Api_Key") {
                    ApiKey::init(api_key.to_string());
                }
            }
            Some(("gptinit", init_match)) => {
                if let Some(api_key) = init_match.get_one::<String>("gpt_Key") {
                    ApiKey::gptinit(api_key.to_string());
                }
            }
            Some(("domain", domain_match)) => {
                let domain = match domain_match.get_one::<String>("domain_name") {
                    Some(domain) => domain.to_string(),
                    None => {
                        Output::error(
                            "Error: You must choose a domain name.\r\nPlease execute -h for help.",
                        );
                        std::process::exit(1);
                    }
                };
                let start = parse_i32(domain_match, "start", 0);
                let size = parse_i32(domain_match, "size", 10);
                if size > 100 {
                    Output::warning("Warning: Size is set to a maximum of 100, if set too high it may cause abnormal slowdowns or timeouts.");
                }
                let query = format!("domain:*.{}", domain);
                let data_type_str = get_str(domain_match, "type", "ip,port,domain");
                let data_type: Vec<&str> = data_type_str.split(',').collect();

                let cdn = parse_i32(domain_match, "cdn", 0);
                let mg = parse_i32(domain_match, "honey_jar", 0);
                let zxsj = parse_i32(domain_match, "latest_data", 0);
                let wxqq = parse_i32(domain_match, "filter_request", 0);
                let sjqc = parse_i32(domain_match, "deduplication", 0);
                let response =
                    Quake::query(&query, "", start, size, "", "", cdn, mg, zxsj, wxqq, sjqc);

                let _count = parse_i32(domain_match, "count", 0);
                let onlycount = false;

                let output = match domain_match.get_one::<String>("output") {
                    Some(output) => output.to_string(),
                    None => {
                        display::show_domain(response, onlycount, true, data_type);
                        let res = ApiKey::get_api().expect("Failed to read apikey:\t");
                        let info_jf = match Quake::new(res).info() {
                            Ok(value) => value,
                            Err(e) => {
                                Output::error(&format!("Query failed: {}", e));
                                std::process::exit(1);
                            }
                        };
                        display::show_info_jf(info_jf);
                        std::process::exit(0);
                    }
                };
                match persistence::save_domain_data(&output, response, data_type) {
                    Ok(count) => {
                        Output::success(&format!(
                            "Successfully saved {} pieces of data to {}",
                            count, output
                        ));
                    }
                    Err(e) => {
                        Output::error(&format!("Data saving failure:{}", e));
                    }
                };
            }
            Some(("host", host_match)) => {
                let query_host_file = get_str(host_match, "query_host_file", "");
                let start = parse_i32(host_match, "start", 0);
                let size = parse_i32(host_match, "size", 10);
                if size > 100 {
                    Output::warning("Warning: Size is set to a maximum of 100, if set too high it may cause abnormal slowdowns or timeouts.");
                }
                if query_host_file.is_empty() {
                    let ip = match host_match.get_one::<String>("ip") {
                        Some(ip) => ip.to_string(),
                        None => {
                            Output::error(
                                "Error: You must choose a ip or cidr.\r\nPlease execute -h for help.",
                            );
                            std::process::exit(1);
                        }
                    };
                    let query = &format!("ip:{}", ip);
                    let response = Quake::query_host(query, start, size);
                    let output = match host_match.get_one::<String>("output") {
                        Some(name) => name.to_string(),
                        None => {
                            display::show_host(response, true);
                            let res = ApiKey::get_api().expect("Failed to read apikey:\t");
                            let info_jf = match Quake::new(res).info() {
                                Ok(value) => value,
                                Err(e) => {
                            Output::error(&format!("Query failed: {}", e));
                                    std::process::exit(1);
                                }
                            };
                            display::show_info_jf(info_jf);
                            std::process::exit(0);
                        }
                    };
                    let filename = &output;
                    // save to file.
                    match persistence::save_host_data(filename, response) {
                        Ok(count) => {
                            Output::success(&format!(
                                "Successfully saved {} pieces of data to {}",
                                count, output
                            ));
                        }
                        Err(e) => {
                            Output::error(&format!("Data saving failure:{}", e));
                        }
                    };
                } else {
                    let host_string = persistence::read_file_host(&query_host_file);
                    let query = host_string.as_str();
                    if query.is_empty() {
                        Output::info("The host file is None!");
                        std::process::exit(1);
                    }
                    let response = Quake::query_host_by_scroll(query, size);
                    let output = match host_match.get_one::<String>("output") {
                        Some(name) => name.to_string(),
                        None => {
                            display::show_host_by_scroll(response, true);
                            std::process::exit(0);
                        }
                    };
                    let filename = &output;
                    // save to file
                    match persistence::save_host_by_scroll_data(filename, response) {
                        Ok(count) => {
                            Output::success(&format!(
                                "Successfully saved {} pieces of data to {}",
                                count, output
                            ));
                        }
                        Err(e) => {
                            Output::error(&format!("Data saving failure:{}", e));
                        }
                    };
                }
            }
            Some(("search", search_match)) => {
                let upload = get_str(search_match, "upload", "");
                let query_file = get_str(search_match, "query_file", "");
                let query_string;
                let query = match search_match.get_one::<String>("query_string") {
                    Some(query) => query.to_string(),
                    None => {
                        if upload.is_empty() && query_file.is_empty() {
                            Output::error("Error: You must enter a search syntax.\r\nPlease execute -h for help.");
                            std::process::exit(1);
                        } else if !query_file.is_empty() {
                            query_string = persistence::read_file_search(&query_file);
                            query_string
                        } else {
                            String::new()
                        }
                    }
                };
                let start = parse_i32(search_match, "start", 0);
                let size = parse_i32(search_match, "size", 10);
                let cdn = parse_i32(search_match, "cdn", 0);
                let mg = parse_i32(search_match, "honey_jar", 0);
                let zxsj = parse_i32(search_match, "latest_data", 0);
                let wxqq = parse_i32(search_match, "filter_request", 0);
                let sjqc = parse_i32(search_match, "deduplication", 0);
                let time_start = get_str(search_match, "time_start", "");
                let time_end = get_str(search_match, "time_end", "");
                if size > 100 {
                    Output::warning("Warning: Size is set to a maximum of 100, if set too high it may cause abnormal slowdowns or timeouts.");
                }
                let data_type_str = get_str(search_match, "type", "ip,port,title");
                let data_type: Vec<&str> = data_type_str.split(',').collect();
                let filter = get_str(search_match, "filter", "");
                if query_file.is_empty() {
                    let response = Quake::query(
                        &query, &upload, start, size, &time_start, &time_end, cdn, mg, zxsj, wxqq, sjqc,
                    );
                    let output = match search_match.get_one::<String>("output") {
                        Some(name) => name.to_string(),
                        None => {
                            display::show(response, true, &filter, data_type);
                            let res = ApiKey::get_api().expect("Failed to read apikey:\t");
                            let info_jf = match Quake::new(res).info() {
                                Ok(value) => value,
                                Err(e) => {
                                    Output::error(&format!("Query failed: {}", e));
                                    std::process::exit(1);
                                }
                            };
                            display::show_info_jf(info_jf);
                            std::process::exit(0);
                        }
                    };
                    match persistence::save_search_data(&output, response, &filter, data_type) {
                        Ok(count) => {
                            Output::success(&format!(
                                "Successfully saved {} pieces of data to {}",
                                count, output
                            ));
                        }
                        Err(e) => {
                            Output::error(&format!("Data saving failure:{}", e));
                        }
                    };
                } else {
                    if !query.is_empty() {
                        Output::info(&format!("Search with {}", query));
                    }
                    let response = Quake::query_for_scroll(
                        &query, size, &time_start, &time_end, cdn, mg, zxsj, wxqq, sjqc,
                    );
                    let output = match search_match.get_one::<String>("output") {
                        Some(name) => name.to_string(),
                        None => {
                            display::show_scroll(response, true, &filter, data_type);
                            std::process::exit(0);
                        }
                    };
                    match persistence::save_scroll_data(&output, response, &filter, data_type) {
                        Ok(count) => {
                            Output::success(&format!(
                                "Successfully saved {} pieces of data to {}",
                                count, output
                            ));
                        }
                        Err(e) => {
                            Output::error(&format!("Data saving failure:{}", e));
                        }
                    };
                }
            }
            Some(("info", _)) => {
                let res = ApiKey::get_api().expect("Failed to read apikey:\t");
                let info = match Quake::new(res).info() {
                    Ok(value) => value,
                    Err(e) => {
                        Output::error(&format!("Query failed: {}", e));
                        std::process::exit(1);
                    }
                };
                display::show_info(info);
            }
            //gpt引擎
            Some(("gpt", gpt_match)) => {
                let gptcs = match gpt_match.get_one::<String>("gpt_match") {
                    Some(gptcs) => gptcs.to_string(),
                    None => {
                        Output::error("You have to say something. \r\nPlease execute -h for help.");
                        std::process::exit(1);
                    }
                };

                let gpt_sj: Result<String, Box<dyn Error>> = match Gpt::query_gpt(&gptcs) {
                    Ok(res) => {
                        Output::info(&format!(
                            "Successfully converted the quake language method:{}",
                            res
                        ));
                        Ok(res)
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        Err(err)
                    }
                };
                let upload = get_str(gpt_match, "upload", "");
                let query_file = get_str(gpt_match, "query_file", "");
                let query = gpt_sj.unwrap().trim_matches('"').replace('\\', "");

                let sizere = Regex::new(r"--size\s+(\d+)").unwrap();
                let time_startre = Regex::new(r"--time_start\s+(\d+-\d+)(?:-\d+)?").unwrap();
                let time_endre = Regex::new(r"--time_end\s+(\d+-\d+)(?:-\d+)?").unwrap();
                let outputre = Regex::new(r"--output\s+([^\s]+)").unwrap();
                let andre = Regex::new(r"and\s*$").unwrap();
                let size = sizere
                    .captures(&query)
                    .and_then(|c| c.get(1))
                    .and_then(|m| m.as_str().parse::<i32>().ok())
                    .unwrap_or(10);
                let time_start = time_startre
                    .captures(&query)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let time_end = time_endre
                    .captures(&query)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let start = parse_i32(gpt_match, "start", 0);
                let cdn = parse_i32(gpt_match, "cdn", 0);
                let mg = parse_i32(gpt_match, "honey_jar", 0);
                let zxsj = parse_i32(gpt_match, "latest_data", 0);
                let wxqq = parse_i32(gpt_match, "filter_request", 0);
                let sjqc = parse_i32(gpt_match, "deduplication", 0);

                let query = sizere.replace_all(&query, "").to_string();
                let query = time_startre.replace_all(&query, "").to_string();
                let query = time_endre.replace_all(&query, "").to_string();
                let outputquery = query.clone();
                let query = outputre.replace_all(&query, "").to_string();
                let query = andre.replace_all(&query, "").to_string();
                let query = query.replace('"', "");

                if size > 100 {
                    Output::warning("Warning: Size is set to a maximum of 100, if set too high it may cause abnormal slowdowns or timeouts.");
                }

                let data_type_str = get_str(gpt_match, "type", "ip,port,title");
                let data_type: Vec<&str> = data_type_str.split(',').collect();
                let filter = get_str(gpt_match, "filter", "");

                if query_file.is_empty() {
                    let response = Quake::query(
                        &query, &upload, start, size, &time_start, &time_end, cdn, mg, zxsj, wxqq, sjqc,
                    );
                    let output = outputre
                        .captures(&outputquery)
                        .and_then(|c| c.get(1))
                        .map(|m| m.as_str().to_string());
                    let output = match output {
                        Some(name) => name,
                        None => {
                            display::show(response, true, &filter, data_type);
                            std::process::exit(0);
                        }
                    };
                    match persistence::save_search_data(&output, response, &filter, data_type) {
                        Ok(count) => {
                            Output::success(&format!(
                                "Successfully saved {} pieces of data to {}",
                                count, output
                            ));
                        }
                        Err(e) => {
                            Output::error(&format!("Data saving failure:{}", e));
                        }
                    };
                } else {
                    if !query.is_empty() {
                        Output::info(&format!("Search with {}", query));
                    }
                    let response = Quake::query_for_scroll(
                        &query, size, &time_start, &time_end, cdn, mg, zxsj, wxqq, sjqc,
                    );
                    let output = match gpt_match.get_one::<String>("output") {
                        Some(name) => name.to_string(),
                        None => {
                            display::show_scroll(response, true, &filter, data_type);
                            std::process::exit(0);
                        }
                    };
                    match persistence::save_scroll_data(&output, response, &filter, data_type) {
                        Ok(count) => {
                            Output::success(&format!(
                                "Successfully saved {} pieces of data to {}",
                                count, output
                            ));
                        }
                        Err(e) => {
                            Output::error(&format!("Data saving failure:{}", e));
                        }
                    };
                }
            }
            Some(("honeypot", honeypot_match)) => {
                let ip = match honeypot_match.get_one::<String>("ip") {
                    Some(query) => query.to_string(),
                    None => {
                        Output::error(
                            "Error: You must choose a ip.\r\nPlease execute -h for help.",
                        );
                        std::process::exit(1);
                    }
                };
                Output::info(&format!("Search with {}", ip));
                let mut query = String::from("app: \"*蜜罐*\" AND ip:");
                query += &ip;
                let res = ApiKey::get_api().expect("Failed to read apikey:\t");
                let s = AggService {
                    query,
                    start: 0,
                    size: 5,
                    ignore_cache: false,
                    aggregation_list: vec![String::from("app")],
                };
                let response: Value = match Quake::new(res).aggservice(&s) {
                    Ok(response) => response,
                    Err(e) => {
                        Output::error(&format!("Query failed: {}", e));
                        std::process::exit(1);
                    }
                };
                display::display_honeypot(response);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgAction, Command};

    fn test_command() -> Command {
        Command::new("test")
            .arg(Arg::new("num").long("num").action(ArgAction::Set))
            .arg(Arg::new("text").long("text").action(ArgAction::Set))
            .arg(Arg::new("flag").long("flag").action(ArgAction::Set))
    }

    // ========== parse_i32 测试 ==========

    #[test]
    fn test_parse_i32_valid_value() {
        let matches = test_command()
            .try_get_matches_from(["test", "--num", "42"])
            .unwrap();
        assert_eq!(parse_i32(&matches, "num", 0), 42);
    }

    #[test]
    fn test_parse_i32_default_value() {
        let matches = test_command()
            .try_get_matches_from(["test"])
            .unwrap();
        assert_eq!(parse_i32(&matches, "num", 10), 10);
    }

    #[test]
    fn test_parse_i32_invalid_value_returns_default() {
        let matches = test_command()
            .try_get_matches_from(["test", "--num", "not_a_number"])
            .unwrap();
        assert_eq!(parse_i32(&matches, "num", 99), 99);
    }

    #[test]
    fn test_parse_i32_zero() {
        let matches = test_command()
            .try_get_matches_from(["test", "--num", "0"])
            .unwrap();
        assert_eq!(parse_i32(&matches, "num", 10), 0);
    }

    #[test]
    fn test_parse_i32_negative() {
        let matches = test_command()
            .try_get_matches_from(["test", "--num=-5"])
            .unwrap();
        assert_eq!(parse_i32(&matches, "num", 0), -5);
    }

    // ========== get_str 测试 ==========

    #[test]
    fn test_get_str_valid_value() {
        let matches = test_command()
            .try_get_matches_from(["test", "--text", "hello"])
            .unwrap();
        assert_eq!(get_str(&matches, "text", "default"), "hello");
    }

    #[test]
    fn test_get_str_default_value() {
        let matches = test_command()
            .try_get_matches_from(["test"])
            .unwrap();
        assert_eq!(get_str(&matches, "text", "fallback"), "fallback");
    }

    #[test]
    fn test_get_str_empty_string_value() {
        let matches = test_command()
            .try_get_matches_from(["test", "--text", ""])
            .unwrap();
        assert_eq!(get_str(&matches, "text", "default"), "");
    }

    #[test]
    fn test_get_str_with_special_chars() {
        let matches = test_command()
            .try_get_matches_from(["test", "--text", "port:80 AND ip:1.1.1.1"])
            .unwrap();
        assert_eq!(
            get_str(&matches, "text", ""),
            "port:80 AND ip:1.1.1.1"
        );
    }

    #[test]
    fn test_get_str_chinese_chars() {
        let matches = test_command()
            .try_get_matches_from(["test", "--text", "你好世界"])
            .unwrap();
        assert_eq!(get_str(&matches, "text", ""), "你好世界");
    }
}
