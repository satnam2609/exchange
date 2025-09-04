use std::{
    cell::RefCell,
    fmt::Debug,
    rc::{Rc, Weak},
};

use serde::{Deserialize, Serialize};
use memmap::engseq::RawSequencedOrder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize,Deserialize)]
pub enum Side {
    ASK,
    BID,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize,Deserialize)]
pub enum OrderType {
    LIMIT,
    MARKET,
}
#[derive(Debug, Clone, Serialize,Deserialize)]
pub struct RawOrder {
    pub seq_id: u128,
    pub order_id: String,
    pub quote: String,
    pub price: f64,
    pub size: u64,
    pub side: Side,
    pub order_type: OrderType,
}


impl From<RawSequencedOrder> for RawOrder{
    fn from(value: RawSequencedOrder) -> Self {
        let side= if value.side {Side::BID} else {Side::ASK};
        let order_type= if value.order_type { OrderType::LIMIT} else {OrderType::MARKET};

        Self{
            seq_id:value.seq_id,
            order_id:value.order_id,
            quote:value.quote,
            price:value.price,
            size:value.size,
            side,
            order_type,
        }
    }
}

#[derive(Clone)]
pub struct Order {
    pub seq_id: u128,
    pub order_id: String,
    pub quote: String,
    pub price: f64,
    pub size: u64,
    pub side: Side,
    pub order_type: OrderType,
    pub prev: Option<Weak<RefCell<Order>>>,
    pub next: Option<Rc<RefCell<Order>>>,
}

impl Debug for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Order")
            .field("sequence ID", &self.seq_id)
            .field("order ID", &self.order_id)
            .field("quote", &self.quote)
            .field("price", &self.price)
            .field("size", &self.size)
            .field("side", &self.side)
            .field("type", &self.order_type)
            .finish()
    }
}

impl From<RawOrder> for Order {
    fn from(value: RawOrder) -> Order {
        Order {
            seq_id: value.seq_id.to_owned(),
            order_id: value.order_id.to_owned(),
            quote: value.quote.to_owned(),
            price: value.price.to_owned(),
            size: value.size.to_owned(),
            side: value.side.to_owned(),
            order_type: value.order_type.to_owned(),
            prev: None,
            next: None,
        }
    }
}

