use dioxus::prelude::*;


#[component]
pub fn Login() -> Element {
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
            }
        }
    }
}