use leptos::prelude::*;

use crate::app::{ColorMode, ThemeContext};

#[component]
pub fn ThemeToggle() -> impl IntoView {
    let theme = expect_context::<ThemeContext>();

    let toggle = move |_| {
        let current = theme.mode.get();
        let next = match current {
            ColorMode::Light => ColorMode::Dark,
            _ => ColorMode::Light,
        };

        // Save to cookie via JS
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::wasm_bindgen::prelude::wasm_bindgen;

            #[wasm_bindgen(
                inline_js = "export function set_cookie(value) { document.cookie = 'djv-theme=' + value + '; path=/; max-age=31536000'; }"
            )]
            extern "C" {
                fn set_cookie(value: &str);
            }

            set_cookie(&next.to_string());
        }

        theme.set_mode.set(next);
    };

    let is_dark = move || !matches!(theme.mode.get(), ColorMode::Light);

    let label = move || {
        if is_dark() {
            "Switch to light mode"
        } else {
            "Switch to dark mode"
        }
    };

    view! {
        <button
            class=move || if is_dark() { "theme-toggle theme-toggle--dark" } else { "theme-toggle theme-toggle--light" }
            on:click=toggle
            aria-label=label
            title=label
        >
            <span class="theme-toggle__icon theme-toggle__icon--light" aria-hidden="true">
                <SunIcon />
            </span>
            <span class="theme-toggle__icon theme-toggle__icon--dark" aria-hidden="true">
                <MoonIcon />
            </span>
        </button>
    }
}

#[component]
fn SunIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="5"/>
            <line x1="12" y1="1" x2="12" y2="3"/>
            <line x1="12" y1="21" x2="12" y2="23"/>
            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
            <line x1="1" y1="12" x2="3" y2="12"/>
            <line x1="21" y1="12" x2="23" y2="12"/>
            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
        </svg>
    }
}

#[component]
fn MoonIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
        </svg>
    }
}
