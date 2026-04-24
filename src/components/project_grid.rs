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

struct Group {
    kind: &'static str,
    title: &'static str,
    note: &'static str,
}

const GROUPS: &[Group] = &[
    Group {
        kind: "crate",
        title: "crates",
        note: "rust libraries published on crates.io",
    },
    Group {
        kind: "npm",
        title: "npm packages",
        note: "typescript work",
    },
    Group {
        kind: "repo",
        title: "repositories",
        note: "open-source projects",
    },
];

#[component]
pub fn ProjectGrid(projects: Vec<ProjectData>) -> impl IntoView {
    let sections: Vec<_> = GROUPS
        .iter()
        .filter_map(|g| {
            let items: Vec<ProjectData> = projects
                .iter()
                .filter(|p| p.kind == g.kind)
                .cloned()
                .collect();
            if items.is_empty() {
                None
            } else {
                Some((g, items))
            }
        })
        .collect();
    drop(projects);

    view! {
        {sections
            .into_iter()
            .map(|(g, items)| {
                let count = format!("{:02}", items.len());
                view! {
                    <section class="section">
                        <header class="section__head">
                            <div class="section__title-row">
                                <h2 class="section__title">{g.title}</h2>
                                <span class="section__count">{count}</span>
                            </div>
                            <span class="section__note">{g.note}</span>
                        </header>
                        <ul class="project-list">
                            {items
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
                                            updated_at=p.updated_at
                                        />
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </ul>
                    </section>
                }
            })
            .collect::<Vec<_>>()}
    }
}

#[component]
pub fn ProjectGridEmpty() -> impl IntoView {
    view! {
        <p class="project-empty">"No projects match these filters."</p>
    }
}
