use std::{
    cell::RefCell,
    fmt::Debug,
    rc::{Rc, Weak},
};

use core_utils::{OrderType, RawOrder, Side};

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
