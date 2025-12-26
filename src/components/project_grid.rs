use leptos::prelude::*;

use super::ProjectCard;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectData {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub kind: String,
    pub language: Option<String>,
    pub popularity: i32,
    pub version: Option<String>,
    pub commit_count: Option<i32>,
    pub updated_at: Option<String>,
}

#[component]
pub fn ProjectGrid(projects: Vec<ProjectData>) -> impl IntoView {
    view! {
        <ul class="project-list">
            {projects
                .into_iter()
                .map(|p| {
                    view! {
                        <ProjectCard
                            name=p.name
                            description=p.description
                            url=p.url
                            kind=Some(p.kind)
                            language=p.language
                            popularity=p.popularity
                            version=p.version
                            commit_count=p.commit_count
                            updated_at=p.updated_at
                        />
                    }
                })
                .collect::<Vec<_>>()}
        </ul>
    }
}

#[component]
pub fn ProjectGridEmpty() -> impl IntoView {
    view! {
        <div class="project-empty">
            <svg viewBox="0 0 200 200" class="project-empty-art">
                <defs>
                    <linearGradient id="warmGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                        <stop offset="0%" style="stop-color:#e0dbd4;stop-opacity:0.3" />
                        <stop offset="100%" style="stop-color:#8b7355;stop-opacity:0.1" />
                    </linearGradient>
                </defs>
                // Abstract geometric pattern
                <circle cx="100" cy="100" r="80" fill="none" stroke="#e0dbd4" stroke-width="1"/>
                <circle cx="100" cy="100" r="60" fill="none" stroke="#e0dbd4" stroke-width="1"/>
                <circle cx="100" cy="100" r="40" fill="none" stroke="#e0dbd4" stroke-width="1"/>
                <line x1="20" y1="100" x2="180" y2="100" stroke="#e0dbd4" stroke-width="1"/>
                <line x1="100" y1="20" x2="100" y2="180" stroke="#e0dbd4" stroke-width="1"/>
                <circle cx="100" cy="100" r="8" fill="url(#warmGrad)"/>
            </svg>
            <p class="project-empty-text">"No projects found"</p>
        </div>
    }
}
