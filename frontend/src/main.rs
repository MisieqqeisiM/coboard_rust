#![allow(non_snake_case)]
mod use_client;

use std::collections::HashMap;

use common::{
    entities::Position,
    websocket::{ToClient, ToServer},
};
use ev::mousemove;
use leptos::*;
use leptos_use::*;
use logging::log;
use use_client::*;

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
fn App() -> impl IntoView {
    let (x, set_x) = create_signal(0);
    let (y, set_y) = create_signal(0);

    let _ = use_event_listener(use_document(), mousemove, move |e| {
        set_x.set(e.client_x());
        set_y.set(e.client_y());
    });

    let client = create_local_resource(|| (), |_| Client::new());

    let (clients, set_clients) = create_signal(HashMap::<u64, Position>::new());

    create_effect(move |_| {
        let Some(Some(client)) = client.get() else {
            return;
        };
        let Some(message) = client.message() else {
            return;
        };
        log!("{:?}", message);
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
        let Some(Some(client)) = client.get() else {
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
}
fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
