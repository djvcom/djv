use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_query_map,
    StaticSegment,
};

use crate::components::{
    ContributionData, ContributionsEmpty, ContributionsList, FilterBar, ProjectData, ProjectGrid,
    ProjectGridEmpty, ProjectsPlaceholder,
};

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
    pub topic: Option<String>,
    pub sort: Option<String>,
}

#[server(FetchProjects)]
#[cfg_attr(feature = "ssr", tracing::instrument(skip_all, fields(
    filter.kind = ?filters.kind,
    filter.language = ?filters.language,
    filter.topic = ?filters.topic,
    filter.sort = ?filters.sort,
)))]
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
        topic: filters.topic,
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

#[server(FetchTopics)]
pub async fn fetch_topics() -> Result<Vec<String>, ServerFnError> {
    use crate::db::get_distinct_topics;
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::PgPool;

    let Extension(pool): Extension<PgPool> = extract().await?;

    let topics = get_distinct_topics(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    Ok(topics)
}

#[server(FetchContributions)]
pub async fn fetch_contributions() -> Result<Vec<ContributionData>, ServerFnError> {
    use crate::db::get_contributions;
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::PgPool;

    let Extension(pool): Extension<PgPool> = extract().await?;

    let contributions = get_contributions(&pool, 10)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    Ok(contributions
        .into_iter()
        .map(|c| ContributionData {
            repo_name: format!("{}/{}", c.repo_owner, c.repo_name),
            title: c.title.unwrap_or_default(),
            url: c.url,
            merged_at: c.merged_at.map(|dt| dt.format("%Y-%m-%d").to_string()),
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
            topic: q.get("topic").map(|s| s.to_string()),
            sort: q.get("sort").map(|s| s.to_string()),
        }
    });

    let projects = Resource::new(move || filters.get(), fetch_projects);
    let topics = Resource::new(|| (), |_| fetch_topics());
    let contributions = Resource::new(|| (), |_| fetch_contributions());

    let navigate = leptos_router::hooks::use_navigate();

    let on_filter_change = Callback::new(move |(name, value): (String, Option<String>)| {
        let current = query.get();

        let mut params: Vec<(String, String)> = vec![];

        // Keep existing params, updating or removing the changed one
        for key in ["kind", "language", "topic", "sort"] {
            if let Some(v) = current.get(key) {
                if name != key {
                    params.push((key.to_string(), v.to_string()));
                }
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
                    let available_topics = topics.get()
                        .and_then(|r| r.ok())
                        .unwrap_or_default();
                    view! {
                        <FilterBar
                            kind_filter=f.kind.clone()
                            language_filter=f.language.clone()
                            topic_filter=f.topic.clone()
                            sort_filter=f.sort.clone()
                            topics=available_topics
                            on_filter_change=on_filter_change
                        />
                    }
                }}
                <Suspense fallback=move || view! { <ProjectsPlaceholder /> }>
                    {move || {
                        projects.get().map(|result| {
                            match result {
                                Ok(data) if !data.is_empty() => {
                                    view! { <ProjectGrid projects=data /> }.into_any()
                                }
                                Ok(_) => view! { <ProjectGridEmpty /> }.into_any(),
                                Err(_) => view! { <ProjectsPlaceholder /> }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </section>

            <section class="contributions">
                <h2>"Contributions"</h2>
                <Suspense fallback=move || view! { <ContributionsEmpty /> }>
                    {move || {
                        contributions.get().map(|result| {
                            match result {
                                Ok(data) if !data.is_empty() => {
                                    view! { <ContributionsList contributions=data /> }.into_any()
                                }
                                _ => view! { <ContributionsEmpty /> }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </section>
        </div>
    }
}
