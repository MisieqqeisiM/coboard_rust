#![allow(non_snake_case)]
mod board;
mod client;
mod line_drawing;
mod webgl_utils;

use board::canvas::{CanvasControls, LineControls};
use board::controls::Controls;
use board::cursor_box::CursorBox;
use board::{camera::Camera, canvas::Canvas};
use client::*;
use common::entities::Line;
use common::{entities::Position, websocket::ToClient};
use leptos::*;
use leptos_use::*;
use std::collections::HashMap;

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

    let (tmp_line, set_tmp_line) = create_signal(Line {
        id: 0,
        points: Vec::new(),
        width: 10.0,
    });

    let camera = create_rw_signal(Camera::new());

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

    view! {
        {move || {
            match client.get() {
                Some(client) => {
                    view! {
                        <Canvas controls=CanvasControls {
                            camera: camera.read_only(),
                            add_line: create_signal(Line {
                                    id: 0,
                                    points: vec![],
                                    width: 30.0,
                                })
                                .0,
                            tmp_line: LineControls {
                                set: tmp_line,
                                add: create_signal(Position { x: 0.0, y: 0.0 }).0,
                            },
                        }/>
                        <Controls client=client.clone() camera=camera tmp_line=set_tmp_line/>
                        <CursorBox clients=clients camera=camera.read_only()/>
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
