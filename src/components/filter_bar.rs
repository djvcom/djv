use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct FilterOption {
    pub value: String,
    pub label: String,
    pub active: bool,
}

fn kinds_for(f: Option<&str>) -> Vec<FilterOption> {
    vec![
        FilterOption {
            value: String::new(),
            label: "all".to_owned(),
            active: f.is_none(),
        },
        FilterOption {
            value: "crate".to_owned(),
            label: "crates".to_owned(),
            active: f == Some("crate"),
        },
        FilterOption {
            value: "repo".to_owned(),
            label: "repos".to_owned(),
            active: f == Some("repo"),
        },
    ]
}

fn languages_for(f: Option<&str>) -> Vec<FilterOption> {
    vec![
        FilterOption {
            value: String::new(),
            label: "any".to_owned(),
            active: f.is_none(),
        },
        FilterOption {
            value: "Rust".to_owned(),
            label: "rust".to_owned(),
            active: f == Some("Rust"),
        },
        FilterOption {
            value: "TypeScript".to_owned(),
            label: "typescript".to_owned(),
            active: f == Some("TypeScript"),
        },
        FilterOption {
            value: "Nix".to_owned(),
            label: "nix".to_owned(),
            active: f == Some("Nix"),
        },
    ]
}

fn topics_for(f: Option<&str>, topics: Vec<String>) -> Vec<FilterOption> {
    std::iter::once(FilterOption {
        value: String::new(),
        label: "any".to_owned(),
        active: f.is_none(),
    })
    .chain(topics.into_iter().map(|t| FilterOption {
        active: f == Some(t.as_str()),
        value: t.clone(),
        label: t,
    }))
    .collect()
}

fn sorts_for(f: Option<&str>) -> Vec<FilterOption> {
    vec![
        FilterOption {
            value: "popularity".to_owned(),
            label: "popular".to_owned(),
            active: f != Some("name") && f != Some("updated"),
        },
        FilterOption {
            value: "name".to_owned(),
            label: "name".to_owned(),
            active: f == Some("name"),
        },
        FilterOption {
            value: "updated".to_owned(),
            label: "recent".to_owned(),
            active: f == Some("updated"),
        },
    ]
}

fn render_group(
    name: &'static str,
    options: Vec<FilterOption>,
    on_filter_change: Callback<(String, Option<String>)>,
) -> impl IntoView {
    view! {
        <div class="filter-group">
            <span class="filter-group__label">{name}</span>
            <div class="filter-group__options">
                {options
                    .into_iter()
                    .map(|opt| {
                        let filter_name = name.to_owned();
                        let value = opt.value.clone();
                        let class = if opt.active {
                            "filter-btn filter-btn--active"
                        } else {
                            "filter-btn"
                        };
                        view! {
                            <button
                                class=class
                                on:click=move |_| {
                                    let val = if value.is_empty() { None } else { Some(value.clone()) };
                                    on_filter_change.run((filter_name.clone(), val));
                                }
                            >
                                {opt.label}
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
pub fn FilterBar(
    kind_filter: Option<String>,
    language_filter: Option<String>,
    topic_filter: Option<String>,
    sort_filter: Option<String>,
    #[prop(optional)] topics: Vec<String>,
    #[prop(into)] on_filter_change: Callback<(String, Option<String>)>,
    is_expanded: ReadSignal<bool>,
    set_expanded: WriteSignal<bool>,
) -> impl IntoView {
    struct Selected {
        kind: Option<String>,
        language: Option<String>,
        topic: Option<String>,
        sort: Option<String>,
    }
    let selected = Selected {
        kind: kind_filter,
        language: language_filter,
        topic: topic_filter,
        sort: sort_filter,
    };

    let has_active_filters = selected.kind.is_some()
        || selected.language.is_some()
        || selected.topic.is_some()
        || selected.sort.as_deref().is_some_and(|s| s != "popularity");

    let kinds = kinds_for(selected.kind.as_deref());
    let languages = languages_for(selected.language.as_deref());
    let sorts = sorts_for(selected.sort.as_deref());
    let topic_options = topics_for(selected.topic.as_deref(), topics);
    let show_topics = topic_options.len() > 1;

    view! {
        <div class="filter-bar">
            <button
                class=move || {
                    let mut c = String::from("filter-bar__summary");
                    if has_active_filters {
                        c.push_str(" filter-bar__summary--active");
                    }
                    c
                }
                aria-label="Toggle filters"
                aria-expanded=move || is_expanded.get().to_string()
                on:click=move |_| set_expanded.update(|v| *v = !*v)
            >
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"></polygon>
                </svg>
                <span>"filters"</span>
            </button>

            <div
                class=move || {
                    let mut c = String::from("filter-bar__panel");
                    if is_expanded.get() {
                        c.push_str(" filter-bar__panel--open");
                    }
                    c
                }
            >
                <div class="filter-bar__panel-inner">
                    {render_group("kind", kinds, on_filter_change)}
                    {render_group("language", languages, on_filter_change)}
                    {render_group("sort", sorts, on_filter_change)}
                    {show_topics.then(|| render_group("topic", topic_options, on_filter_change))}
                </div>
            </div>
        </div>
    }
}
