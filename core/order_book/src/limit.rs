use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::order::Order;

#[derive(Clone)]
pub struct Limit {
    pub price: f64,
    pub vol: u64,
    pub head: Option<Rc<RefCell<Order>>>,
    pub tail: Option<Rc<RefCell<Order>>>,
}

impl Debug for Limit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Limit Node")
            .field("price", &self.price)
            .field("volume", &self.vol)
            .finish()
    }
}

impl Limit {
    pub fn new(price: f64) -> Limit {
        Limit {
            price,
            vol: 0,
            head: None,
            tail: None,
        }
    }
}
