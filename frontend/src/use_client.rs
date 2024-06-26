use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use common::websocket::{ToClient, ToServer};
use leptos::{create_signal, window, ReadSignal, SignalGet, SignalSet};
use web_sys::{js_sys::{ArrayBuffer, Uint8Array}, wasm_bindgen::{closure::Closure, JsCast}, BinaryType, Event, MessageEvent, WebSocket};

#[derive(Clone)]
pub struct Client {
    websocket: WebSocket,
    message: ReadSignal<Option<ToClient>>,
    message_queue: Rc<RefCell<VecDeque<ToServer>>>
}

impl Client {
    pub async fn new() -> Option<Client>{
        let host = window().location().host().unwrap();
        let protocol = window().location().protocol().unwrap();
        let base = format!("{protocol}//{host}");
        let url = reqwest::get(format!("{base}/api/board_url?name=general")).await.ok()?.text().await.ok()?;
        let (message, set_message) = create_signal(None);
        let websocket = WebSocket::new(&url).ok()?;
        websocket.set_binary_type(BinaryType::Arraybuffer);

        let message_queue = Rc::new(RefCell::new(VecDeque::new()));

        let onmessage = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            let data = e.data().dyn_into::<ArrayBuffer>().unwrap();
            let data = Uint8Array::new(&data).to_vec();
            let message = serde_cbor::from_slice::<ToClient>(&data).unwrap();
            set_message.set(Some(message));
        });

        websocket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        let websocket_clone = websocket.clone();
        let message_queue_clone = message_queue.clone();
        let onopen = Closure::<dyn FnMut(_)>::new(move |_: Event| {
            let mut message_queue = message_queue_clone.borrow_mut();
            while let Some(message) = message_queue.pop_front() {
                websocket_clone.send_with_u8_array(&serde_cbor::to_vec(&message).unwrap()).unwrap();
            }
        });

        websocket.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();
        
        Some(Client {
            websocket,
            message,
            message_queue,
        })
    }

    pub fn message(&self) -> Option<ToClient> {
        self.message.get()
    }

    pub fn send(&self, message: ToServer) {
        if self.websocket.ready_state() == WebSocket::OPEN {
            self.websocket.send_with_u8_array(&serde_cbor::to_vec(&message).unwrap()).unwrap();
        } else {
            self.message_queue.borrow_mut().push_back(message);
        }
    }
}