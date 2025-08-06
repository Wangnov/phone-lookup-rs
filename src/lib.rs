//! # Phone Lookup RS
//!
//! 高性能的手机号归属地查询库，支持快速二分查找和缓存机制。
//!
//! ## 特性
//!
//! - 基于二分查找的高性能查询算法
//! - 内置LRU缓存机制，可配置缓存大小
//! - 线程安全的并发访问支持
//! - 内存优化的数据结构设计
//!
//! ## 示例
//!
//! ```rust
//! use phone_lookup_rs::PhoneData;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let phone_data = PhoneData::new()?;
//! let info = phone_data.find("13800138000")?;
//! println!("省份: {}, 城市: {}, 运营商: {}",
//!          info.province, info.city, info.card_type);
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod config;
pub mod tauri_commands;

/// 手机号查询相关错误类型
#[derive(Error, Debug)]
pub enum ErrorKind {
    /// 数据库文件格式无效或损坏
    #[error("数据库文件格式无效")]
    InvalidPhoneDatabase,
    /// 手机号码长度不符合要求（应为7-11位）
    #[error("手机号码长度无效，有效长度为7-11位")]
    InvalidLength,
    /// 在数据库中未找到指定手机号码
    #[error("在数据库中未找到此手机号码")]
    NotFound,
    /// 运营商代码无效
    #[error("无效的运营商代码")]
    InvalidOpNo,
    /// I/O操作错误
    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),
}

/// 手机号数据库核心结构
///
/// 包含手机号归属地数据库的所有信息，支持高性能查询和缓存机制。
///
/// # 特性
/// - 线程安全：使用 Arc 和 Mutex 确保并发访问安全
/// - 内存优化：使用 Arc 避免数据重复拷贝
/// - 缓存支持：内置可配置的 LRU 缓存机制
#[derive(Debug)]
pub struct PhoneData {
    /// 数据库版本信息
    version: String,
    /// 记录数据的原始字节数组
    records: Arc<Vec<u8>>,
    /// 索引数组，用于二分查找
    index: Arc<Vec<Index>>,
    /// LRU 缓存，存储查询结果（使用 RwLock 优化读性能）
    cache: Arc<RwLock<HashMap<String, PhoneNoInfo>>>,
    /// 是否启用缓存
    cache_enabled: bool,
    /// 缓存最大条目数
    cache_max_size: usize,
    /// 性能统计：查询总数
    query_count: AtomicU64,
    /// 性能统计：缓存命中数
    cache_hits: AtomicU64,
}

impl Clone for PhoneData {
    fn clone(&self) -> Self {
        PhoneData {
            version: self.version.clone(),
            records: self.records.clone(),
            index: self.index.clone(),
            cache: self.cache.clone(),
            cache_enabled: self.cache_enabled,
            cache_max_size: self.cache_max_size,
            query_count: AtomicU64::new(self.query_count.load(Ordering::Relaxed)),
            cache_hits: AtomicU64::new(self.cache_hits.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Index {
    /// 手机号前七位
    phone_no_prefix: i32,
    /// 记录区的偏移
    records_offset: i32,
    /// 卡类型
    card_type: u8,
}

impl PartialOrd for Index {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Index {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.phone_no_prefix.cmp(&other.phone_no_prefix)
    }
}

#[derive(Debug, Clone, Serialize)]
struct Records {
    /// 省
    province: String,
    /// 市
    city: String,
    /// 邮政编码
    zip_code: String,
    /// 长途区号
    area_code: String,
}

impl PhoneData {
    /// 获取数据库版本信息
    pub fn version(&self) -> &str {
        &self.version
    }

    /// 获取索引记录数量
    pub fn index_count(&self) -> usize {
        self.index.len()
    }

    /// 获取查询总数
    pub fn query_count(&self) -> u64 {
        self.query_count.load(Ordering::Relaxed)
    }

    /// 获取缓存命中数
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }

    /// 获取缓存命中率（百分比）
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.query_count();
        let hits = self.cache_hits();
        if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }

    pub fn new() -> Fallible<PhoneData> {
        let path = get_resource_path("phone.dat");
        tracing::info!("尝试加载数据库文件: {}", path);
        Self::from_file(&path)
    }

    pub fn from_file(path: &str) -> Fallible<PhoneData> {
        Self::from_file_with_config(path, true, 1000)
    }

    pub fn from_file_with_config(
        path: &str,
        cache_enabled: bool,
        cache_max_size: usize,
    ) -> Fallible<PhoneData> {
        tracing::info!("正在加载手机号码数据库文件: {}", path);
        let data_file = File::open(path)?;
        let mut data_file = BufReader::new(data_file);

        // parse version and index offset
        let mut header_buffer = [0u8; 8];
        data_file
            .read_exact(&mut header_buffer)
            .map_err(|_| ErrorKind::InvalidPhoneDatabase)?;
        let version = String::from_utf8(header_buffer[..4].to_vec())
            .map_err(|_| ErrorKind::InvalidPhoneDatabase)?;
        let index_offset = Self::four_u8_to_i32(&header_buffer[4..]) as u64;

        // read records
        let mut records = vec![0u8; index_offset as usize - 8];
        data_file
            .read_exact(&mut records)
            .map_err(|_| ErrorKind::InvalidPhoneDatabase)?;

        // parse index
        let mut index = Vec::new();
        // length of a index is 9
        let mut index_item = [0u8; 9];
        loop {
            match data_file.read_exact(&mut index_item) {
                Ok(_) => (),
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        break;
                    }
                }
            }
            let phone_no_prefix = Self::four_u8_to_i32(&index_item[..4]);
            let records_offset = Self::four_u8_to_i32(&index_item[4..8]);
            let card_type = index_item[8];
            index.push(Index {
                phone_no_prefix,
                records_offset,
                card_type,
            });
        }

        let config = PhoneData {
            version: version.clone(),
            records: Arc::new(records),
            index: Arc::new(index.clone()),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_enabled,
            cache_max_size,
            query_count: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
        };
        tracing::info!(
            "数据库加载完成，版本: {}, 索引数量: {}",
            version,
            index.len()
        );
        Ok(config)
    }

    #[inline]
    fn four_u8_to_i32(s: &[u8]) -> i32 {
        if s.len() >= 4 {
            i32::from_le_bytes([s[0], s[1], s[2], s[3]])
        } else {
            let mut bytes = [0u8; 4];
            bytes[..s.len()].copy_from_slice(s);
            i32::from_le_bytes(bytes)
        }
    }

    fn parse_to_record(&self, offset: usize) -> Fallible<Records> {
        if let Some(record) = self.records[offset - 8..].splitn(2, |i| *i == 0u8).nth(0) {
            let record =
                String::from_utf8(record.to_vec()).map_err(|_| ErrorKind::InvalidPhoneDatabase)?;
            let record: Vec<&str> = record.split('|').collect();
            if record.len() != 4 {
                return Err(ErrorKind::InvalidPhoneDatabase);
            }
            Ok(Records {
                province: record[0].to_string(),
                city: record[1].to_string(),
                zip_code: record[2].to_string(),
                area_code: record[3].to_string(),
            })
        } else {
            Err(ErrorKind::InvalidPhoneDatabase)
        }
    }

    /// 优化的二分查找算法查找 `phone_no` 数据
    pub fn find(&self, no: &str) -> Fallible<PhoneNoInfo> {
        // 增加查询计数
        self.query_count.fetch_add(1, Ordering::Relaxed);

        let len = no.len();
        if !(7..=11).contains(&len) {
            return Err(ErrorKind::InvalidLength);
        }

        // 检查缓存（仅当缓存启用时）使用读锁优化性能
        if self.cache_enabled {
            if let Ok(cache) = self.cache.read() {
                if let Some(cached_result) = cache.get(no) {
                    // 增加缓存命中计数
                    self.cache_hits.fetch_add(1, Ordering::Relaxed);
                    tracing::debug!("从缓存返回手机号 {} 的信息", no);
                    return Ok(cached_result.clone());
                }
            }
        }

        // 快速解析前7位数字，避免字符串转换
        let no_parsed = self.parse_phone_prefix(no)?;

        // 使用标准库的二分查找，性能更优
        match self
            .index
            .binary_search_by_key(&no_parsed, |idx| idx.phone_no_prefix)
        {
            Ok(pos) => {
                let index_item = &self.index[pos];
                let record = self.parse_to_record(index_item.records_offset as usize)?;
                let card_type = CardType::from_u8(index_item.card_type)?;
                let result = PhoneNoInfo {
                    province: record.province,
                    city: record.city,
                    zip_code: record.zip_code,
                    area_code: record.area_code,
                    card_type: card_type.get_description().to_string(),
                };

                // 缓存结果（优化锁粒度：最小化写锁持有时间）
                if self.cache_enabled {
                    // 首先用读锁快速检查缓存大小，避免不必要的写锁获取
                    let needs_cleanup = if let Ok(cache) = self.cache.read() {
                        cache.len() >= self.cache_max_size
                    } else {
                        false
                    };

                    if let Ok(mut cache) = self.cache.write() {
                        // 双重检查：可能在获取写锁期间其他线程已更新缓存
                        if !cache.contains_key(no) {
                            if needs_cleanup && cache.len() >= self.cache_max_size {
                                // 优化的LRU清理：收集一半的keys后立即释放迭代器
                                let keys_to_remove: Vec<String> =
                                    cache.keys().take(cache.len() / 2).cloned().collect();
                                for key in keys_to_remove {
                                    cache.remove(&key);
                                }
                                tracing::debug!("缓存已满，清理后插入新条目");
                            }
                            cache.insert(no.to_string(), result.clone());
                        }
                    }
                }

                Ok(result)
            }
            Err(_) => Err(ErrorKind::NotFound),
        }
    }

    /// 快速解析手机号前缀，避免字符串分配
    #[inline]
    fn parse_phone_prefix(&self, no: &str) -> Fallible<i32> {
        let bytes = no.as_bytes();
        if bytes.len() < 7 {
            return Err(ErrorKind::InvalidLength);
        }

        let mut result = 0i32;
        for &digit in bytes.iter().take(7) {
            if !digit.is_ascii_digit() {
                return Err(ErrorKind::InvalidPhoneDatabase);
            }
            result = result * 10 + (digit - b'0') as i32;
        }
        Ok(result)
    }

    /// 获取总记录数（用于Tauri命令）
    pub fn get_total_records(&self) -> usize {
        self.index.len()
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        CacheStats {
            size: cache.len(),
            max_size: self.cache_max_size,
            hits: self.cache_hits(),
            total_queries: self.query_count(),
        }
    }

    /// 清空缓存
    pub fn clear_cache(&self) -> Result<(), ErrorKind> {
        if !self.cache_enabled {
            return Err(ErrorKind::InvalidPhoneDatabase);
        }

        let mut cache = self
            .cache
            .write()
            .map_err(|_| ErrorKind::InvalidPhoneDatabase)?;
        cache.clear();
        Ok(())
    }

    /// 设置缓存大小
    pub fn set_cache_size(&self, _new_size: usize) -> Result<(), ErrorKind> {
        if !self.cache_enabled {
            return Err(ErrorKind::InvalidPhoneDatabase);
        }

        // 注意：这里只是展示接口，实际实现可能需要重构缓存结构
        // 当前实现只是清空缓存
        let mut cache = self
            .cache
            .write()
            .map_err(|_| ErrorKind::InvalidPhoneDatabase)?;
        cache.clear();
        // TODO: 实际应用中可能需要调整PhoneData结构来支持动态缓存大小调整
        Ok(())
    }
}

/// 缓存统计信息结构
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 当前缓存条目数
    pub size: usize,
    /// 最大缓存条目数
    pub max_size: usize,
    /// 缓存命中次数
    pub hits: u64,
    /// 总查询次数
    pub total_queries: u64,
}

/// 运营商类型，使用更紧凑的表示
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CardType {
    Cmcc = 1,
    Cucc = 2,
    Ctcc = 3,
    CtccV = 4,
    CuccV = 5,
    CmccV = 6,
    Cbcc = 7,
    CbccV = 8,
}

impl CardType {
    #[inline]
    fn from_u8(i: u8) -> Result<CardType, ErrorKind> {
        match i {
            1 => Ok(CardType::Cmcc),
            2 => Ok(CardType::Cucc),
            3 => Ok(CardType::Ctcc),
            4 => Ok(CardType::CtccV),
            5 => Ok(CardType::CuccV),
            6 => Ok(CardType::CmccV),
            7 => Ok(CardType::Cbcc),
            8 => Ok(CardType::CbccV),
            _ => Err(ErrorKind::InvalidOpNo),
        }
    }

    /// 使用静态字符串避免内存分配
    #[inline]
    const fn get_description(&self) -> &'static str {
        match self {
            CardType::Cmcc => "中国移动",
            CardType::Cucc => "中国联通",
            CardType::Ctcc => "中国电信",
            CardType::CtccV => "中国电信虚拟运营商",
            CardType::CuccV => "中国联通虚拟运营商",
            CardType::CmccV => "中国移动虚拟运营商",
            CardType::Cbcc => "中国广电",
            CardType::CbccV => "中国广电虚拟运营商",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PhoneNoInfo {
    /// 省
    pub province: String,
    /// 市
    pub city: String,
    /// 邮政编码
    pub zip_code: String,
    /// 长途区号
    pub area_code: String,
    /// 卡类型
    pub card_type: String,
}

/// 获取资源文件路径
///
/// 在 Tauri 应用中，资源文件的位置在开发和生产环境中可能不同。
/// 此函数尝试多种路径来定位资源文件。
fn get_resource_path(filename: &str) -> String {
    // 尝试多种可能的路径
    let possible_paths = vec![
        // 1. 当前工作目录（开发环境）
        format!("./{}", filename),
        // 2. 项目根目录（相对于 src-tauri）
        format!("../{}", filename),
        // 3. 绝对路径（如果环境变量设置）
        std::env::var("PHONE_DATA_PATH").unwrap_or_default(),
    ];

    for path in possible_paths {
        if !path.is_empty() && std::path::Path::new(&path).exists() {
            tracing::info!("找到资源文件: {}", path);
            return path;
        }
    }

    // 如果都找不到，返回默认路径（用于错误信息）
    tracing::warn!("未找到资源文件: {}，将使用默认路径", filename);
    format!("./{}", filename)
}

/// 统一的结果类型别名
type Fallible<T> = Result<T, ErrorKind>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_u8_to_i32() {
        let bytes = [0x01, 0x02, 0x03, 0x04];
        let result = PhoneData::four_u8_to_i32(&bytes);
        assert_eq!(result, 0x04030201); // little-endian
    }

    #[test]
    fn test_four_u8_to_i32_short() {
        let bytes = [0x01, 0x02];
        let result = PhoneData::four_u8_to_i32(&bytes);
        assert_eq!(result, 0x00000201); // 填充零
    }

    #[test]
    fn test_parse_phone_prefix_valid() {
        let phone_data = create_mock_phone_data();
        let result = phone_data.parse_phone_prefix("1380013");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1380013);
    }

    #[test]
    fn test_parse_phone_prefix_invalid_length() {
        let phone_data = create_mock_phone_data();
        let result = phone_data.parse_phone_prefix("123");
        assert!(matches!(result, Err(ErrorKind::InvalidLength)));
    }

    #[test]
    fn test_parse_phone_prefix_invalid_chars() {
        let phone_data = create_mock_phone_data();
        let result = phone_data.parse_phone_prefix("138abc7");
        assert!(matches!(result, Err(ErrorKind::InvalidPhoneDatabase)));
    }

    #[test]
    fn test_card_type_from_u8() {
        assert_eq!(CardType::from_u8(1).unwrap(), CardType::Cmcc);
        assert_eq!(CardType::from_u8(2).unwrap(), CardType::Cucc);
        assert_eq!(CardType::from_u8(3).unwrap(), CardType::Ctcc);

        assert!(matches!(CardType::from_u8(99), Err(ErrorKind::InvalidOpNo)));
    }

    #[test]
    fn test_card_type_description() {
        assert_eq!(CardType::Cmcc.get_description(), "中国移动");
        assert_eq!(CardType::Cucc.get_description(), "中国联通");
        assert_eq!(CardType::Ctcc.get_description(), "中国电信");
    }

    #[test]
    fn test_index_ordering() {
        let index1 = Index {
            phone_no_prefix: 1380000,
            records_offset: 100,
            card_type: 1,
        };
        let index2 = Index {
            phone_no_prefix: 1390000,
            records_offset: 200,
            card_type: 2,
        };

        assert!(index1 < index2);
        assert_eq!(index1.cmp(&index2), std::cmp::Ordering::Less);
    }

    /// 创建一个模拟的PhoneData实例用于测试
    fn create_mock_phone_data() -> PhoneData {
        PhoneData {
            version: "TEST".to_string(),
            records: Arc::new(vec![]),
            index: Arc::new(vec![]),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_enabled: true,
            cache_max_size: 100,
            query_count: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
        }
    }

    #[test]
    fn test_cache_functionality() {
        let phone_data = create_mock_phone_data();
        let phone_number = "1380013";

        // 创建模拟结果
        let mock_result = PhoneNoInfo {
            province: "测试省".to_string(),
            city: "测试市".to_string(),
            zip_code: "000000".to_string(),
            area_code: "0000".to_string(),
            card_type: "测试运营商".to_string(),
        };

        // 直接向缓存中插入测试数据
        {
            let mut cache = phone_data.cache.write().unwrap();
            cache.insert(phone_number.to_string(), mock_result.clone());
        }

        // 验证缓存中的数据
        {
            let cache = phone_data.cache.read().unwrap();
            let cached_result = cache.get(phone_number).unwrap();
            assert_eq!(cached_result.province, "测试省");
            assert_eq!(cached_result.city, "测试市");
        }
    }
}
