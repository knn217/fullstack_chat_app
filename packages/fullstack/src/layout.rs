use dioxus::prelude::*;

use ui::{Echo, Hero};

use crate::api;

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

#[component]
fn Login() -> Element {
    let nav = use_navigator();
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut token = use_signal(|| None::<String>);

    rsx! {
        div { class: "auth-page",
            div { class: "auth-card",
                h1 { "Log in" }

                form { onsubmit: move |evt| async move {},
                    input {
                        placeholder: "Username",
                        value: "{username}",
                        oninput: move |e| username.set(e.value()),
                    }
                    input {
                        r#type: "password",
                        placeholder: "Password",
                        value: "{password}",
                        oninput: move |e| password.set(e.value()),
                    }
                    button { r#type: "submit", "Log in" }
                }

                p { class: "status", "{status}" }

                Link { to: Route::Home {}, "Back home" }
            }
        }
    }
}