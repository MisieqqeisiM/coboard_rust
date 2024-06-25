use common::websocket::{ToClient, ToServer};
use leptos::{create_effect, create_signal, Signal, SignalGet, SignalSet};
use leptos_use::{use_websocket, UseWebsocketReturn};

pub struct UseClientReturn<SendMessageFn>
where
    SendMessageFn: Fn(ToServer) + Clone + 'static,
{
    pub message: Signal<Option<ToClient>>,
    pub send_message: SendMessageFn,
}

pub fn use_client() -> UseClientReturn<impl Fn(ToServer) + Clone + 'static> {
    let UseWebsocketReturn {
        message_bytes,
        send_bytes,
        ..
    } = use_websocket("/api/ws");

    let (message, set_message) = create_signal(None);

    create_effect(move |_| {
        set_message.set(
            message_bytes
                .get()
                .map(|message| serde_cbor::from_slice::<ToClient>(&message).unwrap()),
        );
    });

    let send_message = move |message: ToServer| {
        send_bytes(serde_cbor::to_vec(&message).unwrap());
    };

    return UseClientReturn {
        message: message.into(),
        send_message,
    };
}
