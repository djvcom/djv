#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
    use djv::app::*;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use opentelemetry_configuration::OtelSdkBuilder;

    let _guard = OtelSdkBuilder::new()
        .service_name(env!("CARGO_PKG_NAME"))
        .service_version(env!("CARGO_PKG_VERSION"))
        .resource_attribute("vcs.repository.url.full", "https://github.com/djvcom/djv")
        .resource_attribute("vcs.repository.name", env!("CARGO_PKG_NAME"))
        .resource_attribute("vcs.ref.head.revision", env!("VCS_REF_HEAD_REVISION"))
        .resource_attribute("vcs.ref.head.name", env!("VCS_REF_HEAD_NAME"))
        .resource_attribute("vcs.ref.head.type", "branch")
        .endpoint("http://127.0.0.1:4318")
        .with_standard_env()
        .build()
        .expect("failed to initialise OpenTelemetry");

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default())
        .with_state(leptos_options);

    if let Ok(socket_path) = std::env::var("DJV_SOCKET") {
        tracing::info!("listening on unix socket {}", &socket_path);
        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    } else {
        tracing::info!("listening on http://{}", &addr);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
