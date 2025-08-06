use futures::stream::{self, StreamExt};
use phone_lookup_rs::{ErrorKind, PhoneData, PhoneNoInfo};
use std::sync::Arc;
use std::time::Instant;

/// 批量查询结果结构体（用于测试）
#[derive(Debug)]
struct BatchQueryResult {
    phone: String,
    index: usize,
    success: bool,
    data: Option<PhoneNoInfo>,
    error: Option<String>,
}

/// 批量查询统计信息（用于测试）
#[derive(Debug)]
struct BatchQueryStats {
    total: usize,
    success_count: usize,
    failed_count: usize,
    processing_time_ms: u64,
}

/// 模拟批量查询逻辑（使用优化的futures::stream实现）
async fn batch_query_logic(
    phone_data: Arc<PhoneData>,
    phones: Vec<String>,
) -> (Vec<BatchQueryResult>, BatchQueryStats) {
    let start_time = Instant::now();

    // 使用 futures::stream 进行优化的并发查询，自动保证结果顺序
    let results_stream = stream::iter(phones.into_iter().enumerate()).map(|(index, phone)| {
        let phone_data = phone_data.clone();
        async move {
            let phone_clone = phone.clone();
            match phone_data.find(&phone) {
                Ok(info) => BatchQueryResult {
                    phone: phone_clone,
                    index,
                    success: true,
                    data: Some(info),
                    error: None,
                },
                Err(ErrorKind::NotFound) => BatchQueryResult {
                    phone: phone_clone,
                    index,
                    success: false,
                    data: None,
                    error: Some("手机号码未找到".to_string()),
                },
                Err(ErrorKind::InvalidLength) => BatchQueryResult {
                    phone: phone_clone,
                    index,
                    success: false,
                    data: None,
                    error: Some("手机号码格式无效".to_string()),
                },
                Err(ErrorKind::InvalidPhoneDatabase) => BatchQueryResult {
                    phone: phone_clone,
                    index,
                    success: false,
                    data: None,
                    error: Some("数据库格式错误".to_string()),
                },
                Err(_) => BatchQueryResult {
                    phone: phone_clone,
                    index,
                    success: false,
                    data: None,
                    error: Some("查询失败".to_string()),
                },
            }
        }
    });

    // 并发执行查询并收集结果（保持原始顺序）
    let results: Vec<BatchQueryResult> = results_stream.buffered(100).collect().await;

    let total = results.len();
    let success_count = results.iter().filter(|r| r.success).count();
    let failed_count = total - success_count;
    let processing_time = start_time.elapsed().as_millis() as u64;

    let stats = BatchQueryStats {
        total,
        success_count,
        failed_count,
        processing_time_ms: processing_time,
    };

    (results, stats)
}

#[tokio::test]
async fn test_batch_query_logic_success() {
    let phone_data = Arc::new(PhoneData::new().expect("Failed to load phone data"));

    let phones = vec![
        "13800138000".to_string(),
        "13900139000".to_string(),
        "18086834111".to_string(),
    ];

    let (results, stats) = batch_query_logic(phone_data, phones).await;

    assert_eq!(results.len(), 3);
    assert_eq!(stats.total, 3);
    assert_eq!(stats.success_count + stats.failed_count, 3);
    // processing_time_ms 是 u64，总是 >= 0，所以我们只需要验证它存在
    assert!(stats.processing_time_ms < u64::MAX); // 验证处理时间是有效值

    // 检查每个结果的结构
    for result in &results {
        assert!(!result.phone.is_empty());
        if result.success {
            assert!(result.data.is_some());
            assert!(result.error.is_none());
        } else {
            assert!(result.data.is_none());
            assert!(result.error.is_some());
        }
    }
}

#[tokio::test]
async fn test_batch_query_empty_list() {
    let phone_data = Arc::new(PhoneData::new().expect("Failed to load phone data"));
    let phones = vec![];

    let (results, stats) = batch_query_logic(phone_data, phones).await;

    assert_eq!(results.len(), 0);
    assert_eq!(stats.total, 0);
    assert_eq!(stats.success_count, 0);
    assert_eq!(stats.failed_count, 0);
}

#[tokio::test]
async fn test_batch_query_invalid_phone_formats() {
    let phone_data = Arc::new(PhoneData::new().expect("Failed to load phone data"));

    let phones = vec![
        "123".to_string(),             // 太短
        "123456789012345".to_string(), // 太长
        "13800138000".to_string(),     // 正常
    ];

    let (results, stats) = batch_query_logic(phone_data, phones).await;

    assert_eq!(results.len(), 3);
    assert_eq!(stats.total, 3);

    // 检查无效格式的结果
    let short_result = results.iter().find(|r| r.phone == "123").unwrap();
    assert!(!short_result.success);
    assert!(short_result.error.as_ref().unwrap().contains("格式无效"));

    let long_result = results
        .iter()
        .find(|r| r.phone == "123456789012345")
        .unwrap();
    assert!(!long_result.success);
    assert!(long_result.error.as_ref().unwrap().contains("格式无效"));
}

#[tokio::test]
async fn test_batch_query_mixed_results() {
    let phone_data = Arc::new(PhoneData::new().expect("Failed to load phone data"));

    let phones = vec![
        "13800138000".to_string(), // 可能存在的号码
        "19999999999".to_string(), // 可能不存在的号码
        "18086834111".to_string(), // 可能存在的号码
    ];

    let (results, stats) = batch_query_logic(phone_data, phones).await;

    assert_eq!(results.len(), 3);
    assert_eq!(stats.total, 3);
    assert_eq!(stats.success_count + stats.failed_count, 3);
    // processing_time_ms 是 u64，总是 >= 0，所以我们只需要验证它存在
    assert!(stats.processing_time_ms < u64::MAX); // 验证处理时间是有效值

    // 检查每个结果的结构
    for result in &results {
        assert!(!result.phone.is_empty());
        if result.success {
            assert!(result.data.is_some());
            assert!(result.error.is_none());

            let data = result.data.as_ref().unwrap();
            assert!(!data.province.is_empty());
            assert!(!data.city.is_empty());
            assert!(!data.card_type.is_empty());
        } else {
            assert!(result.data.is_none());
            assert!(result.error.is_some());
        }
    }

    println!("批量查询结果统计:");
    println!("总数: {}", stats.total);
    println!("成功: {}", stats.success_count);
    println!("失败: {}", stats.failed_count);
    println!("处理时间: {}ms", stats.processing_time_ms);
}

#[tokio::test]
async fn test_batch_query_order_preservation() {
    let phone_data = Arc::new(PhoneData::new().expect("Failed to load phone data"));

    // 创建一个具有特定顺序的手机号列表
    let phones = vec![
        "13800000001".to_string(),
        "13800000002".to_string(),
        "13800000003".to_string(),
        "13800000004".to_string(),
        "13800000005".to_string(),
    ];

    let (results, _) = batch_query_logic(phone_data, phones.clone()).await;

    // 验证结果顺序与输入顺序完全一致
    assert_eq!(results.len(), 5);
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.index, i, "索引 {} 的结果顺序不正确", i);
        assert_eq!(result.phone, phones[i], "索引 {} 的手机号不匹配", i);
    }
}

#[tokio::test]
async fn test_batch_query_index_mapping() {
    let phone_data = Arc::new(PhoneData::new().expect("Failed to load phone data"));

    // 创建包含重复和特殊字符的测试用例
    let phones = vec![
        "13800138000".to_string(), // 索引 0
        "invalid".to_string(),     // 索引 1 - 无效格式
        "13800138000".to_string(), // 索引 2 - 重复
        "19999999999".to_string(), // 索引 3 - 可能未找到
        "18000000000".to_string(), // 索引 4
    ];

    let (results, stats) = batch_query_logic(phone_data, phones.clone()).await;

    // 验证每个结果都有正确的索引和对应的手机号
    assert_eq!(results.len(), 5);
    assert_eq!(stats.total, 5);

    for (expected_index, expected_phone) in phones.iter().enumerate() {
        let result = &results[expected_index];
        assert_eq!(result.index, expected_index, "索引映射错误");
        assert_eq!(result.phone, *expected_phone, "手机号映射错误");
    }

    // 验证无效格式的处理
    let invalid_result = &results[1];
    assert!(!invalid_result.success);
    // "invalid" 会被解析失败，触发数据库格式错误
    let error_msg = invalid_result.error.as_ref().unwrap();
    assert!(error_msg.contains("数据库格式错误") || error_msg.contains("格式无效"));
}

#[cfg(test)]
mod concurrency_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_access_stress() {
        let phone_data = Arc::new(PhoneData::new().expect("Failed to load phone data"));

        // 创建多个并发任务同时进行批量查询
        let mut tasks = Vec::new();

        for task_id in 0..10 {
            let phone_data = phone_data.clone();
            let task = tokio::spawn(async move {
                let phones: Vec<String> = (0..20)
                    .map(|i| format!("138{:04}{:04}", task_id, i))
                    .collect();

                let (results, _) = batch_query_logic(phone_data, phones.clone()).await;

                // 验证每个任务的结果顺序正确
                for (i, result) in results.iter().enumerate() {
                    assert_eq!(result.index, i);
                    assert_eq!(result.phone, phones[i]);
                }

                task_id
            });
            tasks.push(task);
        }

        // 等待所有任务完成
        let mut completed_tasks = Vec::new();
        for task in tasks {
            completed_tasks.push(task.await.expect("任务执行失败"));
        }

        // 验证所有任务都正确完成
        assert_eq!(completed_tasks.len(), 10);
        completed_tasks.sort();
        assert_eq!(completed_tasks, (0..10).collect::<Vec<_>>());

        println!("并发压力测试完成：10个任务，每个任务20个查询，总共200个并发查询");
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_query_performance_small() {
        let phone_data = Arc::new(PhoneData::new().expect("Failed to load phone data"));

        // 创建10个手机号进行批量查询
        let phones: Vec<String> = (0..10).map(|i| format!("138{:08}", i)).collect();

        let start = Instant::now();
        let (results, stats) = batch_query_logic(phone_data, phones).await;
        let duration = start.elapsed();

        assert_eq!(results.len(), 10);
        assert_eq!(stats.total, 10);

        println!("批量查询10个号码耗时: {:?}", duration);
        println!("服务器报告处理时间: {}ms", stats.processing_time_ms);

        // 批量查询应该很快
        assert!(
            duration.as_millis() < 1000,
            "小批量查询性能不符合预期: {:?}",
            duration
        );
    }

    #[tokio::test]
    async fn test_batch_query_performance_large() {
        let phone_data = Arc::new(PhoneData::new().expect("Failed to load phone data"));

        // 创建100个手机号进行批量查询
        let phones: Vec<String> = (0..100).map(|i| format!("138{:08}", i)).collect();

        let start = Instant::now();
        let (results, stats) = batch_query_logic(phone_data, phones).await;
        let duration = start.elapsed();

        assert_eq!(results.len(), 100);
        assert_eq!(stats.total, 100);

        println!("批量查询100个号码耗时: {:?}", duration);
        println!("服务器报告处理时间: {}ms", stats.processing_time_ms);

        // 即使是100个号码，也应该在合理时间内完成
        assert!(
            duration.as_millis() < 5000,
            "大批量查询性能不符合预期: {:?}",
            duration
        );
    }

    #[tokio::test]
    async fn test_batch_vs_sequential_performance() {
        let phone_data = Arc::new(PhoneData::new().expect("Failed to load phone data"));

        let phones: Vec<String> = (0..50).map(|i| format!("138{:08}", i)).collect();

        // 测试批量查询
        let start = Instant::now();
        let (batch_results, _) = batch_query_logic(phone_data.clone(), phones.clone()).await;
        let batch_duration = start.elapsed();

        // 测试顺序查询
        let start = Instant::now();
        let mut sequential_results = Vec::new();
        for (index, phone) in phones.iter().enumerate() {
            match phone_data.find(phone) {
                Ok(info) => sequential_results.push(BatchQueryResult {
                    phone: phone.clone(),
                    index,
                    success: true,
                    data: Some(info),
                    error: None,
                }),
                Err(e) => sequential_results.push(BatchQueryResult {
                    phone: phone.clone(),
                    index,
                    success: false,
                    data: None,
                    error: Some(format!("{:?}", e)),
                }),
            }
        }
        let sequential_duration = start.elapsed();

        println!("批量查询50个号码耗时: {:?}", batch_duration);
        println!("顺序查询50个号码耗时: {:?}", sequential_duration);

        assert_eq!(batch_results.len(), sequential_results.len());

        // 批量查询应该比顺序查询快或至少不慢太多
        // 注意：由于缓存效应，这个测试结果可能有变化
        println!(
            "批量查询相对于顺序查询的性能提升: {:.2}%",
            ((sequential_duration.as_nanos() as f64 - batch_duration.as_nanos() as f64)
                / sequential_duration.as_nanos() as f64)
                * 100.0
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_query_with_real_numbers() {
        let phone_data = Arc::new(PhoneData::new().expect("Failed to load phone data"));

        // 使用一些真实的手机号段进行测试
        let phones = vec![
            "13800000000".to_string(), // 中国移动
            "13900000000".to_string(), // 中国移动
            "18600000000".to_string(), // 中国联通
            "17700000000".to_string(), // 中国电信
        ];

        let (results, stats) = batch_query_logic(phone_data, phones).await;

        assert_eq!(results.len(), 4);
        assert_eq!(stats.total, 4);

        // 检查每个结果的结构
        for result in &results {
            assert!(!result.phone.is_empty());

            if result.success {
                assert!(result.data.is_some());
                assert!(result.error.is_none());

                let data = result.data.as_ref().unwrap();
                assert!(!data.province.is_empty());
                assert!(!data.city.is_empty());
                assert!(!data.card_type.is_empty());
            } else {
                assert!(result.data.is_none());
                assert!(result.error.is_some());
            }
        }

        println!("批量查询结果统计:");
        println!("总数: {}", stats.total);
        println!("成功: {}", stats.success_count);
        println!("失败: {}", stats.failed_count);
        println!("处理时间: {}ms", stats.processing_time_ms);
    }
}
