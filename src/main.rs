#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
    use djv::app::*;
    use djv::proxy_headers::RecordProxyHeadersLayer;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use opentelemetry_configuration::{
        capture_rust_build_info, ComputeEnvironment, OtelSdkBuilder,
    };

    let _guard = OtelSdkBuilder::new()
        .service_name(env!("CARGO_PKG_NAME"))
        .service_version(env!("CARGO_PKG_VERSION"))
        .instrumentation_scope_name(env!("CARGO_PKG_NAME"))
        .deployment_environment("production")
        .compute_environment(ComputeEnvironment::Auto)
        .with_rust_build_info(capture_rust_build_info!())
        .resource_attribute("vcs.repository.url.full", "https://github.com/djvcom/djv")
        .resource_attribute("vcs.repository.name", env!("CARGO_PKG_NAME"))
        .resource_attribute("vcs.ref.head.revision", env!("VCS_REF_HEAD_REVISION"))
        .resource_attribute("vcs.ref.head.name", env!("VCS_REF_HEAD_NAME"))
        .resource_attribute("vcs.ref.head.type", "branch")
        .endpoint("http://127.0.0.1:4318")
        .with_standard_env()
        .build()
        .expect("failed to initialise OpenTelemetry");

    // Optionally initialise the database pool if DATABASE_URL is set
    let db_pool = if std::env::var("DATABASE_URL").is_ok() {
        match djv::db::init_pool().await {
            Ok(pool) => {
                tracing::info!("database pool initialised");
                // Run migrations
                if let Err(e) = djv::db::run_migrations(&pool).await {
                    tracing::error!("failed to run migrations: {}", e);
                }
                Some(pool)
            }
            Err(e) => {
                tracing::warn!("failed to initialise database pool: {}", e);
                None
            }
        }
    } else {
        tracing::info!("DATABASE_URL not set, running without database");
        None
    };

    // Spawn background sync task if database is available
    if let Some(ref pool) = db_pool {
        use djv::sync::{
            forges::GitHubForge, spawn_sync_task, ContributionsSync, CratesIoRegistry, SyncConfig,
            SyncSource, SyncSources,
        };

        let mut forges: Vec<Box<dyn SyncSource>> = Vec::new();

        if let Some(github) = GitHubForge::from_env() {
            forges.push(Box::new(github));
        }

        let crates_io = CratesIoRegistry::from_env();
        let contributions = ContributionsSync::from_env();

        let sources = SyncSources {
            forges,
            crates_io,
            contributions,
        };

        let config = SyncConfig::from_env();
        spawn_sync_task(pool.clone(), sources, config);
    }

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let mut app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default())
        .layer(RecordProxyHeadersLayer)
        .with_state(leptos_options);

    // Add database pool as extension if available
    if let Some(pool) = db_pool {
        app = app.layer(axum::Extension(pool));
    }

    let addr = std::env::var("DJV_LISTEN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(addr);

    tracing::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
