use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct FilterOption {
    pub value: String,
    pub label: String,
    pub active: bool,
}

#[component]
pub fn FilterBar(
    kind_filter: Option<String>,
    language_filter: Option<String>,
    topic_filter: Option<String>,
    sort_filter: Option<String>,
    #[prop(optional)] topics: Vec<String>,
    #[prop(optional)] project_count: Option<usize>,
    #[prop(into)] on_filter_change: Callback<(String, Option<String>)>,
) -> impl IntoView {
    // Collapsed/expanded state for filter groups
    let (is_expanded, set_expanded) = signal(false);

    // Derive current sort label for summary
    let current_sort = match sort_filter.as_deref() {
        Some("name") => "name",
        Some("updated") => "recent",
        _ => "popular",
    };
    let kinds = vec![
        FilterOption {
            value: "".to_string(),
            label: "all".to_string(),
            active: kind_filter.is_none(),
        },
        FilterOption {
            value: "crate".to_string(),
            label: "crates".to_string(),
            active: kind_filter.as_deref() == Some("crate"),
        },
        FilterOption {
            value: "repo".to_string(),
            label: "repos".to_string(),
            active: kind_filter.as_deref() == Some("repo"),
        },
    ];

    let languages = vec![
        FilterOption {
            value: "".to_string(),
            label: "any".to_string(),
            active: language_filter.is_none(),
        },
        FilterOption {
            value: "Rust".to_string(),
            label: "rust".to_string(),
            active: language_filter.as_deref() == Some("Rust"),
        },
        FilterOption {
            value: "TypeScript".to_string(),
            label: "typescript".to_string(),
            active: language_filter.as_deref() == Some("TypeScript"),
        },
        FilterOption {
            value: "Nix".to_string(),
            label: "nix".to_string(),
            active: language_filter.as_deref() == Some("Nix"),
        },
    ];

    let topic_options: Vec<FilterOption> = std::iter::once(FilterOption {
        value: "".to_string(),
        label: "any".to_string(),
        active: topic_filter.is_none(),
    })
    .chain(topics.into_iter().map(|t| FilterOption {
        value: t.clone(),
        label: t.clone(),
        active: topic_filter.as_deref() == Some(&t),
    }))
    .collect();

    let sorts = vec![
        FilterOption {
            value: "popularity".to_string(),
            label: "popular".to_string(),
            active: sort_filter.as_deref() != Some("name")
                && sort_filter.as_deref() != Some("updated"),
        },
        FilterOption {
            value: "name".to_string(),
            label: "name".to_string(),
            active: sort_filter.as_deref() == Some("name"),
        },
        FilterOption {
            value: "updated".to_string(),
            label: "recent".to_string(),
            active: sort_filter.as_deref() == Some("updated"),
        },
    ];

    let render_group = move |name: &'static str, options: Vec<FilterOption>| {
        view! {
            <div class="filter-group">
                <span class="filter-label">{name}</span>
                <div class="filter-options">
                    {options
                        .into_iter()
                        .map(|opt| {
                            let filter_name = name.to_string();
                            let value = opt.value.clone();
                            let class = if opt.active { "filter-btn active" } else { "filter-btn" };
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
    };

    let show_topics = !topic_options.is_empty() && topic_options.len() > 1;

    view! {
        <div class="filter-bar">
            <div class="filter-summary">
                {project_count.map(|count| view! {
                    <span class="filter-count">{count} " projects"</span>
                })}
                <span class="filter-current">" · sorted by " {current_sort}</span>
                <button
                    class="filter-toggle"
                    on:click=move |_| set_expanded.update(|v| *v = !*v)
                >
                    {move || if is_expanded.get() { "hide filters" } else { "filter" }}
                </button>
            </div>
            <Show when=move || is_expanded.get()>
                <div class="filter-groups">
                    {render_group("kind", kinds.clone())}
                    {render_group("language", languages.clone())}
                    {if show_topics {
                        Some(render_group("topic", topic_options.clone()))
                    } else {
                        None
                    }}
                    {render_group("sort", sorts.clone())}
                </div>
            </Show>
        </div>
    }
}
