use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_query_map,
    StaticSegment,
};

use crate::components::{FilterBar, ProjectData, ProjectGrid, ProjectGridEmpty};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" type="image/svg+xml" href="/favicon.svg"/>
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
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/djv.css"/>
        <Title text="Daniel Verrall"/>

        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ProjectFilters {
    pub kind: Option<String>,
    pub language: Option<String>,
    pub sort: Option<String>,
}

#[server(FetchProjects)]
pub async fn fetch_projects(filters: ProjectFilters) -> Result<Vec<ProjectData>, ServerFnError> {
    use crate::db::{get_projects, ProjectFilters as DbFilters, ProjectKind, SortOrder};
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::PgPool;

    let Extension(pool): Extension<PgPool> = extract().await?;

    let db_filters = DbFilters {
        kind: filters
            .kind
            .as_deref()
            .and_then(|k| k.parse::<ProjectKind>().ok()),
        language: filters.language,
        topic: None,
        sort: filters
            .sort
            .as_deref()
            .and_then(|s| s.parse::<SortOrder>().ok()),
    };

    let projects = get_projects(&pool, &db_filters)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    Ok(projects
        .into_iter()
        .map(|p| ProjectData {
            id: p.id.to_string(),
            name: p.name,
            description: p.description,
            url: p.url,
            kind: p.kind.to_string(),
            language: p.language,
        })
        .collect())
}

#[component]
fn HomePage() -> impl IntoView {
    let query = use_query_map();

    let filters = Memo::new(move |_| {
        let q = query.get();
        ProjectFilters {
            kind: q.get("kind").map(|s| s.to_string()),
            language: q.get("language").map(|s| s.to_string()),
            sort: q.get("sort").map(|s| s.to_string()),
        }
    });

    let projects = Resource::new(move || filters.get(), fetch_projects);

    let on_filter_change = Callback::new(move |(name, value): (String, Option<String>)| {
        let navigate = leptos_router::hooks::use_navigate();
        let current = query.get();

        let mut params: Vec<(String, String)> = vec![];

        // Keep existing params, updating or removing the changed one
        if let Some(k) = current.get("kind") {
            if name != "kind" {
                params.push(("kind".to_string(), k.to_string()));
            }
        }
        if let Some(l) = current.get("language") {
            if name != "language" {
                params.push(("language".to_string(), l.to_string()));
            }
        }
        if let Some(s) = current.get("sort") {
            if name != "sort" {
                params.push(("sort".to_string(), s.to_string()));
            }
        }

        // Add the new value if present
        if let Some(v) = value {
            params.push((name, v));
        }

        let query_string = params
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let url = if query_string.is_empty() {
            "/".to_string()
        } else {
            format!("/?{}", query_string)
        };

        navigate(&url, Default::default());
    });

    let current_filters = filters;

    view! {
        <div class="container">
            <header class="hero">
                <h1>"Daniel Verrall"</h1>
                <p class="tagline">"rust • opentelemetry • nix"</p>
            </header>

            <section class="projects">
                <h2>"Projects"</h2>
                {move || {
                    let f = current_filters.get();
                    view! {
                        <FilterBar
                            kind_filter=f.kind
                            language_filter=f.language
                            sort_filter=f.sort
                            on_filter_change=on_filter_change
                        />
                    }
                }}
                <Suspense fallback=move || view! { <ProjectGridEmpty /> }>
                    {move || {
                        projects.get().map(|result| {
                            match result {
                                Ok(data) if !data.is_empty() => {
                                    view! { <ProjectGrid projects=data /> }.into_any()
                                }
                                _ => view! { <ProjectGridEmpty /> }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </section>
        </div>
    }
}
