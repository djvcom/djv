use leptos::prelude::*;

#[component]
pub fn ProjectsPlaceholder() -> impl IntoView {
    view! {
        <div class="projects-placeholder">
            <svg viewBox="0 0 400 300" class="placeholder-art" aria-label="Abstract landscape illustration">
                <defs>
                    <linearGradient id="skyGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                        <stop offset="0%" style="stop-color:#faf8f5;stop-opacity:1" />
                        <stop offset="100%" style="stop-color:#e8e3dc;stop-opacity:1" />
                    </linearGradient>
                    <linearGradient id="hillGrad1" x1="0%" y1="0%" x2="0%" y2="100%">
                        <stop offset="0%" style="stop-color:#c4b8a8;stop-opacity:0.6" />
                        <stop offset="100%" style="stop-color:#a89880;stop-opacity:0.4" />
                    </linearGradient>
                    <linearGradient id="hillGrad2" x1="0%" y1="0%" x2="0%" y2="100%">
                        <stop offset="0%" style="stop-color:#8b7355;stop-opacity:0.5" />
                        <stop offset="100%" style="stop-color:#6b5842;stop-opacity:0.3" />
                    </linearGradient>
                </defs>

                // Sky background
                <rect x="0" y="0" width="400" height="300" fill="url(#skyGrad)"/>

                // Distant hills
                <path d="M0 200 Q80 160 160 180 Q240 200 320 170 Q360 155 400 175 L400 300 L0 300 Z"
                      fill="url(#hillGrad1)"/>

                // Middle hills
                <path d="M0 230 Q60 200 120 220 Q180 240 240 210 Q300 180 360 200 Q380 210 400 195 L400 300 L0 300 Z"
                      fill="url(#hillGrad2)"/>

                // Foreground
                <path d="M0 260 Q100 240 200 255 Q300 270 400 250 L400 300 L0 300 Z"
                      fill="#e0dbd4" fill-opacity="0.8"/>

                // Sun/moon circle
                <circle cx="320" cy="80" r="25" fill="#fff8f0" fill-opacity="0.8"/>

                // Subtle geometric accents
                <line x1="50" y1="100" x2="50" y2="140" stroke="#c4b8a8" stroke-width="1" stroke-opacity="0.4"/>
                <line x1="350" y1="120" x2="350" y2="150" stroke="#c4b8a8" stroke-width="1" stroke-opacity="0.4"/>

                // Abstract trees/markers
                <circle cx="80" cy="235" r="3" fill="#8b7355" fill-opacity="0.4"/>
                <circle cx="180" cy="245" r="2" fill="#8b7355" fill-opacity="0.3"/>
                <circle cx="280" cy="225" r="4" fill="#8b7355" fill-opacity="0.35"/>
            </svg>
            <p class="placeholder-text">"Projects loading..."</p>
        </div>
    }
}
