use leptos::prelude::*;

use super::ThemeToggle;

#[component]
pub fn Masthead() -> impl IntoView {
    view! {
        <aside class="masthead">
            <div class="masthead__strip" aria-hidden="true">
                <span></span>
                <span></span>
                <span></span>
                <span></span>
                <span></span>
                <span></span>
            </div>

            <div class="masthead__rail" aria-hidden="true">
                "rust · opentelemetry · nix"
            </div>

            <div class="masthead__theme">
                <ThemeToggle />
            </div>

            <div class="masthead__body">
                <div>
                    <h1 class="masthead__name">
                        <a href="/">"Daniel"<br/>"Verrall"</a>
                    </h1>
                    <p class="masthead__rail-mobile">"rust · opentelemetry · nix"</p>
                </div>

                <div class="masthead__spacer" aria-hidden="true"></div>

                <nav class="masthead__links" aria-label="contact">
                    <a href="https://github.com/djvcom" target="_blank" rel="noopener noreferrer">
                        "github.com/djvcom"
                    </a>
                    <a href="https://crates.io/users/djvcom" target="_blank" rel="noopener noreferrer">
                        "crates.io/users/djvcom"
                    </a>
                </nav>
            </div>
        </aside>
    }
}
