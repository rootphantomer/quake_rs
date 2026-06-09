# Quake Command-Line Application

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)
[![Release](https://img.shields.io/github/v/release/360quake/quake_rs)](https://github.com/360quake/quake_rs/releases)

360 Quake 网络空间测绘系统的命令行工具，支持资产搜索、域名查询、IP 查询、蜜罐识别及 GPT 智能语法转换。

## 功能特性

- **search** — 使用 Quake 语法搜索全网资产
- **host** — 查询 IP/CIDR 开放端口与服务
- **domain** — 查询域名子域及关联信息
- **honeypot** — 识别蜜罐系统
- **info** — 查看账户信息与剩余积分
- **gpt** — AI 自动将自然语言转换为 Quake 搜索语法

## 安装

### 下载预编译二进制

前往 [Releases](https://github.com/360quake/quake_rs/releases/) 页面下载对应平台的二进制文件。

### 从源码编译

确保已安装 [Rust](https://www.rust-lang.org/tools/install)，然后执行：

```bash
cargo build --release
```

编译产物位于 `target/release/quake`。

## 快速开始

### 1. 初始化 API Key

API Key 请在 [Quake 个人中心](https://quake.360.net/centre/user-info) 获取。

```bash
quake init <your_api_key>
```

如需使用 GPT 功能，还需初始化 OpenAI API Key（从 [OpenAI Platform](https://platform.openai.com/account/api-keys) 获取）：

```bash
quake gptinit <your_openai_api_key>
```

### 2. 搜索查询

```bash
# 基本搜索
quake search 'port:80'

# 指定字段和时间范围
quake search 'port:80' -t ip,port,title -s 2020-01-01 -e 2023-01-01

# 从文件批量查询并导出结果
quake search -q query.txt -o result.txt

# 上传 IP 列表批量查询（不超过 1000 条）
quake search -u ips.txt

# 使用正则过滤
quake search 'app:"exchange 2010"' -t ip,port,title -f "X-OWA-Version: (.*)"
```

**search 支持的字段（`-t` 参数）**：

| 字段 | 说明 |
|------|------|
| ip | IP 地址 |
| port | 端口 |
| title | 站点标题 |
| product_name_cn | 产品名称 |
| version | 版本号 |
| protocol | 协议 |
| country | 国家 |
| province | 省份 |
| city | 城市 |
| owner | 运营商 |
| time | 时间 |
| ssldomain | SSL 证书域名 |
| domain | 域名 |

**search 常用选项**：

| 选项 | 说明 |
|------|------|
| `-o, --output` | 导出结果到文件 |
| `-q, --query_file` | 从文件读取查询语句 |
| `-u, --upload` | 上传 IP 列表文件 |
| `-s, --start_time` | 搜索开始时间 |
| `-e, --end_time` | 搜索结束时间 |
| `--size` | 返回条数（最大 100，默认 10） |
| `--start` | 分页起始位置（默认 0） |
| `-f, --filter` | 正则表达式过滤 |
| `-c, --cdn` | 设为 1 排除 CDN 数据 |
| `-m, --honey_jar` | 设为 1 排除蜜罐数据 |
| `-l, --latest_data` | 设为 1 仅显示最新数据 |
| `-r, --filter_request` | 设为 1 过滤无效请求（400/401/403 等） |
| `-d, --deduplication` | 设为 1 对数据去重 |

### 3. IP 查询

```bash
# 单个 IP
quake host 5.188.34.101

# CIDR 段
quake host 5.188.34.101/24

# 从文件批量查询并导出
quake host -q hosts.txt -o host_result.txt
```

### 4. 域名查询

```bash
# 查询域名关联信息
quake domain 360.cn

# 指定字段和翻页
quake domain 360.cn -t ip,port,domain,title --start 10 --size 10

# 导出结果
quake domain 360.cn -o domain_result.txt
```

### 5. 蜜罐识别

```bash
quake honeypot 93.89.146.23
```

### 6. 用户信息

```bash
quake info
```

### 7. GPT 智能查询

使用自然语言描述需求，自动转换为 Quake 搜索语法：

```bash
# 搜索返回包里含有 admin 的数据
quake gpt '搜索返回包里里面有admin'

# 查询特定地区和类型
quake gpt '来一打中国江西apache服务器数据'

# 指定时间范围和排除条件
quake gpt '来20个不要来自台湾的apache服务器数据从2022年1月到2023年1月'

# 导出结果
quake gpt '来20个河南的linux服务器数据从2021年到2022年导出到当前目录下a.txt'
```

> **注意**：GPT 功能处于训练测试阶段，不一定 100% 得到预期结果，欢迎反馈。

## 帮助信息

```bash
quake --help
quake <subcommand> --help
```

## 问题反馈

请添加微信：quake_360 邀您加入技术交流群 :)

## 更新日志

参见 [CHANGELOG.md](CHANGELOG.md)。
