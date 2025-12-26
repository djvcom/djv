use leptos::prelude::*;

fn icon_path(language: &str) -> &'static str {
    match language.to_lowercase().as_str() {
        "rust" => "/icons/rust.svg",
        "nix" => "/icons/nix.svg",
        "typescript" => "/icons/typescript.svg",
        "javascript" => "/icons/javascript.svg",
        _ => "/icons/code.svg",
    }
}

#[component]
pub fn LanguageIcon(language: String, #[prop(optional)] clickable: bool) -> impl IntoView {
    let path = icon_path(&language);
    let filter_url = format!("/?language={}", language.to_lowercase());
    let title = format!("Filter by {}", language);

    if clickable {
        view! {
            <a href=filter_url class="language-icon clickable" title=title>
                <img src=path alt="" />
            </a>
        }
        .into_any()
    } else {
        view! {
            <span class="language-icon" title=language>
                <img src=path alt="" />
            </span>
        }
        .into_any()
    }
}
