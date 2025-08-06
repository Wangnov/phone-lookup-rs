![Phone Lookup RS](assets/banner.png)

<div align="center">

[![Version](https://img.shields.io/badge/version-v1.0.0-blue.svg?style=for-the-badge)](https://github.com/wangnov/phone-lookup-rs)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg?style=for-the-badge)](LICENSE)
[![Actix Web](https://img.shields.io/badge/Actix_Web-4.11-blue.svg?style=for-the-badge)](https://actix.rs/)
[![Performance](https://img.shields.io/badge/Response_Time-<1ms-brightgreen.svg?style=for-the-badge)](#-性能指标)

[![GitHub Stars](https://img.shields.io/github/stars/wangnov/phone-lookup-rs?style=social)](https://github.com/wangnov/phone-lookup-rs/stargazers)
[![GitHub Forks](https://img.shields.io/github/forks/wangnov/phone-lookup-rs?style=social)](https://github.com/wangnov/phone-lookup-rs/network/members)
[![GitHub Issues](https://img.shields.io/github/issues/wangnov/phone-lookup-rs?style=social)](https://github.com/wangnov/phone-lookup-rs/issues)

</div>

---

## 🌟 项目概述

**Phone Lookup RS** 是一个基于 Rust 开发的高性能手机号归属地查询服务，提供 **三端统一部署方案**：

- 🖥️ **API 服务**：轻量级 REST API 服务，适合集成到现有系统
- 🌐 **Web 应用**：现代化 Web 界面，支持单查询和批量查询
- 📱 **桌面应用**：跨平台桌面客户端，支持 Windows、macOS、Linux

## ✨ 特性

- 🔍 **快速查询**：基于二分查找算法，平均查询时间 < 1ms
- 🧠 **智能缓存**：内置 LRU 缓存，提升热点数据查询性能
- ⚡ **高并发**：基于 Actix Web 框架，支持高并发请求
- 📦 **批量查询**：支持最多100个手机号的并发批量查询
- 📊 **详细日志**：完整的请求日志和性能监控
- 🛡️ **错误处理**：完善的错误处理和中文错误信息
- 🧪 **完整测试**：包含单元测试、性能测试和并发测试
- 🌐 **三端部署**：API / Web / Desktop 统一架构

## 📊 数据统计

- 归属地信息库文件大小：4557kb
- 归属地信息库最后更新：2025年02月
- 手机号段记录条数：517258
- 支持运营商：中国移动、中国联通、中国电信、中国广电及其虚拟运营商

## 🚀 快速开始

### 📋 系统要求

- **Rust**: 1.70+
- **Node.js**: 18+ (仅桌面应用开发需要)
- **操作系统**: Windows / macOS / Linux

### 🔧 安装和配置

```bash
# 克隆项目
git clone https://github.com/wangnov/phone-lookup-rs.git
cd phone-lookup-rs
```

## 🎯 三端部署方案

### 1️⃣ API 服务部署

适用于：后端集成、微服务架构、API 对接

```bash
# 开发模式
cargo run

# 生产模式（推荐）
cargo run --release

# 指定配置文件
PHONE_DATA_CONFIG=./config.toml cargo run --release
```

**API 端点**：
- `GET /query/{phone}` - 单个查询
- `GET /query?phone={phone}` - 查询参数方式
- `POST /batch-query` - 批量查询
- `GET /health` - 健康检查

### 2️⃣ Web 应用部署

适用于：内部系统、用户自助查询、演示展示

```bash
# 1. 构建前端
cd frontend
npm install
npm run build

# 2. 启动 Web 服务
cd ..
cargo run --release
```

**访问地址**：`http://localhost:8080`

**特性**：
- 🎨 现代化 UI 界面
- 📱 响应式设计
- ⚡ 实时查询
- 📊 批量处理
- 📈 缓存统计

### 3️⃣ 桌面应用部署

适用于：离线使用、个人工具、企业内部分发

```bash
# 1. 安装 Tauri CLI
cargo install tauri-cli

# 2. 安装前端依赖
cd frontend
npm install

# 3. 开发模式运行
cargo tauri dev

# 4. 构建生产版本
cargo tauri build
```

**支持平台**：
- 🪟 **Windows**: `.msi` / `.exe` 安装包
- 🍎 **macOS**: `.app` / `.dmg` 应用包
- 🐧 **Linux**: `.deb` / `.rpm` / `.AppImage` 包

**桌面应用特性**：
- 🖱️ 原生界面体验
- 📂 本地数据库存储
- 🚀 快速启动
- 💾 离线可用
- 🔄 自动更新支持

## 📋 配置文件

项目支持通过 `config.toml` 文件进行配置：

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 0  # 0 = 自动检测CPU核心数

[cache]
enabled = true
max_size = 1000

[logging]
level = "info"
```

## 🔌 API 接口

### 单查询接口

```bash
# 路径参数
curl "http://127.0.0.1:8080/query/18086834111"

# 查询参数
curl "http://127.0.0.1:8080/query?phone=18086834111"
```

**响应格式**：
```json
{
    "code": 0,
    "data": {
        "province": "四川",
        "city": "成都",
        "zip_code": "610000",
        "area_code": "028",
        "card_type": "中国电信"
    },
    "success": true,
    "message": "success"
}
```

### 批量查询接口

```bash
curl -X POST "http://127.0.0.1:8080/batch-query" \
  -H "Content-Type: application/json" \
  -d '{
    "phones": [
      "18086834111",
      "13800138000",
      "13900139000"
    ]
  }'
```

**响应格式**：
```json
{
    "code": 0,
    "data": {
        "results": [
            {
                "phone": "18086834111",
                "index": 0,
                "success": true,
                "data": {
                    "province": "四川",
                    "city": "成都",
                    "zip_code": "610000",
                    "area_code": "028",
                    "card_type": "中国电信"
                },
                "error": null
            }
        ],
        "stats": {
            "total": 1,
            "success_count": 1,
            "failed_count": 0,
            "processing_time_ms": 5
        }
    },
    "success": true,
    "message": "success"
}
```

## 🧪 测试

### 运行测试套件

```bash
# 所有测试
cargo test

# 性能测试
cargo test --release -- --nocapture performance_tests

# 批量查询测试
cargo test --release -- --nocapture batch_query_test
```

### 测试覆盖

- ✅ **功能测试**：基本查询功能验证
- ✅ **边界测试**：无效输入、边界条件
- ✅ **性能测试**：10,000 次查询性能基准
- ✅ **缓存测试**：缓存命中性能验证
- ✅ **并发测试**：多线程并发查询安全性
- ✅ **批量测试**：批量查询顺序和性能验证

## 📈 性能指标

| 指标 | 数值 | 说明 |
|------|------|------|
| **查询速度** | < 1ms | 单次查询平均响应时间 |
| **缓存命中** | < 0.1ms | 热点数据查询时间 |
| **并发支持** | 1000+ | 同时连接数 |
| **内存占用** | < 50MB | 包含数据库和缓存 |
| **批量处理** | 100个/次 | 单次批量查询上限 |
| **吞吐量** | 10000+ QPS | 高并发查询性能 |

## 🏗️ 技术架构

### 后端技术栈

- **Web 框架**：Actix Web 4.11.0
- **异步运行时**：Tokio 1.46.1
- **序列化**：Serde 1.0.219
- **日志系统**：Tracing + Tracing-subscriber
- **错误处理**：Thiserror 1.0.69
- **桌面框架**：Tauri 2.7.0

### 前端技术栈

- **框架**：React 19.0.0-rc.1
- **构建工具**：Vite 7.0.4
- **样式框架**：Tailwind CSS 3.4.17
- **图标库**：Lucide React 0.536.0
- **类型支持**：TypeScript 5.8.3

### 数据结构优化

- **索引结构**：内存中的有序数组，支持二分查找
- **缓存策略**：基于 HashMap 的 LRU 缓存
- **内存管理**：使用 Arc<T> 实现零拷贝数据共享
- **并发安全**：使用 RwLock 优化读写性能

## 📂 项目结构

```
phone-lookup-rs/                    # 项目根目录
├── src/                            # Rust 后端源码
│   ├── main.rs                     # API 服务入口
│   ├── lib.rs                      # 核心查询逻辑
│   ├── config.rs                   # 配置管理
│   └── tauri_commands.rs           # Tauri 命令接口
├── src-tauri/                      # Tauri 桌面应用配置
│   ├── src/
│   │   ├── main.rs                 # 桌面应用入口
│   │   └── lib.rs                  # Tauri 应用逻辑
│   ├── tauri.conf.json             # Tauri 配置文件
│   └── Cargo.toml                  # 桌面应用依赖
├── frontend/                       # Web 前端源码
│   ├── src/
│   │   ├── components/             # React 组件
│   │   ├── hooks/                  # React Hooks
│   │   ├── lib/                    # 工具库
│   │   └── App.tsx                 # 主应用组件
│   ├── dist/                       # 构建输出目录
│   ├── package.json                # 前端依赖配置
│   └── vite.config.ts              # Vite 构建配置
├── tests/                          # 测试文件
├── assets/                         # 资源文件
├── phone.dat                       # 数据库文件
├── config.toml                     # 服务配置文件
└── README.md                       # 项目文档
```

## 🐳 Docker 部署

### API 服务容器化

```dockerfile
FROM rust:1.70-alpine as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/phone-lookup-rs /usr/local/bin/
COPY phone.dat config.toml /usr/local/bin/
EXPOSE 8080
CMD ["phone-lookup-rs"]
```

### Docker Compose 部署

```yaml
version: '3.8'
services:
  phone-lookup-api:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./phone.dat:/usr/local/bin/phone.dat
      - ./config.toml:/usr/local/bin/config.toml
    restart: unless-stopped
    
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/conf.d/default.conf
    depends_on:
      - phone-lookup-api
```

## 🔧 高级配置

### 性能调优建议

1. **生产环境优化**
   ```bash
   # 使用 release 模式
   cargo run --release
   
   # 设置环境变量
   export RUST_LOG=info
   export PHONE_DATA_CACHE_SIZE=2000
   ```

2. **并发配置**
   ```toml
   [server]
   workers = 8  # 根据 CPU 核心数调整
   max_connections = 1000
   keep_alive = 30
   ```

3. **缓存优化**
   ```toml
   [cache]
   enabled = true
   max_size = 5000  # 根据内存情况调整
   ttl = 3600       # 缓存过期时间（秒）
   ```

### 监控和日志

```toml
[logging]
level = "info"
format = "json"
file = "/var/log/phone-lookup.log"

[metrics]
enabled = true
endpoint = "/metrics"
```

## 📝 更新日志

### v1.0.0 (2025-08-06) 🎉

**🚀 正式版本发布 - 三端统一部署**

- ✨ **桌面应用**：基于 Tauri 2.7.0 的跨平台桌面客户端
- 🌐 **Web 应用**：React + TypeScript 现代化 Web 界面
- 🔧 **架构升级**：Monorepo 结构，支持 API/Web/Desktop 三端部署
- 📦 **打包优化**：支持 Windows、macOS、Linux 原生安装包
- 🎨 **UI 重构**：深空蓝配色，响应式设计，流畅动画
- 📊 **功能增强**：实时缓存统计、系统监控、批量处理
- 🔄 **资源管理**：智能路径查找，支持不同环境资源加载

### v0.2.0 (2025-08-06)

- 🚀 **批量查询接口优化**：基于专家建议的全面性能优化
- ⚡ **锁粒度优化**：减少缓存写锁竞争，提升并发性能  
- 🔄 **异步流优化**：使用 futures::stream::buffered 替代手动任务管理
- 📊 **响应结构增强**：添加索引字段确保客户端准确映射查询结果
- 🧪 **测试覆盖增强**：新增结果顺序、索引映射和并发压力测试

### v0.1.0 (2025-08-05)

- ✨ 添加智能缓存机制
- 🐛 修复错误处理和日志
- ⚡ 优化查询性能
- 🧪 完善测试覆盖
- 📝 更新文档和配置

## 🎯 路线图

### 短期计划
- [ ] 添加 Prometheus 监控指标
- [ ] 实现更智能的 LRU 缓存算法
- [ ] 添加 API 限流和频控功能
- [x] ~~支持批量查询接口~~ ✅ **已完成**
- [x] ~~桌面应用开发~~ ✅ **已完成**

### 中期计划
- [ ] 数据库热更新功能
- [ ] 实现分布式部署支持
- [ ] 添加用户认证和权限管理
- [ ] 支持自定义数据源

### 长期计划
- [ ] 机器学习号码识别
- [ ] 云服务版本
- [ ] 移动端 App
- [ ] 企业级版本

## 📦 下载和分发

### 桌面应用下载

| 平台 | 下载链接 | 安装包类型 |
|------|----------|------------|
| 🪟 Windows | [下载 .msi](https://github.com/wangnov/phone-lookup-rs/releases) | Windows Installer |
| 🍎 macOS | [下载 .dmg](https://github.com/wangnov/phone-lookup-rs/releases) | macOS Disk Image |
| 🐧 Linux | [下载 .AppImage](https://github.com/wangnov/phone-lookup-rs/releases) | Universal Linux |

### Web 版本体验

- 🌐 **在线演示**：[https://phone-lookup.example.com](https://github.com/wangnov/phone-lookup-rs)
- 📱 **移动适配**：支持手机和平板访问
- 🔗 **API 文档**：[https://phone-lookup.example.com/docs](https://github.com/wangnov/phone-lookup-rs)

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

1. **Fork** 项目
2. **创建**特性分支：`git checkout -b feature/amazing-feature`
3. **提交**更改：`git commit -m 'feat: 添加某个特性'`
4. **推送**分支：`git push origin feature/amazing-feature`
5. **创建** Pull Request

## 🙏 致谢

本项目基于 [kurisu994/phone_data](https://github.com/kurisu994/phone_data) 进行二次开发和优化。

感谢原作者 [kurisu994](https://github.com/kurisu994) 提供的优秀基础实现，包括：
- 数据库文件格式设计
- 基础查询算法实现
- 项目架构设计

在此基础上，本项目进行了以下重要改进：
- 🚀 **性能优化**：使用 RwLock 替代 Mutex，提升并发读性能
- 🧠 **智能缓存**：实现 LRU 缓存机制，显著提升热点数据查询速度  
- 📊 **监控统计**：添加查询统计和缓存命中率监控
- 🛡️ **错误处理**：完善的错误分类和中文错误信息
- 🧪 **测试覆盖**：新增单元测试、性能测试和并发测试
- 📝 **文档完善**：详细的 API 文档和代码注释
- ⚙️ **配置验证**：严格的配置参数验证机制
- 🌐 **三端部署**：API / Web / Desktop 统一架构
- 🎨 **现代界面**：React + Tauri 现代化用户体验

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

---

<div align="center">

**如果这个项目对你有帮助，请给个 ⭐ Star！**

Made with ❤️ by [Wangnov](https://github.com/wangnov)

</div>