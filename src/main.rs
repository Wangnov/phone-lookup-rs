use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
