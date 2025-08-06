//! Tauri 命令包装器
//! 
//! 将现有的手机号查询逻辑包装为 Tauri 命令，实现前后端通信

#[cfg(feature = "tauri-app")]
use std::sync::Arc;
#[cfg(feature = "tauri-app")]
use tauri::State;
#[cfg(feature = "tauri-app")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "tauri-app")]
use crate::{PhoneData, PhoneNoInfo};

/// 批量查询结果结构
#[cfg(feature = "tauri-app")]
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchQueryResult {
    /// 手机号
    pub phone: String,
    /// 结果索引（保证顺序）
    pub index: usize,
    /// 查询结果
    pub result: Option<PhoneNoInfo>,
    /// 错误信息
    pub error: Option<String>,
}

/// 应用信息结构
#[cfg(feature = "tauri-app")]
#[derive(Debug, Serialize, Deserialize)]
pub struct AppInfo {
    /// 应用名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 作者
    pub author: String,
    /// 数据库统计信息
    pub database_info: DatabaseInfo,
}

/// 数据库信息
#[cfg(feature = "tauri-app")]
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseInfo {
    /// 记录总数
    pub total_records: usize,
    /// 缓存大小
    pub cache_size: usize,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 总查询次数
    pub total_queries: u64,
}

/// 单个手机号查询命令
/// 
/// # 参数
/// - phone: 手机号字符串
/// - data: PhoneData 状态
/// 
/// # 返回
/// 成功时返回 PhoneNoInfo，失败时返回错误字符串
#[cfg(feature = "tauri-app")]
#[tauri::command]
pub async fn query_phone(
    phone: String,
    data: State<'_, Arc<PhoneData>>
) -> Result<PhoneNoInfo, String> {
    eprintln!("[DEBUG] 收到查询请求: {}", phone);
    log::info!("查询手机号: {}", phone);
    
    match data.find(&phone) {
        Ok(info) => {
            eprintln!("[DEBUG] 查询成功: {} -> {:?}", phone, info);
            log::debug!("查询成功: {} -> {:?}", phone, info);
            Ok(info)
        }
        Err(e) => {
            eprintln!("[DEBUG] 查询失败: {} -> {}", phone, e);
            log::warn!("查询失败: {} -> {}", phone, e);
            Err(format!("查询失败: {}", e))
        }
    }
}

/// 批量手机号查询命令
/// 
/// # 参数
/// - phones: 手机号数组
/// - data: PhoneData 状态
/// 
/// # 返回
/// 批量查询结果数组
#[cfg(feature = "tauri-app")]
#[tauri::command]
pub async fn query_phones_batch(
    phones: Vec<String>,
    data: State<'_, Arc<PhoneData>>
) -> Result<Vec<BatchQueryResult>, String> {
    use futures::stream::{self, StreamExt};
    
    log::info!("批量查询手机号，数量: {}", phones.len());
    
    if phones.is_empty() {
        return Ok(Vec::new());
    }
    
    if phones.len() > 100 {
        return Err("批量查询最多支持100个手机号".to_string());
    }
    
    let data_ref = data.inner().clone();
    
    // 使用 futures::stream::buffered 并发处理，保证结果顺序
    let results: Vec<BatchQueryResult> = stream::iter(phones.into_iter().enumerate())
        .map(|(index, phone)| {
            let data_clone = data_ref.clone();
            let phone_clone = phone.clone();
            
            async move {
                match data_clone.find(&phone_clone) {
                    Ok(info) => BatchQueryResult {
                        phone: phone_clone,
                        index,
                        result: Some(info),
                        error: None,
                    },
                    Err(e) => BatchQueryResult {
                        phone: phone_clone,
                        index,
                        result: None,
                        error: Some(e.to_string()),
                    },
                }
            }
        })
        .buffered(100) // 最多100个并发查询
        .collect()
        .await;
    
    log::info!("批量查询完成，处理数量: {}", results.len());
    Ok(results)
}

/// 获取应用信息命令
#[cfg(feature = "tauri-app")]
#[tauri::command]
pub async fn get_app_info(
    data: State<'_, Arc<PhoneData>>
) -> Result<AppInfo, String> {
    eprintln!("[DEBUG] 收到获取应用信息请求");
    log::info!("获取应用信息");
    
    // 获取缓存统计信息
    let cache_stats = data.get_cache_stats();
    
    let app_info = AppInfo {
        name: "手机号归属地查询工具".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        author: "Wangnov".to_string(),
        database_info: DatabaseInfo {
            total_records: data.get_total_records(),
            cache_size: cache_stats.size,
            cache_hits: cache_stats.hits,
            total_queries: cache_stats.total_queries,
        },
    };
    
    eprintln!("[DEBUG] 返回应用信息: {:?}", app_info);
    Ok(app_info)
}

/// 清空缓存命令
#[cfg(feature = "tauri-app")]
#[tauri::command]
pub async fn clear_cache(
    data: State<'_, Arc<PhoneData>>
) -> Result<String, String> {
    log::info!("清空缓存");
    
    match data.clear_cache() {
        Ok(_) => Ok("缓存已清空".to_string()),
        Err(e) => Err(format!("清空缓存失败: {}", e)),
    }
}

/// 设置缓存大小命令
#[cfg(feature = "tauri-app")]
#[tauri::command] 
pub async fn set_cache_size(
    size: usize,
    data: State<'_, Arc<PhoneData>>
) -> Result<String, String> {
    log::info!("设置缓存大小: {}", size);
    
    if size > 100000 {
        return Err("缓存大小不能超过100000".to_string());
    }
    
    match data.set_cache_size(size) {
        Ok(_) => Ok(format!("缓存大小已设置为: {}", size)),
        Err(e) => Err(format!("设置缓存大小失败: {}", e)),
    }
}

#[cfg(feature = "tauri-app")]
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_query_phone_command() {
        // 注意: 这里需要实际的 PhoneData 实例来测试
        // 在实际测试中，你可能需要使用模拟数据或测试数据库
    }
    
    #[tokio::test]
    async fn test_batch_query_empty() {
        // 测试空数组的批量查询
        // 这个测试不需要实际的数据库连接
    }
}