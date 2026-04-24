use leptos::prelude::*;

fn format_number(n: i32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}m", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn lang_dot_class(language: Option<&str>) -> Option<&'static str> {
    match language.map(str::to_ascii_lowercase).as_deref() {
        Some("rust") => Some("lang-dot lang-dot--rust"),
        Some("typescript") => Some("lang-dot lang-dot--typescript"),
        Some("nix") => Some("lang-dot lang-dot--nix"),
        _ => None,
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
    #[prop(optional)] _commit_count: Option<i32>,
    updated_at: Option<String>,
) -> impl IntoView {
    let dot_class = lang_dot_class(language.as_deref());
    let language_title = language.clone();

    let metric = if popularity > 0 {
        let (value, unit) = match kind.as_deref() {
            Some("crate") | Some("npm") => (format_number(popularity), "dl"),
            _ => (format!("★ {}", format_number(popularity)), ""),
        };
        Some((value, unit))
    } else {
        None
    };

    let meta_parts: Vec<String> = [version.map(|v| format!("v{}", v)), updated_at]
        .into_iter()
        .flatten()
        .collect();
    let meta_text = meta_parts.join("  ·  ");

    let description_el = description.filter(|d| !d.is_empty()).map(|d| {
        view! { <p class="project-row__desc">{d}</p> }
    });

    view! {
        <li class="project-row">
            <a href=url target="_blank" rel="noopener noreferrer">
                <div>
                    <div class="project-row__head">
                        <h3 class="project-row__name">{name}</h3>
                        {dot_class.map(|c| view! {
                            <span class=c title=language_title.clone().unwrap_or_default()></span>
                        })}
                    </div>
                    {description_el}
                </div>
                <div class="project-row__stats">
                    {metric.map(|(value, unit)| view! {
                        <span class="project-row__metric">
                            {value}
                            {(!unit.is_empty()).then(|| view! {
                                <span class="project-row__metric-unit">{unit}</span>
                            })}
                        </span>
                    })}
                    {(!meta_text.is_empty()).then(|| view! {
                        <span class="project-row__meta">{meta_text}</span>
                    })}
                </div>
            </a>
        </li>
    }
}
