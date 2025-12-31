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

fn forge_icon_path(url: &str) -> Option<(&'static str, &'static str)> {
    if url.contains("gitlab.com") || url.contains("gitlab.") {
        Some(("/icons/gitlab.svg", "GitLab"))
    } else if url.contains("github.com") {
        Some(("/icons/github.svg", "GitHub"))
    } else {
        None
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

    let metric = if popularity > 0 {
        let label = match kind.as_deref() {
            Some("crate") | Some("npm") => "downloads",
            _ => "stars",
        };
        Some((format_number(popularity), label))
    } else {
        None
    };

    let overlay_items: Vec<String> = [
        version.map(|v| format!("v{}", v)),
        commit_count.map(|c| format!("{} commits", c)),
        updated_at.map(|d| format!("updated {}", d)),
    ]
    .into_iter()
    .flatten()
    .collect();

    let lang_for_icon = language.clone();
    let forge_icon = if lang_for_icon.is_none() {
        forge_icon_path(&url)
    } else {
        None
    };

    view! {
        <li class="project-card">
            <a href=url target="_blank" rel="noopener">
                <div class="project-header">
                    <h3>{name}</h3>
                    <div class="project-header-meta">
                        {lang_for_icon.map(|l| view! { <LanguageIcon language=l /> })}
                        {forge_icon.map(|(path, title)| view! {
                            <span class="language-icon" title=title>
                                <img src=path alt="" />
                            </span>
                        })}
                        {badge.map(|b| view! { <span class="project-badge">{b}</span> })}
                    </div>
                </div>
                <p class="project-description">{description_text}</p>
                <div class="project-meta">
                    {metric.map(|(value, label)| view! {
                        <span class="project-metric">{value}" "{label}</span>
                    })}
                    <span class="project-overlay">
                        {overlay_items.join(" · ")}
                    </span>
                </div>
            </a>
        </li>
    }
}
