use leptos::prelude::*;

use super::ThemeToggle;

#[component]
pub fn Header(
    /// Show the tagline (homepage style)
    #[prop(default = false)]
    is_home: bool,
) -> impl IntoView {
    view! {
        <header class="header">
            {if is_home {
                view! {
                    <h1 class="header__name">"Daniel Verrall"</h1>
                }.into_any()
            } else {
                view! {
                    <h1 class="header__name">
                        <a href="/">"Daniel Verrall"</a>
                    </h1>
                }.into_any()
            }}
            <ThemeToggle />
        </header>
        <div class="header__meta">
            <span class="header__tagline">"rust · opentelemetry · nix"</span>
            <nav class="header__nav">
                <a href="/projects" class:active=!is_home>"projects"</a>
            </nav>
        </div>
    }
}
