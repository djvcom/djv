use leptos::prelude::*;

#[component]
pub fn ProjectCard(
    name: String,
    description: Option<String>,
    url: String,
    kind: Option<String>,
    language: Option<String>,
) -> impl IntoView {
    let description_text = description.unwrap_or_default();
    let badge = kind.or(language);

    view! {
        <li class="project-card">
            <a href=url target="_blank" rel="noopener">
                <div class="project-header">
                    <h3>{name}</h3>
                    {badge.map(|b| view! { <span class="project-badge">{b}</span> })}
                </div>
                <p class="project-description">{description_text}</p>
            </a>
        </li>
    }
}
