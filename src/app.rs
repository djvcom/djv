use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/djv.css"/>

        <Title text="Daniel Verrall"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="container">
            <header class="hero">
                <h1>"Daniel Verrall"</h1>
                <p class="tagline">"building with rust • watching it run"</p>
            </header>

            <section class="projects">
                <h2>"Projects"</h2>
                <ul class="project-list">
                    <ProjectCard
                        name="lambda-observability"
                        description="Rust workspace for AWS Lambda observability - runtime simulator, OpenTelemetry configuration, and Lambda extension for telemetry collection"
                        url="https://github.com/djvcom/lambda-observability"
                    />
                    <ProjectCard
                        name="mock-collector"
                        description="Mock OpenTelemetry OTLP collector for testing Rust applications"
                        url="https://github.com/djvcom/mock-collector"
                    />
                    <ProjectCard
                        name="semantic-lambda"
                        description="OpenTelemetry semantic span wrapper for AWS Lambda handlers"
                        url="https://github.com/djvcom/semantic-lambda"
                    />
                    <ProjectCard
                        name="efs-s3-relay"
                        description="Lambda function to relay files from EFS to S3 with OpenTelemetry instrumentation"
                        url="https://github.com/djvcom/efs-s3-relay"
                    />
                    <ProjectCard
                        name="nix-nginx-otel"
                        description="nginx OpenTelemetry module packaged for NixOS"
                        url="https://github.com/djvcom/nix-nginx-otel"
                    />
                    <ProjectCard
                        name="nixos-config"
                        description="NixOS configuration for personal infrastructure"
                        url="https://github.com/djvcom/nixos-config"
                    />
                </ul>
            </section>
        </div>
    }
}

#[component]
fn ProjectCard(name: &'static str, description: &'static str, url: &'static str) -> impl IntoView {
    view! {
        <li class="project-card">
            <a href=url target="_blank" rel="noopener">
                <h3>{name}</h3>
                <p>{description}</p>
            </a>
        </li>
    }
}
