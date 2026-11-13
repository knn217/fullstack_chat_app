use dioxus::prelude::*;

use ui::{Echo, Hero, Login};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const THEME_JS: Asset = asset!("/assets/theme.js");

// URL -> component table. Each variant needs a component of the same name.
#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(Shell)]
        #[route("/")]
        Home {},
        #[route("/login")]
        Login {},
}

// Root: global resources + the router, which renders whichever route matches.
#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Shell() -> Element {
    rsx! {
        // shared visible chrome goes here (navbar, etc.)
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Script { src: THEME_JS }
        nav { class: "topbar",
            Link { class: "topbar-home", to: Route::Home {}, "Home" }

            div { class: "topbar-right",
                Link { to: Route::Login {}, "Log in" }
                button {
                    class: "theme-toggle",
                    title: "Toggle light/dark theme",
                    onclick: move |_| {
                        document::eval("toggleTheme()");
                    },
                    "🌓"
                }
            }
        }
        Outlet::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        Hero {}
        Echo {}
    }
}
