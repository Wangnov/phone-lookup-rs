use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use futures::stream::{self, StreamExt};

use phone_lookup_rs::config::Config;
use phone_lookup_rs::{PhoneData, PhoneNoInfo};

#[derive(Clone)]
struct AppState {
    pub phone_data: Arc<PhoneData>,
    pub config: Config,
}

impl AppState {
    fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let phone_data = PhoneData::from_file_with_config(
            &config.database.path,
            config.cache.enabled,
            config.cache.max_size,
        )?;
        Ok(AppState {
            phone_data: Arc::new(phone_data),
            config,
        })
    }
}

/// API响应结构体
/// 
/// 统一的API响应格式，支持泛型数据类型
#[derive(Debug, Serialize)]
struct ApiResponse<T>
where
    T: Serialize,
{
    /// 响应码：0表示成功，负数表示错误
    code: i32,
    /// 响应数据，成功时包含查询结果
    data: Option<T>,
    /// 请求是否成功的标志
    success: bool,
    /// 响应消息
    message: &'static str,
}

impl<T: Serialize> ApiResponse<T> {
    /// 创建成功响应
    #[inline]
    pub fn success(data: T) -> Self {
        ApiResponse {
            code: 0,
            message: "success",
            data: Some(data),
            success: true,
        }
    }

    /// 创建错误响应
    #[inline]
    pub fn error(message: &'static str) -> Self {
        Self::error_with_code(-1, message)
    }

    /// 创建带错误码的错误响应
    #[inline]
    pub fn error_with_code(code: i32, message: &'static str) -> Self {
        ApiResponse {
            code,
            message,
            data: None,
            success: false,
        }
    }
}

async fn index() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success("Phone Data API v1.0 - Ready"))
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    phone: String,
}

/// 批量查询请求结构体
#[derive(Debug, Deserialize)]
struct BatchQueryRequest {
    /// 手机号列表，最多支持100个
    phones: Vec<String>,
}

/// 批量查询响应结构体
#[derive(Debug, Serialize)]
struct BatchQueryResponse {
    /// 成功查询的结果
    results: Vec<PhoneQueryResult>,
    /// 查询统计信息
    stats: BatchQueryStats,
}

/// 单个手机号查询结果
#[derive(Debug, Serialize)]
struct PhoneQueryResult {
    /// 查询的手机号（确保与请求中的号码完全一致）
    phone: String,
    /// 在请求数组中的索引位置（从0开始）
    index: usize,
    /// 查询是否成功
    success: bool,
    /// 查询结果数据（成功时为Some）
    data: Option<PhoneNoInfo>,
    /// 错误信息（失败时为Some）
    error: Option<String>,
}

/// 批量查询统计信息
#[derive(Debug, Serialize)]
struct BatchQueryStats {
    /// 查询总数
    total: usize,
    /// 成功数量
    success_count: usize,
    /// 失败数量
    failed_count: usize,
    /// 处理时间（毫秒）
    processing_time_ms: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct HealthCheck {
    status: String,
    version: String,
}

#[get("/query")]
async fn query_phone(info: web::Query<QueryParams>, data: web::Data<AppState>) -> impl Responder {
    let params = info.into_inner();

    // 基本输入验证
    if params.phone.is_empty() || params.phone.len() < 7 {
        let response: ApiResponse<PhoneNoInfo> = ApiResponse::error("手机号码格式无效");
        return HttpResponse::BadRequest().json(response);
    }

    let response = match data.phone_data.find(&params.phone) {
        Ok(info) => {
            tracing::info!("成功查询手机号: {}", params.phone);
            ApiResponse::success(info)
        }
        Err(phone_lookup_rs::ErrorKind::NotFound) => {
            tracing::warn!("手机号码未找到: {}", params.phone);
            ApiResponse::error_with_code(-404, "手机号码未找到")
        }
        Err(phone_lookup_rs::ErrorKind::InvalidLength) => {
            tracing::warn!("手机号码格式无效: {}", params.phone);
            ApiResponse::error_with_code(-400, "手机号码格式无效")
        }
        Err(phone_lookup_rs::ErrorKind::InvalidPhoneDatabase) => {
            tracing::error!("数据库格式错误: {}", params.phone);
            ApiResponse::error_with_code(-500, "数据库格式错误")
        }
        Err(phone_lookup_rs::ErrorKind::Io(e)) => {
            tracing::error!("I/O错误: {} - {:?}", params.phone, e);
            ApiResponse::error_with_code(-500, "系统内部错误")
        }
        Err(e) => {
            tracing::error!("查询失败: {} - {:?}", params.phone, e);
            ApiResponse::error_with_code(-500, "查询失败")
        }
    };

    HttpResponse::Ok().json(response)
}

#[get("/query/{phone}")]
async fn query_phone_by_path(
    phone: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let phone_number = phone.into_inner();

    // 基本输入验证
    if phone_number.is_empty() || phone_number.len() < 7 {
        let response: ApiResponse<PhoneNoInfo> = ApiResponse::error("手机号码格式无效");
        return HttpResponse::BadRequest().json(response);
    }

    let response = match data.phone_data.find(&phone_number) {
        Ok(info) => ApiResponse::success(info),
        Err(phone_lookup_rs::ErrorKind::NotFound) => ApiResponse::error("手机号码未找到"),
        Err(phone_lookup_rs::ErrorKind::InvalidLength) => ApiResponse::error("手机号码格式无效"),
        Err(_) => ApiResponse::error("查询失败"),
    };

    HttpResponse::Ok().json(response)
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    if req_body.len() > 1024 {
        let response: ApiResponse<String> = ApiResponse::error("请求体过大");
        return HttpResponse::PayloadTooLarge().json(response);
    }
    HttpResponse::Ok().json(ApiResponse::success(req_body))
}

#[derive(Debug, Deserialize)]
struct ProvinceQuery {
    province: String,
}

#[get("/health")]
async fn health_check(data: web::Data<AppState>) -> impl Responder {
    let cache_status = if data.config.cache.enabled {
        format!("enabled (max: {})", data.config.cache.max_size)
    } else {
        "disabled".to_string()
    };

    let health = HealthCheck {
        status: "healthy".to_string(),
        version: format!(
            "API: {} | DB: {} | Records: {} | Cache: {} | Port: {} | Queries: {} | Cache Hit Rate: {:.2}%",
            env!("CARGO_PKG_VERSION"),
            data.phone_data.version(),
            data.phone_data.index_count(),
            cache_status,
            data.config.server.port,
            data.phone_data.query_count(),
            data.phone_data.cache_hit_rate()
        ),
    };
    tracing::debug!("健康检查请求");
    HttpResponse::Ok().json(ApiResponse::success(health))
}

#[post("/demo")]
async fn demo_endpoint(pa: web::Json<ProvinceQuery>) -> impl Responder {
    let province_data = pa.into_inner();
    tracing::info!("Province query: {}", province_data.province);
    HttpResponse::Ok().json(ApiResponse::success(format!(
        "Province: {}",
        province_data.province
    )))
}

/// 批量查询手机号归属地信息
/// 
/// 支持同时查询多个手机号，返回每个手机号的查询结果和统计信息
#[post("/batch-query")]
async fn batch_query(
    request: web::Json<BatchQueryRequest>, 
    data: web::Data<AppState>
) -> impl Responder {
    let start_time = Instant::now();
    let batch_request = request.into_inner();
    
    // 输入验证
    if batch_request.phones.is_empty() {
        let response: ApiResponse<BatchQueryResponse> = ApiResponse::error("手机号列表不能为空");
        return HttpResponse::BadRequest().json(response);
    }
    
    if batch_request.phones.len() > 100 {
        let response: ApiResponse<BatchQueryResponse> = ApiResponse::error("批量查询最多支持100个手机号");
        return HttpResponse::BadRequest().json(response);
    }
    
    // 检查每个手机号的基本格式
    for phone in &batch_request.phones {
        if phone.is_empty() || phone.len() < 7 || phone.len() > 11 {
            let response: ApiResponse<BatchQueryResponse> = ApiResponse::error("手机号格式无效");
            return HttpResponse::BadRequest().json(response);
        }
    }
    
    tracing::info!("开始批量查询 {} 个手机号", batch_request.phones.len());
    
    // 使用 futures::stream 进行优化的并发查询，自动保证结果顺序
    let phone_data = data.phone_data.clone();
    let phones = batch_request.phones.clone();
    
    // 创建查询结果的 Future 流（带索引以确保明确映射）
    let results_stream = stream::iter(phones.into_iter().enumerate()).map(|(index, phone)| {
        let phone_data = phone_data.clone();
        async move {
            let phone_clone = phone.clone();
            match phone_data.find(&phone) {
                Ok(info) => PhoneQueryResult {
                    phone: phone_clone,
                    index,
                    success: true,
                    data: Some(info),
                    error: None,
                },
                Err(phone_lookup_rs::ErrorKind::NotFound) => PhoneQueryResult {
                    phone: phone_clone,
                    index,
                    success: false,
                    data: None,
                    error: Some("手机号码未找到".to_string()),
                },
                Err(phone_lookup_rs::ErrorKind::InvalidLength) => PhoneQueryResult {
                    phone: phone_clone,
                    index,
                    success: false,
                    data: None,
                    error: Some("手机号码格式无效".to_string()),
                },
                Err(phone_lookup_rs::ErrorKind::InvalidPhoneDatabase) => PhoneQueryResult {
                    phone: phone_clone,
                    index,
                    success: false,
                    data: None,
                    error: Some("数据库格式错误".to_string()),
                },
                Err(_) => PhoneQueryResult {
                    phone: phone_clone,
                    index,
                    success: false,
                    data: None,
                    error: Some("查询失败".to_string()),
                }
            }
        }
    });
    
    // 并发执行查询并收集结果（保持原始顺序）
    let results: Vec<PhoneQueryResult> = results_stream.buffered(100).collect().await;
    
    // 统计查询结果
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
    
    let batch_response = BatchQueryResponse {
        results,
        stats,
    };
    
    tracing::info!(
        "批量查询完成: 总数={}, 成功={}, 失败={}, 耗时={}ms",
        total, success_count, failed_count, processing_time
    );
    
    HttpResponse::Ok().json(ApiResponse::success(batch_response))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 加载配置
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}", e);
        std::process::exit(1);
    });

    // 初始化日志系统
    let log_level = match config.logging.level.as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => tracing::Level::INFO, // 默认级别
    };

    if config.logging.format == "json" {
        tracing_subscriber::fmt()
            .json()
            .with_max_level(log_level)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(log_level)
            .init();
    }

    // 初始化应用状态
    let app_state = AppState::new(config.clone()).unwrap_or_else(|e| {
        tracing::error!("Failed to initialize app state: {}", e);
        std::process::exit(1);
    });

    let bind_address = (config.server.host.clone(), config.server.port);
    let workers = if config.server.workers == 0 {
        num_cpus::get()
    } else {
        config.server.workers
    };

    tracing::info!(
        "启动手机号归属地查询 API 服务器: {}:{} (workers: {})",
        config.server.host,
        config.server.port,
        workers
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .service(query_phone)
            .service(query_phone_by_path)
            .service(batch_query)
            .service(health_check)
            .service(demo_endpoint)
            .service(echo)
            .route("/", web::get().to(index))
    })
    .workers(workers)
    .bind(bind_address)?
    .run()
    .await
}
