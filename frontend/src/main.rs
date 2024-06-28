#![allow(non_snake_case)]
mod canvas;
mod client;
mod line_drawing;

use std::collections::HashMap;

use canvas::Canvas;
use client::*;
use common::{
    entities::Position,
    websocket::{ToClient, ToServer},
};
use ev::mousemove;
use leptos::*;
use leptos_use::*;
use logging::log;

#[component]
fn Cursor(name: String, position: Signal<Position>) -> impl IntoView {
    let x = move || position.get().x;
    let y = move || position.get().y;
    view! {
        <div class="cursor" style=move || { format!("transform: translate({}px, {}px)", x(), y()) }>
            <img class="image" src="/assets/img/pencil.svg" width="30" height="30"/>
            <div class="label">
                <p>{name}</p>
            </div>
        </div>
    }
}

#[component]
fn LoadingSpinner(text: &'static str) -> impl IntoView {
    view! {
        <div class="loading-screen-wrapper no-select">
            <div class="loading-element">
                <div class="spinner">
                    <img src="/assets/img/coboard.svg" width="250" height="250"/>
                </div>
                <div class="loading-text">{text}</div>
            </div>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (x, set_x) = create_signal(0);
    let (y, set_y) = create_signal(0);

    let _ = use_event_listener(use_document(), mousemove, move |e| {
        set_x.set(e.client_x());
        set_y.set(e.client_y());
    });

    let client = create_local_resource(|| (), |_| Client::new());

    let messaged = create_memo(move |_| {
        if let Some(Some(client)) = client.get() {
            client.message().is_some()
        } else {
            false
        }
    });

    let check_connection = {
        let UseIntervalReturn { counter, .. } = use_interval(500);
        counter
    };

    let client_memo = create_memo(move |_| client.get());

    create_effect(move |_| {
        let _ = check_connection.get();
        match client_memo.get() {
            Some(Some(cl)) => {
                if !cl.connected() && messaged.get() {
                    client.refetch();
                }
            }
            Some(None) => client.refetch(),
            _ => (),
        }
    });

    let (clients, set_clients) = create_signal(HashMap::<u64, Position>::new());

    let client = create_memo(move |_| match client.get() {
        Some(Some(client)) => {
            if client.message().is_some() {
                Some(client)
            } else {
                None
            }
        }
        _ => None,
    });

    create_effect(move |_| {
        let Some(client) = client.get() else {
            return;
        };
        let Some(message) = client.message() else {
            return;
        };
        match message {
            ToClient::NewClient { id } => {
                set_clients.update(|clients| {
                    clients.insert(id, Position { x: 0.0, y: 0.0 });
                });
            }
            ToClient::ClientMoved { id, x, y } => {
                set_clients.update(|clients| {
                    clients.insert(id, Position { x, y });
                });
            }
            ToClient::ClientDisconnected { id } => {
                set_clients.update(|clients| {
                    clients.remove(&id);
                });
            }
            ToClient::ClientList { clients } => set_clients.update(|clients_map| {
                clients_map.clear();
                for (id, pos) in clients {
                    clients_map.insert(id, pos);
                }
            }),
        }
    });

    let UseIntervalReturn { counter, .. } = use_interval(50);

    create_effect(move |_| {
        let Some(client) = client.get() else {
            return;
        };
        let _ = counter.get();
        let x = x.get_untracked();
        let y = y.get_untracked();
        client.send(ToServer::Move {
            x: x as f32,
            y: y as f32,
        });
    });

    view! {
        {move || {
            match client.get() {
                Some(client) => {
                    view! {
                        <Canvas client=client/>
                        <For
                            each=move || clients.get()
                            key=move |(id, _)| id.clone()
                            children=move |(id, _)| {
                                let position = create_memo(move |_| {
                                    clients.with(|clients| clients.get(&id).unwrap().to_owned())
                                });
                                view! { <Cursor name=format!("{}", id) position=position.into()/> }
                            }
                        />
                    }
                        .into()
                }
                None => {
                    view! { <LoadingSpinner text="Connecting..."/> }
                }
            }
        }}
    }
}
fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
