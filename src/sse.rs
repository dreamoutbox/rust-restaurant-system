use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseEvent {
    NewOrderItems {
        table_number: i32,
        order_id: Uuid,
        items_count: usize,
    },
    ItemStatusChanged {
        table_number: i32,
        order_id: Uuid,
        item_id: Uuid,
        item_name: String,
        status: String,
    },
    OrderCheckout {
        table_number: i32,
        order_id: Uuid,
        total: String,
    },
    PaymentReceived {
        table_number: i32,
        order_id: Uuid,
        method: String,
    },
}

#[derive(Clone)]
pub struct SseBroadcaster {
    tx: broadcast::Sender<SseEvent>,
}

impl SseBroadcaster {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn send(&self, event: SseEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SseEvent> {
        self.tx.subscribe()
    }
}
