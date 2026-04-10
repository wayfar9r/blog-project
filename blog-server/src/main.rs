use std::net::SocketAddr;
use std::sync::Arc;

use actix_web::App;
use anyhow::anyhow;

use actix_web::middleware::Logger;

use blog_server::application::auth_service::AuthService;
use blog_server::application::blog_service::BlogService;
use blog_server::data::post_repository::PostRepository;
use blog_server::data::user_repository::UserRepository;
use blog_server::presentation::exchange::blog_service_server::BlogServiceServer;
use blog_server::presentation::grpc_service::BlogGrpcService;

use blog_server::infrastructure::{self, database};
use blog_server::presentation;
use tracing_subscriber::{EnvFilter, fmt};

fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,blog-project=debug"))
        .expect("failed to setting filter for logging");

    let subscriber = fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_level(true)
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .json()
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

#[derive(Clone)]
#[allow(unused)]
struct AppData {
    name: String,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    tracing::info!("Running blog server");

    dotenvy::dotenv().map_err(|e| anyhow!("failed to load env configuration: {e}"))?;

    let db_pool = Arc::new(
        database::create_pool(
            &std::env::var("DATABASE_URL").expect("DATABASE_URL config not found"),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to create database pool: {e}"))?,
    );
    database::run_migrations(&db_pool)
        .await
        .map_err(|e| anyhow::anyhow!("error while running migration: {e}"))?;

    let auth_service = Arc::new(AuthService::new(UserRepository::new(Arc::clone(&db_pool))));
    let blog_service = Arc::new(BlogService::new(PostRepository::new(Arc::clone(&db_pool))));

    let jwt_service = Arc::new(infrastructure::jwt::JwtService::new(
        std::env::var("JWT_SECRET").expect("JWT_SECRET config not found"),
        std::env::var("JWT_TTL")
            .expect("JWT_TTL config not found")
            .parse::<u32>()
            .expect("JWT_TTL config is not number"),
    ));

    let http_jwt_service = Arc::clone(&jwt_service);
    let http_auth_service = Arc::clone(&auth_service);
    let http_blog_service = Arc::clone(&blog_service);

    let http_server = actix_web::HttpServer::new(move || {
        let _app_data = AppData {
            name: "blog".into(),
        };

        let auth_middleware = actix_web_httpauth::middleware::HttpAuthentication::bearer(
            presentation::middleware::jwt_validator,
        );

        let cors = actix_cors::Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allow_any_origin()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(actix_web::web::Data::new(Arc::clone(&http_jwt_service)))
            .app_data(actix_web::web::Data::new(Arc::clone(&http_auth_service)))
            .app_data(actix_web::web::Data::new(Arc::clone(&http_blog_service)))
            .service(
                actix_web::web::scope("/api/auth")
                    .service(presentation::http_handlers::login)
                    .service(presentation::http_handlers::register),
            )
            .service(presentation::http_handlers::get_post)
            .service(presentation::http_handlers::get_posts)
            .service(
                actix_web::web::scope("")
                    .wrap(auth_middleware)
                    .service(presentation::http_handlers::create_post)
                    .service(presentation::http_handlers::delete_post)
                    .service(presentation::http_handlers::update_post),
            )
    })
    .shutdown_timeout(1)
    .bind(format!(
        "{}:{}",
        dotenvy::var("HOST")?,
        dotenvy::var("PORT")?
    ))?
    .run();

    let grpc_service = BlogGrpcService::new(
        Arc::clone(&auth_service),
        Arc::clone(&blog_service),
        Arc::clone(&jwt_service),
    );

    let grpc_server = tonic::transport::Server::builder()
        .add_service(BlogServiceServer::new(grpc_service))
        .serve("0.0.0.0:50051".parse::<SocketAddr>().unwrap());
    tokio::select! {
        r1 = http_server => {
            if let Err(e) = r1 {
                tracing::error!("http server error: {e}");
            }
        },
        r2 = grpc_server => {
            if let Err(e) = r2 {
                tracing::error!("grpc server error: {e}");
            }
        },
    };
    Ok(())
}
