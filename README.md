# Phone Lookup RS

🚀 高性能手机号归属地查询服务，基于 Rust 实现，使用优化的二分查找算法和智能缓存机制。

## ✨ 特性

- 🔍 **快速查询**：基于二分查找算法，平均查询时间 < 1ms
- 🧠 **智能缓存**：内置 LRU 缓存，提升热点数据查询性能
- ⚡ **高并发**：基于 Actix Web 框架，支持高并发请求
- 📊 **详细日志**：完整的请求日志和性能监控
- 🛡️ **错误处理**：完善的错误处理和中文错误信息
- 🧪 **完整测试**：包含单元测试、性能测试和并发测试

## 📊 数据统计

- 归属地信息库文件大小：4557kb
- 归属地信息库最后更新：2025年02月
- 手机号段记录条数：517258
- 支持运营商：中国移动、中国联通、中国电信、中国广电及其虚拟运营商

## phone.dat文件格式
```
    | 4 bytes |                     <- phone.dat 版本号（如：1701即17年1月份）
    ------------
    | 4 bytes |                     <-  第一个索引的偏移
    -----------------------
    |  offset - 8            |      <-  记录区
    -----------------------
    |  index                 |      <-  索引区
    -----------------------
```
1. 头部为8个字节，版本号为4个字节，第一个索引的偏移为4个字节；
2. 记录区 中每条记录的格式为"<省份>|<城市>|<邮编>|<长途区号>\0"。 每条记录以'\0'结束；
3. 索引区 中每条记录的格式为"<手机号前七位><记录区的偏移><卡类型>"，每个索引的长度为9个字节；

## 🚀 快速开始

### 安装和运行

```bash
# 克隆项目
git clone https://github.com/wangnov/phone-lookup-rs.git
cd phone-lookup-rs

# 编译和运行
cargo run

# 发布模式运行（推荐生产环境）
cargo run --release
```

### 📋 配置文件

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

### 1. 路径参数查询

```bash
curl "http://127.0.0.1:8080/query/18086834111"
```

### 2. 查询参数查询

```bash
curl "http://127.0.0.1:8080/query?phone=18086834111"
```

### 3. 健康检查

```bash
curl "http://127.0.0.1:8080/health"
```

### 响应格式

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

### 错误响应

```json
{
    "code": -1,
    "data": null,
    "success": false,
    "message": "手机号码未找到"
}
```

## 🧪 测试

### 运行所有测试

```bash
cargo test
```

### 运行性能测试

```bash
cargo test --release -- --nocapture performance_tests
```

### 测试覆盖

- ✅ **功能测试**：基本查询功能
- ✅ **边界测试**：无效长度、格式错误
- ✅ **性能测试**：10,000 次查询性能基准
- ✅ **缓存测试**：缓存命中性能验证
- ✅ **并发测试**：多线程并发查询

## 📈 性能指标

- **查询速度**：单次查询 < 1ms
- **缓存命中**：热点数据查询 < 0.1ms
- **并发支持**：支持数千个并发连接
- **内存占用**：< 50MB（包含数据库和缓存）

## 🏗️ 技术架构

### 核心技术栈

- **Web 框架**：Actix Web 4.11.0
- **异步运行时**：Tokio 1.46.1
- **序列化**：Serde 1.0.219
- **日志系统**：Tracing + Tracing-subscriber
- **错误处理**：Thiserror 1.0.69

### 数据结构优化

- **索引结构**：内存中的有序数组，支持二分查找
- **缓存策略**：基于 HashMap 的简单 LRU 缓存
- **内存管理**：使用 Arc<T> 实现零拷贝数据共享
- **并发安全**：使用 Mutex 保护缓存访问

## 🔧 开发指南

### 项目结构

```
phone-lookup-rs/
├── src/
│   ├── main.rs          # Web 服务入口
│   ├── lib.rs           # 核心查询逻辑
│   └── config.rs        # 配置管理模块
├── tests/
│   └── demo_test.rs     # 测试用例  
├── config.toml          # 配置文件
├── phone.dat           # 数据库文件
├── Cargo.toml          # 依赖配置
├── README.md           # 项目文档
└── .gitignore          # Git 忽略规则
```

### 编译优化

发布版本启用了以下优化：

```toml
[profile.release]
opt-level = 3        # 最高优化级别
lto = true          # 链接时优化
codegen-units = 1   # 单个代码生成单元
panic = "abort"     # 异常时直接终止
strip = true        # 删除调试符号
```

## 🚀 部署建议

### Docker 部署

```dockerfile
FROM rust:alpine as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/phone-lookup-rs /usr/local/bin/
COPY phone.dat /usr/local/bin/
EXPOSE 8080
CMD ["phone-lookup-rs"]
```

### 性能调优

1. **生产环境使用 release 模式**
2. **根据 CPU 核心数调整 workers**
3. **适当调整缓存大小**
4. **启用 gzip 压缩**
5. **配置反向代理（如 Nginx）**

## 📝 更新日志

### v0.1.0 (2025-08-05)
- ✨ 添加智能缓存机制
- 🐛 修复错误处理和日志
- ⚡ 优化查询性能
- 🧪 完善测试覆盖
- 📝 更新文档和配置

## 🎯 路线图

- [ ] 添加 Prometheus 监控指标
- [ ] 实现更智能的 LRU 缓存
- [ ] 添加 API 限流功能
- [ ] 支持批量查询接口
- [ ] 添加数据库热更新功能
- [ ] 实现分布式部署支持

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

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。
