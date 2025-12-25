use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ContributionData {
    pub repo_name: String,
    pub title: String,
    pub url: String,
    pub merged_at: Option<String>,
}

#[component]
pub fn ContributionsList(contributions: Vec<ContributionData>) -> impl IntoView {
    view! {
        <ul class="contributions-list">
            {contributions
                .into_iter()
                .map(|c| {
                    view! {
                        <li class="contribution-item">
                            <a href=c.url.clone() target="_blank" rel="noopener noreferrer">
                                <span class="contribution-repo">{c.repo_name}</span>
                                <span class="contribution-title">{c.title}</span>
                            </a>
                        </li>
                    }
                })
                .collect::<Vec<_>>()}
        </ul>
    }
}

#[component]
pub fn ContributionsEmpty() -> impl IntoView {
    view! {
        <p class="contributions-empty">"No contributions yet"</p>
    }
}
