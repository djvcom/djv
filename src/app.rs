use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Html, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_query_map,
    StaticSegment,
};
use leptos_use::{use_color_mode, ColorMode, UseColorModeReturn};

use crate::components::{
    ContributionData, ContributionsEmpty, ContributionsList, FilterBar, ProjectData, ProjectGrid,
    ProjectGridEmpty, ProjectsPlaceholder, ThemeToggle,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="color-scheme" content="light dark"/>
                <link rel="icon" type="image/svg+xml" href="/favicon.svg"/>
                <link rel="preconnect" href="https://fonts.googleapis.com"/>
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin/>
                <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500&family=JetBrains+Mono:wght@400&family=Libre+Baskerville:wght@400;700&display=swap" rel="stylesheet"/>
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

#[derive(Clone)]
pub struct ThemeContext {
    pub mode: Signal<ColorMode>,
    pub set_mode: WriteSignal<ColorMode>,
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let UseColorModeReturn { mode, set_mode, .. } = use_color_mode();

    provide_context(ThemeContext { mode, set_mode });

    view! {
        <Html {..} class=move || mode.get().to_string()/>
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InitialPageData {
    pub topics: Vec<String>,
    pub contributions: Vec<ContributionData>,
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
    use crate::state::AppState;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(app_state): Extension<AppState> = extract().await?;
    let pool = app_state
        .pool
        .as_ref()
        .ok_or_else(|| ServerFnError::new("Database not available"))?;

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

    let projects = get_projects(pool, &db_filters)
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
            popularity: p.popularity,
            version: p.version,
            commit_count: p.commit_count,
            updated_at: p.updated_at.map(|dt| dt.format("%Y-%m-%d").to_string()),
        })
        .collect())
}

#[server(FetchTopics)]
pub async fn fetch_topics() -> Result<Vec<String>, ServerFnError> {
    use crate::db::get_distinct_topics;
    use crate::state::AppState;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(app_state): Extension<AppState> = extract().await?;
    let pool = app_state
        .pool
        .as_ref()
        .ok_or_else(|| ServerFnError::new("Database not available"))?;

    let topics = get_distinct_topics(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    Ok(topics)
}

#[server(FetchContributions)]
pub async fn fetch_contributions() -> Result<Vec<ContributionData>, ServerFnError> {
    use crate::db::get_contributions;
    use crate::state::AppState;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(app_state): Extension<AppState> = extract().await?;
    let pool = app_state
        .pool
        .as_ref()
        .ok_or_else(|| ServerFnError::new("Database not available"))?;

    let contributions = get_contributions(pool, 10, 2)
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

#[server(FetchInitialPageData)]
pub async fn fetch_initial_page_data() -> Result<InitialPageData, ServerFnError> {
    use crate::db::{get_contributions, get_distinct_topics};
    use crate::state::AppState;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(app_state): Extension<AppState> = extract().await?;
    let pool = app_state
        .pool
        .as_ref()
        .ok_or_else(|| ServerFnError::new("Database not available"))?;

    let (topics_result, contributions_result) =
        tokio::join!(get_distinct_topics(pool), get_contributions(pool, 10, 2));

    let topics = topics_result.map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let contributions = contributions_result
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
        .into_iter()
        .map(|c| ContributionData {
            repo_name: format!("{}/{}", c.repo_owner, c.repo_name),
            title: c.title.unwrap_or_default(),
            url: c.url,
            merged_at: c.merged_at.map(|dt| dt.format("%Y-%m-%d").to_string()),
        })
        .collect();

    Ok(InitialPageData {
        topics,
        contributions,
    })
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
    let initial_data = Resource::new(|| (), |_| fetch_initial_page_data());

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
                <div class="hero__title">
                    <h1>"Daniel Verrall"</h1>
                    <p class="tagline">"rust • opentelemetry • nix"</p>
                </div>
                <ThemeToggle />
            </header>

            <section class="projects">
                <h2>"Projects"</h2>
                {move || {
                    let f = current_filters.get();
                    let available_topics = initial_data.get()
                        .and_then(|r| r.ok())
                        .map(|d| d.topics)
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
                <div class="archive-link">
                    <a href="/?kind=all">"View all projects"</a>
                </div>
            </section>

            <section class="contributions">
                <h2>"Contributions"</h2>
                <Suspense fallback=move || view! { <ContributionsEmpty /> }>
                    {move || {
                        initial_data.get().map(|result| {
                            match result {
                                Ok(data) if !data.contributions.is_empty() => {
                                    view! { <ContributionsList contributions=data.contributions /> }.into_any()
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
