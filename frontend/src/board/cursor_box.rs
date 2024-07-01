use std::collections::HashMap;

use leptos::*;

use common::entities::Position;

use crate::board::cursor::Cursor;

use super::camera::Camera;

#[component]
pub fn CursorBox(
    clients: ReadSignal<HashMap<u64, Position>>,
    camera: ReadSignal<Camera>,
) -> impl IntoView {
    view! {
        <div
            id="cursor_box"
            style=move || {
                format!(
                    "transform: scale({}) translate({}px, {}px)",
                    camera.get().zoom(),
                    -camera.get().x(),
                    -camera.get().y(),
                )
            }
        >

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

        </div>
    }
}
