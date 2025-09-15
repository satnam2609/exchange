use serde::{Deserialize, Serialize};

// ---------- ORDER BOOK JARGONS ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    ASK,
    BID,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    LIMIT,
    MARKET,
}

// ---------- ORDER THAT IS NOT A PART OF LIMIT ORDER BOOK YET ----------

///
/// This struct is will be used to send data between sequencer and the matching engine.
/// As the sequencer will try to sequence incoming orders and assign some sequence ids for
/// event sourcing by the matching engine.
//
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawOrder {
    pub seq_id: u128,
    pub order_id: String,
    pub quote: String,
    pub price: f64,
    pub size: u64,
    pub side: Side,
    pub order_type: OrderType,
}

impl Default for RawOrder {
    fn default() -> Self {
        RawOrder {
            seq_id: 0,
            order_id: "DEFAULT_ORDER".into(),
            quote: "DEFAULT".into(),
            price: 0.0,
            size: 0,
            side: Side::BID,
            order_type: OrderType::LIMIT,
        }
    }
}

impl RawOrder {
    pub fn with_seq_id(&mut self, id: u128) -> &mut Self {
        self.seq_id = id;
        self
    }

    pub fn with_order_id(&mut self, order_id: String) -> &mut Self {
        self.order_id = order_id;
        self
    }

    pub fn with_quote(&mut self, quote: String) -> &mut Self {
        self.quote = quote;
        self
    }

    pub fn with_price(&mut self, price: f64) -> &mut Self {
        self.price = price;
        self
    }

    pub fn with_size(&mut self, size: u64) -> &mut Self {
        self.size = size;
        self
    }

    pub fn with_side(&mut self, side: Side) -> &mut Self {
        self.side = side;
        self
    }

    pub fn with_order_type(&mut self, order_type: OrderType) -> &mut Self {
        self.order_type = order_type;
        self
    }
}

// ---------- MESSAGE USED BY ORDER MANAGER AND SEQUECNER ----------

///
/// This struct is will be used to send data between sequencer and the order manager.
/// As the Order manager takes the order request from the clients and do it's job, and after that enqueue
/// the order into the memmory mapped queue and the sequencer just dequeue's it and does the work.
//
pub struct RawMessage {}

// ---------- EVENTS ----------

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Execution {
    INSERTED,
    CANCELLED,
    FILL,
    PARTIAL(f64, u64),
}

// ---------- EVENTS WITH SEQ-ID ----------

/// This struct will be created by the matching engine after processing
/// the raw order as so to track the state of the matchine.

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ExecuteMessage {
    pub seq_id: u128,         // Sequence ID of the processed order/raw_order.
    pub execution: Execution, // Event
}

impl ExecuteMessage {
    pub fn new(seq_id: u128, execution: Execution) -> Self {
        Self { seq_id, execution }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn set_execution(&mut self, execution: Execution) {
        self.execution = execution;
    }
}

// ---------- RAW ORDER MESSAGE ----------
/// This struct will be used between order manager and the sequencer to
/// access the order data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderValue {
    pub quote: String,
    pub order_id: String,
    pub price: f64,
    pub size: u64,
    pub side: Side,
    pub order_type: OrderType,
}

impl OrderValue {
    pub fn into_raw(&self, seq: u128) -> RawOrder {
        RawOrder::default()
            .with_seq_id(seq)
            .with_order_id(self.order_id.clone())
            .with_quote(self.quote.to_owned())
            .with_price(self.price)
            .with_size(self.size)
            .with_side(self.side)
            .with_order_type(self.order_type)
            .to_owned()
    }
}
