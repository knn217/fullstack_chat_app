use dioxus::prelude::*;

#[component]
pub fn Login() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut token = use_signal(|| None::<String>);

    rsx! {
        div { class: "auth-page",
            div { class: "auth-card",
                h1 { "Log in" }

                form {
                    onsubmit: move |evt| async move {
                        evt.prevent_default();
                        match api::login::login(username(), password()).await {
                            Ok(t) => {
                                token.set(Some(t));
                                status.set("Logged in".to_string());
                            }
                            Err(e) => status.set(format!("Error: {e}")),
                        }
                    },
                    input {
                        name: "username",
                        autocomplete: "username",
                        placeholder: "Username",
                        value: "{username}",
                        oninput: move |e| username.set(e.value()),
                    }
                    input {
                        name: "password",
                        r#type: "password",
                        autocomplete: "current-password",
                        placeholder: "Password",
                        value: "{password}",
                        oninput: move |e| password.set(e.value()),
                    }
                    button { r#type: "submit", "Log in" }
                }

                // GET demo (optional, remove when you have real sessions)
                button {
                    onclick: move |_| async move {
                        match token() {
                            Some(t) => {
                                match api::login::whoami(t).await {
                                    Ok(name) => status.set(format!("You are {name}")),
                                    Err(e) => status.set(format!("Error: {e}")),
                                }
                            }
                            None => status.set("Not logged in".to_string()),
                        }
                    },
                    "Who am I?"
                }

                p { class: "status", "{status}" }
            }
        }
    }
}