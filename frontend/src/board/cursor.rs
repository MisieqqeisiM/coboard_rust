use leptos::*;

use common::entities::Position;

#[component]
pub fn Cursor(name: String, position: Signal<Position>) -> impl IntoView {
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
