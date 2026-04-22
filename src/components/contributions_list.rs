use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ContributionData {
    pub repo_name: String,
    pub title: String,
    pub url: String,
    pub merged_at: Option<String>,
}

#[component]
pub fn ContributionsSection(contributions: Vec<ContributionData>) -> impl IntoView {
    if contributions.is_empty() {
        return view! {
            <section class="section">
                <header class="section__head">
                    <div class="section__title-row">
                        <h2 class="section__title">"contributions"</h2>
                        <span class="section__count">"00"</span>
                    </div>
                    <span class="section__note">"upstream patches to public software"</span>
                </header>
                <p class="contrib-empty">"No contributions yet."</p>
            </section>
        }
        .into_any();
    }

    let count = format!("{:02}", contributions.len());

    view! {
        <section class="section">
            <header class="section__head">
                <div class="section__title-row">
                    <h2 class="section__title">"contributions"</h2>
                    <span class="section__count">{count}</span>
                </div>
                <span class="section__note">"upstream patches to public software"</span>
            </header>
            <ul class="contrib-list">
                {contributions
                    .into_iter()
                    .map(|c| {
                        view! {
                            <li class="contrib-row">
                                <a href=c.url target="_blank" rel="noopener noreferrer">
                                    <span class="contrib-row__repo">{c.repo_name}</span>
                                    <span class="contrib-row__title">{c.title}</span>
                                    <span class="contrib-row__arrow" aria-hidden="true">
                                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                                            <line x1="7" y1="17" x2="17" y2="7" />
                                            <polyline points="7 7 17 7 17 17" />
                                        </svg>
                                    </span>
                                </a>
                            </li>
                        }
                    })
                    .collect::<Vec<_>>()}
            </ul>
        </section>
    }
    .into_any()
}
