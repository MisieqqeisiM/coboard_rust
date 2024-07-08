pub mod entities {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct Position {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Line {
        pub id: i64,
        pub points: Vec<Position>,
        pub width: f32,
    }
}

pub mod api {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Board {
        pub name: String,
    }
}

pub mod websocket {
    use serde::{Deserialize, Serialize};

    use crate::entities::Position;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum ToServer {
        Move { x: f32, y: f32 },
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum ToClient {
        ClientList { clients: Vec<(u64, Position)> },
        NewClient { id: u64 },
        ClientMoved { id: u64, x: f32, y: f32 },
        ClientDisconnected { id: u64 },
    }
}
