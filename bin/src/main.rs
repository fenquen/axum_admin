// use std::time::Duration;

use std::{net::SocketAddr, str::FromStr};

use axum::{
    http::{Method, StatusCode},
    routing::get_service,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use configs::CFG;
//
use app::{
    apps,
    my_env::{self, RT},
    tasks, utils,
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};
// 路由日志追踪

// #[tokio::main]
fn main() {
    RT.block_on(async {
        if std::env::var_os("RUST_LOG").is_none() {
            std::env::set_var("RUST_LOG", &CFG.log.log_level);
        }
        my_env::setup();
        // let console_layer = console_subscriber::ConsoleLayer::builder()
        //     .retention(Duration::from_secs(60))
        //     .server_addr(([127, 0, 0, 1], 5555))
        //     .spawn();
        // let console_layer = console_subscriber::spawn();

        //  设置日志追踪
        // if &CFG.log.log_level == "TRACE" {
        //     LogTracer::builder()
        //         .with_max_level(log::LevelFilter::Trace)
        //         .init()
        //         .unwrap();
        // }

        // 系统变量设置
        let log_env = my_env::get_log_level();

        //  日志设置
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
        utils::ApiUtils::init_all_api().await;
        // 定时任务初始化
        tasks::timer_task_init().await.expect("定时任务初始化失败");

        let addr = SocketAddr::from_str(&CFG.server.address).unwrap();
        //  跨域
        let cors = CorsLayer::new()
            .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_origin(Any);
        // 启动app  注意中间件顺序 最后的先执行，尤其AddData
        // 顺序不对可能会导致数据丢失，无法在某些位置获取数据
        let config = RustlsConfig::from_pem_file(&CFG.cert.cert, &CFG.cert.key).await.unwrap();

        let app = Router::new()
            .nest(&CFG.server.api_prefix, apps::api())
            .nest(
                "/",
                get_service(ServeDir::new("."))
                    .handle_error(|error: std::io::Error| async move { (StatusCode::INTERNAL_SERVER_ERROR, format!("Unhandled internal error: {}", error)) }),
            )
            // .with(Tracing)
            .layer(cors);

        match CFG.server.ssl {
            true => axum_server::bind_rustls(addr, config).serve(app.into_make_service()).await.unwrap(),

            false => axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap(),
        }
    })
}
