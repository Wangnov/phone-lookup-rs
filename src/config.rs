use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "phone-lookup-rs".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 0, // 0 = auto detect
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: "phone.dat".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size: 1000,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
        }
    }
}

impl Config {
    /// 加载配置文件
    /// 
    /// 按以下优先级加载配置：
    /// 1. 默认配置
    /// 2. config.toml 文件（如果存在）
    /// 3. 环境变量（前缀：PHONE_DATA）
    /// 
    /// # 返回值
    /// 
    /// 返回加载并验证后的配置实例
    /// 
    /// # 错误
    /// 
    /// 当配置文件格式错误或配置验证失败时返回错误
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let mut settings =
            config::Config::builder().add_source(config::Config::try_from(&Config::default())?);

        // 尝试加载配置文件
        if Path::new("config.toml").exists() {
            settings = settings.add_source(config::File::with_name("config"));
            tracing::info!("已加载配置文件: config.toml");
        } else {
            tracing::info!("未找到配置文件，使用默认配置");
        }

        // 环境变量覆盖
        settings = settings.add_source(
            config::Environment::with_prefix("PHONE_DATA")
                .prefix_separator("_")
                .separator("__"),
        );

        let config: Config = settings.build()?.try_deserialize()?;
        
        // 验证配置
        config.validate()?;

        tracing::info!("配置加载完成: {:?}", config);
        Ok(config)
    }

    /// 验证配置参数的有效性
    /// 
    /// # 错误
    /// 
    /// 当配置参数无效时返回描述性错误信息
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 验证服务器配置
        if self.server.port == 0 {
            return Err("服务器端口不能为0".into());
        }

        // 验证数据库配置
        if self.database.path.is_empty() {
            return Err("数据库文件路径不能为空".into());
        }
        if !Path::new(&self.database.path).exists() {
            return Err(format!("数据库文件不存在: {}", self.database.path).into());
        }

        // 验证缓存配置
        if self.cache.max_size == 0 && self.cache.enabled {
            return Err("启用缓存时，缓存大小不能为0".into());
        }
        if self.cache.max_size > 1_000_000 {
            tracing::warn!("缓存大小过大({}),可能影响内存使用", self.cache.max_size);
        }

        // 验证日志配置
        let valid_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(format!("无效的日志级别: {}, 有效值: {:?}", 
                              self.logging.level, valid_levels).into());
        }

        let valid_formats = ["json", "pretty"];
        if !valid_formats.contains(&self.logging.format.as_str()) {
            return Err(format!("无效的日志格式: {}, 有效值: {:?}", 
                              self.logging.format, valid_formats).into());
        }

        Ok(())
    }
}
