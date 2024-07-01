use common::{
    entities::{Line, Position},
    websocket::ToServer,
};
use ev::*;
use leptos::*;
use leptos_use::{
    use_event_listener, use_event_listener_with_options, use_interval, UseEventListenerOptions,
    UseIntervalReturn,
};

use crate::{board::camera::Camera, Client};

#[component]
pub fn Controls(
    client: Client,
    camera: RwSignal<Camera>,
    tmp_line: WriteSignal<Line>,
) -> impl IntoView {
    let div = create_node_ref();

    let (mouse_pos, set_mouse_pos) = create_signal((0, 0));

    let _ = use_event_listener(div, mousedown, move |e| {
        set_mouse_pos.set((e.client_x(), e.client_y()))
    });

    let _ = use_event_listener(div, mousemove, move |e| {
        let (mouse_x, mouse_y) = mouse_pos.get_untracked();
        if e.buttons() & 1 != 0 {
            let (x, y) = camera
                .get_untracked()
                .to_board_coords(e.client_x() as f32, e.client_y() as f32);
            tmp_line.update(|line| line.points.push(Position { x, y }));
        }
        if e.buttons() & 4 != 0 {
            camera.update(|camera| {
                camera.change_position(
                    (mouse_x - e.client_x()) as f32,
                    (mouse_y - e.client_y()) as f32,
                )
            });
        }
        set_mouse_pos.set((e.client_x(), e.client_y()));
    });

    let UseIntervalReturn { counter, .. } = use_interval(50);

    create_effect(move |_| {
        let _ = counter.get();
        let (x, y) = mouse_pos.get_untracked();
        let (x, y) = camera.get_untracked().to_board_coords(x as f32, y as f32);
        client.send(ToServer::Move {
            x: x as f32,
            y: y as f32,
        });
    });

    let _ = use_event_listener_with_options(
        document(),
        wheel,
        move |e| {
            e.prevent_default();
            let amount = 1.1_f32.powf(-e.delta_y().signum() as f32);
            camera.update(|camera| {
                camera.change_zoom(e.client_x() as f32, e.client_y() as f32, amount)
            });
        },
        UseEventListenerOptions::default().passive(false),
    );

    view! { <div _ref=div id="controls"></div> }
}
