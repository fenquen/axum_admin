// use std::time::Duration;

use std::{fs::File, io::BufReader, sync::Arc};

//
use app_service::{service_utils, tasks};
use axum::{
    extract::Request,
    handler::HandlerWithoutStateExt,
    http::{Method, StatusCode},
    routing::get_service,
    Router,
};
// use axum_server::tls_rustls::RustlsConfig;
use configs::CFG;
use futures_util::pin_mut;
use hyper::body::Incoming;
use hyper_util::rt::TokioExecutor;
use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig},
    TlsAcceptor,
};
use tower_http::{
    compression::{predicate::NotForContentType, CompressionLayer, DefaultPredicate, Predicate},
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tower_service::Service;
use tracing::{error, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};
use utils::my_env::{self, RT};
// 路由日志追踪

// #[tokio::main]
fn main() {
    RT.block_on(async {
        if std::env::var_os("LOG_LEVEL").is_none() {
            std::env::set_var("LOG_LEVEL", &CFG.log.log_level);
        }
        my_env::setup();
        let log_env = my_env::get_log_level();

        // 日志设置
        let format = my_env::get_log_format();

        // 文件输出
        let file_appender = tracing_appender::rolling::hourly(&CFG.log.dir, &CFG.log.file);
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        // 标准控制台输出
        let (std_non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
        let logger = Registry::default()
            .with(EnvFilter::from_default_env().add_directive(log_env.into()))
            .with(fmt::Layer::default().with_writer(std_non_blocking).event_format(format.clone()).pretty())
            .with(fmt::Layer::default().with_writer(non_blocking).event_format(format))
            // .with(console_layer)
            ;

        tracing::subscriber::set_global_default(logger).unwrap();

        // apis全局初始化
        service_utils::ApiUtils::init_all_api().await;

        // 定时任务初始化
        tasks::timer_task_init().await.expect("定时任务初始化失败");

        //  跨域
        let corsLayer = CorsLayer::new()
            .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_origin(Any)
            .allow_headers(Any);

        // 顺序不对可能会导致数据丢失，无法在某些位置获取数据
        let static_files_service = get_service(
            ServeDir::new(&CFG.web.dir)
                .not_found_service(handle_404.into_service())
                .append_index_html_on_directories(true),
        );

        let router = Router::new()
            //  "/" 与所有路由冲突
            .nest_service("/", static_files_service)
            .nest(&CFG.server.api_prefix, api::api());

        let router = match &CFG.server.content_gzip {
            true => {
                //  开启压缩后 SSE 数据无法返回  text/event-stream 单独处理不压缩
                let predicate = DefaultPredicate::new().and(NotForContentType::new("text/event-stream"));
                router.layer(CompressionLayer::new().compress_when(predicate))
            }
            false => router,
        };

        let router = router.layer(corsLayer);

        let listener = tokio::net::TcpListener::bind(&CFG.server.address).await.unwrap();
        axum::serve(listener, router).await.unwrap();
    })
}

async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}