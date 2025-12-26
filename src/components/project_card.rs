use leptos::prelude::*;

use super::LanguageIcon;

fn format_number(n: i32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}m", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

#[component]
pub fn ProjectCard(
    name: String,
    description: Option<String>,
    url: String,
    kind: Option<String>,
    language: Option<String>,
    popularity: i32,
    version: Option<String>,
    commit_count: Option<i32>,
    updated_at: Option<String>,
) -> impl IntoView {
    let description_text = description.unwrap_or_default();
    let badge = kind.clone().or(language.clone());

    let metric_label = match kind.as_deref() {
        Some("crate") | Some("npm") => "downloads",
        _ => "stars",
    };
    let metric_value = format_number(popularity);

    let overlay_items: Vec<String> = [
        version.map(|v| format!("v{}", v)),
        commit_count.map(|c| format!("{} commits", c)),
        updated_at.map(|d| format!("updated {}", d)),
    ]
    .into_iter()
    .flatten()
    .collect();

    let lang_for_icon = language.clone();

    view! {
        <li class="project-card">
            <a href=url target="_blank" rel="noopener">
                <div class="project-header">
                    <h3>{name}</h3>
                    <div class="project-header-meta">
                        {lang_for_icon.map(|l| view! { <LanguageIcon language=l /> })}
                        {badge.map(|b| view! { <span class="project-badge">{b}</span> })}
                    </div>
                </div>
                <p class="project-description">{description_text}</p>
                <div class="project-meta">
                    <span class="project-metric">{metric_value}" "{metric_label}</span>
                    <span class="project-overlay">
                        {overlay_items.join(" · ")}
                    </span>
                </div>
            </a>
        </li>
    }
}
