<div align="center">

# Quake Command-Line Application

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)
[![Release](https://img.shields.io/github/v/release/360quake/quake_rs)](https://github.com/360quake/quake_rs/releases)
[![CI](https://img.shields.io/badge/CI-GitHub%20Actions-brightgreen)]()
[![Architecture](https://img.shields.io/badge/Architecture-Modular-green)]()

360 Quake 网络空间测绘系统的官方命令行工具。支持资产搜索、IP/CIDR 查询、域名关联查询、蜜罐识别以及 GPT 智能语法转换，让网络安全测绘工作直接在终端完成。

[功能特性](#功能特性) · [安装](#安装) · [快速开始](#快速开始) · [开发](#开发) · [FAQ](#常见问题)

</div>

## 项目架构

quake_rs 采用模块化分层架构，职责清晰：

```
src/
├── main.rs          # 入口点，模块声明与初始化
├── persistence.rs   # 持久化层（文件读写，save_*/read_file_* 独立函数）
├── client.rs        # HTTP API 客户端（Quake 结构体，封装所有 API 请求）
├── display.rs       # 数据展示层（格式化输出，show_* 系列独立函数）
├── cli.rs           # CLI 层（clap 参数定义 + 子命令分发路由）
├── api.rs           # API Key 管理（初始化、校验、存取）
├── models.rs        # 数据模型层（Service/Scroll/Host 等结构体 + Output 工具 + 时间函数）
└── gpt.rs           # GPT 智能语法转换客户端
```

模块间引用关系：

```
cli → client, display, persistence, api, gpt, models
client → models, api
display → models
persistence → display
api → client, models
gpt → api, models
models → (无内部依赖)
```

详细系统设计文档见 [docs/](docs/) 目录，包含 Mermaid 类图、时序图和完整的架构说明。

## 安装

### 下载预编译二进制

从 [Releases](https://github.com/360quake/quake_rs/releases) 页面下载对应平台的二进制文件即可直接使用。

### 从源码编译

确保已安装 [Rust](https://www.rust-lang.org/tools/install)（1.70+），然后执行：

```bash
cargo build --release
```

编译产物位于 `target/release/quake`。

### 系统要求

| 项目 | 要求 |
|------|------|
| Rust 版本 | 1.70+ |
| 操作系统 | Linux / macOS / Windows |
| 架构 | x86_64, aarch64 |

### 验证安装

```bash
./target/release/quake --version
./target/release/quake --help
```

## 功能特性

| 子命令 | 功能 | 说明 |
|--------|------|------|
| `search` | 资产搜索 | 使用 Quake 语法搜索全网服务与资产 |
| `host` | IP 查询 | 查询 IP/CIDR 的开放端口与运行服务 |
| `domain` | 域名查询 | 查询域名关联的子域、IP 与服务 |
| `honeypot` | 蜜罐识别 | 检测目标 IP 是否为蜜罐系统 |
| `info` | 账户信息 | 查看账户资料、月度/长效积分与角色权限 |
| `gpt` | 智能搜索 | AI 自动将自然语言转换为 Quake 搜索语法并执行 |

## 快速开始

### 1. 初始化 API Key

API Key 请从 [Quake 个人中心](https://quake.360.net/centre/user-info) 获取。

```bash
quake init <your_api_key>
```

API Key 会保存在 `~/.config/quake/api_key` 文件中。初始化时会自动校验 Key 的有效性。

如需使用 GPT 功能，还需初始化 OpenAI API Key：

```bash
quake gptinit <your_openai_api_key>
```

GPT API Key 保存在 `~/.config/quake/gptapi_key`，仅在使用 `quake gpt` 子命令时需要。

### 2. 查看账户信息

```bash
quake info
```

输出示例：

```
[+] Successful.
[+] 用户名:  user@example.com
[+] 邮  箱:  user@example.com
[+] 手  机:  138****1234
[+] 月度积分: 5000
[+] 长效积分: 12000
[+] 角  色: 普通用户
```

### 3. 搜索资产

```bash
# 基本搜索
quake search 'port:80'

# 指定字段和时间范围
quake search 'port:80' -t ip,port,title -s 2020-01-01 -e 2023-01-01

# 从文件批量查询（逐行读取并用 OR 拼接，走滚动分页拉取全量数据）
quake search -q query.txt -o result.txt

# 上传 IP 列表查询（不超过 1000 条）
quake search -u ips.txt

# 使用正则过滤响应体
quake search 'app:"exchange 2010"' -t ip,port,title -f "X-OWA-Version: (.*)"

# 排除 CDN 和蜜罐数据
quake search 'port:443' -c 1 -m 1

# 过滤无效请求（400/401/403）并去重
quake search 'port:80' -r 1 -d 1
```

### 4. IP 查询

```bash
# 单个 IP
quake host 5.188.34.101

# CIDR 段
quake host 5.188.34.101/24

# 从文件批量查询 IP（自动滚动分页，不限制条数）
quake host -q hosts.txt -o host_result.txt

# 指定字段和分页
quake host 5.188.34.101 -t ip,port,title,country,province,city --start 0 --size 20
```

### 5. 域名查询

```bash
# 查询域名关联信息
quake domain 360.cn

# 指定字段和翻页
quake domain 360.cn -t ip,port,domain,title --start 10 --size 10

# 仅查看匹配结果数量
quake domain 360.cn -c 1

# 导出结果
quake domain 360.cn -o domain_result.txt
```

### 6. 蜜罐识别

```bash
quake honeypot 93.89.146.23
```

### 7. 智能搜索（GPT）

使用自然语言描述需求，AI 自动转换为 Quake 语法并执行搜索：

```bash
# 搜索返回包里含有 admin 的数据
quake gpt '搜索返回包里里面有admin'

# 查询特定地区和类型
quake gpt '来一打中国江西apache服务器数据'

# 指定时间范围和排除条件
quake gpt '来20个不要来自台湾的apache服务器数据从2022年1月到2023年1月'

# 指定数量并导出
quake gpt '来20个河南的linux服务器数据从2021年到2022年导出到当前目录下a.txt'
```

GPT 子命令也支持等价的选项参数：

```bash
quake gpt '搜索Apache服务器' --size 50
```

GPT 返回的语法中可嵌入 `--size`、`--time_start`、`--time_end`、`--output` 等指令，CLI 会自动解析并执行后续搜索。

> GPT 功能处于训练测试阶段，输出结果可能不完全符合预期。欢迎反馈和 PR 改进。

## 输出字段参考

### search 字段（`-t` 参数）

默认输出 `ip,port,title`。可通过 `-t` 自由组合：

| 字段 | 说明 |
|------|------|
| `ip` | IP 地址 |
| `port` | 端口 |
| `title` | 站点标题 |
| `product_name_cn` | 产品名称（如 Apache、Nginx） |
| `version` | 软件版本号 |
| `protocol` | 服务协议（如 http、ssh） |
| `country` | 国家 |
| `province` | 省份 |
| `city` | 城市 |
| `owner` | 运营商/所有者 |
| `time` | 数据更新时间 |
| `ssldomain` | SSL 证书中的域名 |
| `domain` | HTTP 请求中的 Host 域名 |

### host 字段（`-t` 参数）

| 字段 | 说明 |
|------|------|
| `ip` | IP 地址 |
| `port` | 端口 |
| `title` | 站点标题 |
| `country` | 国家 |
| `province` | 省份 |
| `city` | 城市 |
| `owner` | 运营商 |
| `time` | 更新时间 |
| `ssldomain` | SSL 证书域名 |

### domain 字段（`-t` 参数）

| 字段 | 说明 |
|------|------|
| `domain` | 关联域名 |
| `ip` | 域名解析到的 IP |
| `port` | 端口 |
| `title` | 站点标题 |

## 子命令选项详解

### search（资产搜索）

| 选项 | 说明 |
|------|------|
| `-o, --output` | 导出结果到文件（追加模式） |
| `-q, --query_file` | 从文件读取查询语句（逐行 OR 连接，走滚动分页拉取全量） |
| `-u, --upload` | 上传 IP 列表文件（不超过 1000 条） |
| `-s, --start_time` | 搜索开始时间，格式 `YYYY-MM-DD` |
| `-e, --end_time` | 搜索结束时间，格式 `YYYY-MM-DD` |
| `--size` | 返回条数（最大 100，默认 10） |
| `--start` | 分页起始位置（默认 0） |
| `-f, --filter` | 正则过滤，匹配响应体 / cert / header 内容 |
| `-c, --cdn` | 设为 `1` 排除 CDN 数据 |
| `-m, --honey_jar` | 设为 `1` 排除蜜罐数据 |
| `-l, --latest_data` | 设为 `1` 仅显示最新数据（默认已启用） |
| `-r, --filter_request` | 设为 `1` 过滤无效请求（400/401/403 等） |
| `-d, --deduplication` | 设为 `1` 对结果去重 |

> 使用 `-q` 从文件批量读取时，自动启用滚动分页（scroll）模式，一次性拉取所有匹配结果，不受 `--start` 限制。

### host（IP 查询）

| 选项 | 说明 |
|------|------|
| `-o, --output` | 导出结果到文件（追加模式） |
| `-q, --query_host_file` | 从文件批量查询 IP（每行一个 IP，自动滚动分页） |
| `-t, --type` | 显示字段（默认 `ip,port`） |
| `--size` | 返回条数（最大 100，默认 10） |
| `--start` | 分页起始位置（默认 0） |

### domain（域名查询）

| 选项 | 说明 |
|------|------|
| `-o, --output` | 导出结果到文件 |
| `-t, --type` | 显示字段（默认 `ip,port,domain`） |
| `-c, --count` | 设为 `1` 仅显示匹配数量，不列出详情 |
| `-n, --cdn` | 设为 `1` 排除 CDN 数据 |
| `-m, --honey_jar` | 设为 `1` 排除蜜罐数据 |
| `-l, --latest_data` | 设为 `1` 仅显示最新数据 |
| `-r, --filter_request` | 设为 `1` 过滤无效请求 |
| `-d, --deduplication` | 设为 `1` 对结果去重 |
| `--size` | 返回条数（最大 100，默认 10） |
| `--start` | 分页起始位置（默认 0） |

### info（账户信息）

直接运行即可，显示用户名、邮箱、手机号、月度积分、长效积分和角色权限。

### honeypot（蜜罐识别）

| 选项 | 说明 |
|------|------|
| `<ip>` | 要检测的 IP 地址（必填） |

### gpt（智能搜索）

| 选项 | 说明 |
|------|------|
| `<query>` | 自然语言搜索描述（必填） |
| `--size` | 返回条数（最大 100，默认 10） |

## 时间范围约定

当不指定 `-s` / `-e` 时，默认搜索最近一年的数据。时间范围逻辑：

| `-s` | `-e` | 实际范围 |
|------|------|---------|
| 未指定 | 未指定 | 一年前 → 当前时间 |
| 指定 | 未指定 | 指定日期 → 当前时间 |
| 未指定 | 指定 | 指定日期一年前 → 指定日期 |
| 指定 | 指定 | 指定开始 → 指定结束 |

## 日志与调试

设置 `RUST_LOG` 环境变量可开启调试日志：

```bash
RUST_LOG=debug quake search 'port:80'
```

## 输出行为

### 终端输出（默认）

不带 `-o` 运行子命令时，结果打印到终端，末尾附带当前账户月度积分。

### 文件导出（`-o`）

使用 `-o filename` 时，结果以追加模式写入文件，不输出积分信息。支持多次查询追加到同一文件。

## 常见问题

### 搜索无结果？

- 检查 API Key 是否有效：`quake info`
- 确认语法正确：`quake search 'port:80'` 是最基础的测试查询
- 确认所选时间范围内存在数据
- 开启调试日志定位问题：`RUST_LOG=debug quake search 'port:80'`

### 如何导出全量结果？

- search 使用 `-q query.txt` 从文件读取，自动启用滚动模式拉取全部数据
- host 使用 `-q hosts.txt` 从文件读取 IP，同样走滚动模式，不限制条数

### IP/CIDR 支持什么格式？

支持 IPv4 单 IP、IPv4 CIDR 段（如 `/24`）以及 IPv6 地址。

### 无法连接到 API？

- 确认网络可正常访问 `quake.360.net`
- 检查 API Key 是否正确配置：`quake info`
- 首次运行建议从简单的 `quake search 'port:80'` 开始排查

## 完整帮助

```bash
quake --help            # 查看所有子命令
quake search --help     # 查看 search 子命令选项
quake host --help       # 查看 host 子命令选项
quake domain --help     # 查看 domain 子命令选项
```

## 开发

### 项目结构

本项目采用分层架构：

| 层 | 模块 | 职责 |
|-----|------|------|
| 入口 | `main.rs` | 初始化（env_logger）+ 调用 CLI 入口 |
| CLI | `cli` | 子命令定义（clap builder 模式）+ 参数解析 + 调度 |
| 展示 | `display` | 终端格式化输出，纯函数 |
| 持久化 | `persistence` | 文件读写，结果保存 + 查询文件读取 |
| 客户端 | `client` | Quake API HTTP 客户端，封装所有请求 |
| 模型 | `models` | 数据结构体、Output 工具、时间工具函数 |
| 密钥管理 | `api` | API Key 存取与校验 |
| GPT | `gpt` | OpenAI API 客户端，自然语言 → Quake 语法 |

### 编码规范

- Rust 2018 Edition，稳定版工具链
- **展示与业务分离**：`display` 模块为纯函数，不发起 API 请求
- **持久化集中**：所有文件 I/O 集中在 `persistence` 模块
- **CLI 仅编排**：`cli` 层不含业务逻辑，仅负责参数解析和模块调度
- **构建优化**：Release 启用 LTO、大小优化（`opt-level = 'z'`）、单 codegen unit

### 运行测试

```bash
cargo test
```

### 构建 Release

```bash
cargo build --release
```

## 链接

- [GitHub 仓库](https://github.com/360quake/quake_rs)
- [Quake 网络空间测绘系统](https://quake.360.net)
- [Quake 个人中心](https://quake.360.net/centre/user-info)
- [系统设计文档](docs/system_design.md)

## 问题反馈

- GitHub Issues：[提交 Bug 或功能请求](https://github.com/360quake/quake_rs/issues)
- 技术交流：添加微信 `quake_360` 邀您加入交流群

## 更新日志

参见 [CHANGELOG.md](CHANGELOG.md)。

## 安全策略

参见 [SECURITY.md](SECURITY.md)。
