use dioxus::prelude::*;

use ui::{Echo, Hero};

use crate::api;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

// URL -> component table. Each variant needs a component of the same name.
#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(Shell)]
        #[route("/")]
        Home {},
    #[end_layout]

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
        nav { class: "topbar",
            Link { to: Route::Login {}, "Log in" }
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
        h1 { "Log in" }

        form {
            onsubmit: move |evt| async move {
                evt.prevent_default();
                match api::login(username(), password()).await {
                    Ok(t) => {
                        token.set(Some(t));
                        status.set("Logged in".to_string());
                        // Uncomment to redirect after a successful login:
                        // nav.push(Route::Home {});
                    }
                    Err(e) => status.set(format!("Error: {e}")),
                }
            },
            input {
                placeholder: "username",
                value: "{username}",
                oninput: move |e| username.set(e.value()),
            }
            input {
                r#type: "password",
                placeholder: "password",
                value: "{password}",
                oninput: move |e| password.set(e.value()),
            }
            button { r#type: "submit", "Log in" }
        }

        button {
            onclick: move |_| async move {
                match token() {
                    Some(t) => {
                        match api::whoami(t).await {
                            Ok(name) => status.set(format!("You are {name}")),
                            Err(e) => status.set(format!("Error: {e}")),
                        }
                    }
                    None => status.set("Not logged in".to_string()),
                }
            },
            "Who am I?"
        }

        p { "{status}" }

        Link { to: Route::Home {}, "Back home" }
    }
}