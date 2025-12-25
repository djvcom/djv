#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
    use djv::app::*;
    use djv::config::Config;
    use djv::proxy_headers::RecordProxyHeadersLayer;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use opentelemetry_configuration::{
        capture_rust_build_info, ComputeEnvironment, OtelSdkBuilder,
    };

    // Load configuration
    let config = Config::load().expect("failed to load configuration");

    // Build OTel SDK
    let mut otel_builder = OtelSdkBuilder::new()
        .service_name(env!("CARGO_PKG_NAME"))
        .service_version(env!("CARGO_PKG_VERSION"))
        .instrumentation_scope_name(env!("CARGO_PKG_NAME"))
        .deployment_environment(&config.otel.environment)
        .compute_environment(ComputeEnvironment::Auto)
        .with_rust_build_info(capture_rust_build_info!())
        .resource_attribute("vcs.repository.url.full", "https://github.com/djvcom/djv")
        .resource_attribute("vcs.repository.name", env!("CARGO_PKG_NAME"))
        .resource_attribute("vcs.ref.head.revision", env!("VCS_REF_HEAD_REVISION"))
        .resource_attribute("vcs.ref.head.name", env!("VCS_REF_HEAD_NAME"))
        .resource_attribute("vcs.ref.head.type", "branch");

    // Only set endpoint if explicitly configured, otherwise let OTEL_* env vars take effect
    if let Some(ref endpoint) = config.otel.endpoint {
        otel_builder = otel_builder.endpoint(endpoint);
    }

    let _guard = otel_builder
        .with_standard_env()
        .build()
        .expect("failed to initialise OpenTelemetry");

    // Initialise database pool if configured
    let db_pool = if let Some(ref db_config) = config.database {
        match djv::db::init_pool_with_url(&db_config.url).await {
            Ok(pool) => {
                tracing::info!("database pool initialised");
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
        tracing::info!("database not configured, running without database");
        None
    };

    // Spawn background sync task if database is available
    if let Some(ref pool) = db_pool {
        use djv::sync::{
            forges::GitHubForge, spawn_sync_task, ContributionsSync, CratesIoRegistry, SyncSource,
            SyncSources,
        };

        let mut forges: Vec<Box<dyn SyncSource>> = Vec::new();

        if let Some(ref github_config) = config.sync.github {
            forges.push(Box::new(GitHubForge::new(
                github_config.user.clone(),
                github_config.token.clone(),
            )));
        }

        let crates_io = config
            .sync
            .crates_io
            .as_ref()
            .map(|c| CratesIoRegistry::new(c.user.clone()));

        let contributions = config.sync.contributions.as_ref().map(|c| {
            ContributionsSync::new(
                c.user.clone(),
                config.sync.github.as_ref().and_then(|g| g.token.clone()),
                config.sync.github.as_ref().map(|g| g.user.clone()),
            )
        });

        let sources = SyncSources {
            forges,
            crates_io,
            contributions,
        };

        let sync_config = djv::sync::SyncConfig {
            enabled: config.sync.enabled,
            interval_secs: config.sync.interval_secs,
        };

        spawn_sync_task(pool.clone(), sources, sync_config);
    }

    let leptos_conf = get_configuration(None).unwrap();
    let leptos_options = leptos_conf.leptos_options;
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

    // Use configured listen address
    let addr: std::net::SocketAddr = config
        .listen
        .parse()
        .expect("invalid listen address in config");

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
