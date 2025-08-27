pub mod limit;
pub mod order;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ordered_float::OrderedFloat;
use skiplist::SkipMap;

use crate::{
    limit::Limit,
    order::{Order, RawOrder, Side},
};

/// This struct holds the core logic for managing the pending orders
/// or the orders that are currently not processed by the matching enigne.
pub struct LimitOrderBook {
    pub book_id: String, // The unqiue book id for partionining the exchange
    pub ask_list: SkipMap<OrderedFloat<f64>, Rc<RefCell<Limit>>>, // skip list for storing all the ASK limit nodes.
    pub bid_list: SkipMap<OrderedFloat<f64>, Rc<RefCell<Limit>>>, // skip list for storing all the BID limit nodes.
    pub ask_map: HashMap<OrderedFloat<f64>, Rc<RefCell<Limit>>>, // hash map for fast lookups for the ASK limit nodes.
    pub bid_map: HashMap<OrderedFloat<f64>, Rc<RefCell<Limit>>>, // hash map for fast loopups for the BID limit nodes.
    pub ord_map: HashMap<String, Rc<RefCell<Order>>>, // hash map for fast lookups for all the Orders in the limit order book
    pub best_ask: Option<Rc<RefCell<Order>>>, // A reference to the best ASK order, typically the front node's head order in the ASK skip list.
    pub best_bid: Option<Rc<RefCell<Order>>>, // A reference to the best BID order, typically the back node's head order in the BID skip list.
}

impl From<String> for LimitOrderBook {
    fn from(value: String) -> LimitOrderBook {
        LimitOrderBook {
            book_id: value,
            ask_list: SkipMap::new(),
            bid_list: SkipMap::new(),
            ask_map: HashMap::new(),
            bid_map: HashMap::new(),
            ord_map: HashMap::new(),
            best_ask: None,
            best_bid: None,
        }
    }
}

// Now here comes the implementation of the limit order book.
impl LimitOrderBook {
    /// Insert method does some series of work and inserts the order from the raw order.
    /// ```rust
    /// // limit order book generation from the unique book id
    /// let mut limit_order_book= lob::LimitOrderBook::from(String::from("12"));
    /// assert!(limit_order_book.best_ask.is_none());
    /// assert!(limit_order_book.best_bid.is_none());
    /// // create a raw order and then pass to the order book for insertion
    /// let raw_order=lob::order::RawOrder{ seq_id:"1".into(),order_id:"12121".into(),quote:"BTCINR".into(),price:1000.11, size: 10,side: lob::order::Side::BID, order_type:lob::order::OrderType::LIMIT };
    ///
    /// limit_order_book.insert(raw_order);
    ///
    /// // assert for the best order updation.
    /// assert!(limit_order_book.best_ask.is_none());
    /// assert!(!limit_order_book.best_bid.is_none());
    /// ```
    pub fn insert(&mut self, raw_order: RawOrder) {
        let price = raw_order.price.clone();
        // generates the order from the raw order
        let order = Rc::new(RefCell::new(Order::from(raw_order)));
        // gets the relevant list and the map as the mutable reference.
        let (list, map) = match order.borrow().side {
            Side::ASK => (&mut self.ask_list, &mut self.ask_map),
            Side::BID => (&mut self.bid_list, &mut self.bid_map),
        };
        // if the limit node already exists then fetch from the map or else insert the limit node in the skip list and also insert in map
        // then finally get the limit node.
        let limit = map.entry(OrderedFloat(price)).or_insert_with(|| {
            let limit = Rc::new(RefCell::new(Limit::new(price)));
            list.insert(OrderedFloat(price), limit.clone());
            limit.clone()
        });
        // if the tail of this limit node is None that means, the limit node was created now only
        // and hence update the head and tail of the limit node as the current node.
        // else update the generated order as the new tail and update the prev and prev tail's next pointer.
        // ofcourse we have to update the total volume in the limit node.
        let mut limit_mut_borrowed = limit.borrow_mut();
        limit_mut_borrowed.vol += order.borrow().size.clone();
        if let Some(ref mut tail) = limit_mut_borrowed.tail {
            tail.borrow_mut().next = Some(order.clone());
            order.borrow_mut().prev = Some(Rc::downgrade(&tail));
        } else {
            limit_mut_borrowed.head = Some(order.clone());
        }
        limit_mut_borrowed.tail = Some(order.clone());

        // if the best order (ASK or BID) is empty or None then update this order as the
        // best order from the relevant side.
        match order.borrow().side {
            Side::ASK => {
                if self.best_ask.is_none() {
                    self.best_ask = Some(order.clone());
                }
            }
            Side::BID => {
                if self.best_bid.is_none() {
                    self.best_bid = Some(order.clone());
                }
            }
        };

        // finally, insert the order in the order map for fast lookups.
        self.ord_map
            .insert(order.borrow().order_id.clone(), order.clone());
    }

    /// This method returns the total volume at particular limit price.
    /// ```rust
    /// let mut limit_order_book= lob::LimitOrderBook::from(String::from("1"));
    /// let raw_order=lob::order::RawOrder{ seq_id:"1".into(),order_id:"order_id_10232".into(),quote:"BTCINR".into(),price:1000.11, size: 10,side: lob::order::Side::BID, order_type:lob::order::OrderType::LIMIT };
    ///
    /// limit_order_book.insert(raw_order);
    /// let depth=limit_order_book.depth(lob::order::Side::BID,1000.11);
    /// assert!(depth.is_some());
    /// assert_eq!(depth.unwrap(),10);
    /// ```
    pub fn depth(&self, side: Side, limit: f64) -> Option<u64> {
        let map = match side {
            Side::ASK => &self.ask_map,
            Side::BID => &self.bid_map,
        };

        if let Some(node) = map.get(&OrderedFloat(limit)) {
            return Some(node.borrow().vol.clone());
        }
        None
    }

    /// This method removes the order from the book.
    // For now I have to figure out what must be returned.
    ///```rust
    /// let mut book= lob::LimitOrderBook::from(String::from("BOOK"));
    /// let raw_order=lob::order::RawOrder{ seq_id:"1".into(),order_id:"order_id_10232".into(),quote:"BTCINR".into(),price:1000.11, size: 10,side: lob::order::Side::BID, order_type:lob::order::OrderType::LIMIT };
    /// book.insert(raw_order);
    ///
    /// let depth=book.depth(lob::order::Side::BID,1000.11);
    /// assert!(depth.is_some());
    /// assert_eq!(depth.unwrap(),10);
    /// // removing the order now
    /// book.remove("order_id_10232".into());
    /// // since the order has been removed now, so the total volume
    /// // within that limit node must be reduced to the intial volume.
    /// let depth=book.depth(lob::order::Side::BID,1000.11);
    /// assert!(depth.is_none());
    /// ```
    //
    pub fn remove(&mut self, order_id: String) {
        // try to remove the order from the order map
        if let Some(ref order) = self.ord_map.remove(&order_id) {
            // then take the prev order and the next order,
            // so now the order does not have prev or next pointer.
            let prev_order = order.borrow_mut().prev.take();
            let next_order = order.borrow_mut().next.take();

            // to remove the order from the doubly linked list,
            // update the next of prev's order as the next of the order that
            // is been removed.
            if let Some(prev) = prev_order.clone() {
                if prev.upgrade().is_some() {
                    prev.upgrade().unwrap().borrow_mut().next = next_order.clone();
                } else {
                    // upgrade failed
                }
            }

            // similarly, update the prev of next's order as the prev of the order
            // that is been removed.
            if let Some(next) = next_order.clone() {
                next.borrow_mut().prev = prev_order.clone();
            }

            let map = match order.borrow().side {
                Side::ASK => &mut self.ask_map,
                Side::BID => &mut self.bid_map,
            };

            // update the total volume of the limit node by substracting the size of the removed order.
            if let Some(limit) = map.get(&OrderedFloat(order.borrow().price)) {
                limit.borrow_mut().vol -= order.borrow().size.clone();
            }

            // if the prev and next are None then that means the limit node is empty
            // hence remove the limit node from the map and the skip list.
            if prev_order.is_none() && next_order.is_none() {
                map.remove(&OrderedFloat(order.borrow().price.clone()));
                let list = match order.borrow().side {
                    Side::ASK => &mut self.ask_list,
                    Side::BID => &mut self.bid_list,
                };

                list.remove(&OrderedFloat(order.borrow().price));
            } else if prev_order.is_none() && next_order.is_some() {
                if let Some(limit) = map.get(&OrderedFloat(order.borrow().price.clone())) {
                    limit.borrow_mut().head = next_order;
                }
            } else if prev_order.is_some() && next_order.is_none() {
                if let Some(limit) = map.get(&OrderedFloat(order.borrow().price.clone())) {
                    limit.borrow_mut().tail = next_order;
                }
            }

            // figure something what is to be returned,
            // so that the order manager or the sequencer can
            // emit event as Cancelled.
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_lob() -> LimitOrderBook {
        LimitOrderBook::from(String::from("LIMITORDERBOOK"))
    }

    #[test]
    fn insertion() {
        let mut lob = create_lob();
        let raw_order = RawOrder {
            seq_id: "1".into(),
            order_id: "ORDER1".into(),
            quote: "BTCETH".into(),
            price: 100.10,
            size: 10,
            side: Side::ASK,
            order_type: order::OrderType::LIMIT,
        };

        lob.insert(raw_order);

        assert_eq!(lob.ask_list.len(), 1);
        assert_eq!(lob.bid_list.len(), 0);
        assert_eq!(lob.ask_map.len(), 1);
        assert_eq!(lob.bid_map.len(), 0);
        assert_eq!(lob.ord_map.len(), 1);
        assert!(lob.best_ask.is_some());
    }

    #[test]
    fn insertion_on_same_limit() {
        let mut lob = create_lob();
        for i in 0..10 {
            let raw_order = RawOrder {
                seq_id: format!("{:?}", i),
                order_id: format!("ORDER{:?}", i),
                quote: "BTCETH".into(),
                price: 100.10,
                size: 10,
                side: Side::ASK,
                order_type: order::OrderType::LIMIT,
            };

            lob.insert(raw_order);
        }

        assert_eq!(lob.ask_list.len(), 1);
        assert_eq!(lob.bid_list.len(), 0);
        assert_eq!(lob.ask_map.len(), 1);
        assert_eq!(lob.bid_map.len(), 0);
        assert_eq!(lob.ord_map.len(), 10);

        let limit = lob.ask_map.get(&OrderedFloat(100.10)).unwrap();
        assert_eq!(limit.borrow().vol, 100);
    }

    #[test]
    fn insertion_on_different_limit() {
        let mut lob = create_lob();
        for i in 0..10 {
            let raw_order = RawOrder {
                seq_id: format!("{:?}", i),
                order_id: format!("ORDER{:?}", i),
                quote: "BTCETH".into(),
                price: 100.10 + i as f64,
                size: 10,
                side: Side::ASK,
                order_type: order::OrderType::LIMIT,
            };

            lob.insert(raw_order);
        }

        assert_eq!(lob.ask_list.len(), 10);
        assert_eq!(lob.bid_list.len(), 0);
        assert_eq!(lob.ask_map.len(), 10);
        assert_eq!(lob.bid_map.len(), 0);
        assert_eq!(lob.ord_map.len(), 10);
    }

    #[test]
    fn removal() {
        let mut lob = create_lob();

        let raw_order = RawOrder {
            seq_id: "1".into(),
            order_id: "ORDER1".into(),
            quote: "BTCETH".into(),
            price: 100.10,
            size: 10,
            side: Side::ASK,
            order_type: order::OrderType::LIMIT,
        };

        lob.insert(raw_order);

        assert_eq!(lob.ask_list.len(), 1);
        assert_eq!(lob.bid_list.len(), 0);
        assert_eq!(lob.ask_map.len(), 1);
        assert_eq!(lob.bid_map.len(), 0);
        assert_eq!(lob.ord_map.len(), 1);
        assert!(lob.best_ask.is_some());

        lob.remove("ORDER1".into());

        assert_eq!(lob.ask_list.len(), 0);
        assert_eq!(lob.bid_list.len(), 0);
        assert_eq!(lob.ask_map.len(), 0);
        assert_eq!(lob.bid_map.len(), 0);
        assert_eq!(lob.ord_map.len(), 0);
        assert!(lob.best_ask.is_some());
    }

    #[test]
    fn removal_on_same_limit() {
        let mut lob = create_lob();

        for i in 0..10 {
            let raw_order = RawOrder {
                seq_id: format!("{:?}", i),
                order_id: format!("ORDER{:?}", i),
                quote: "BTCETH".into(),
                price: 100.10,
                size: 10,
                side: Side::ASK,
                order_type: order::OrderType::LIMIT,
            };

            lob.insert(raw_order);
        }

        assert_eq!(lob.ask_list.len(), 1);
        assert_eq!(lob.bid_list.len(), 0);
        assert_eq!(lob.ask_map.len(), 1);
        assert_eq!(lob.bid_map.len(), 0);
        assert_eq!(lob.ord_map.len(), 10);

        let limit = lob.ask_map.get(&OrderedFloat(100.10)).unwrap().clone();
        assert_eq!(limit.borrow().vol, 100);

        assert!(limit.borrow().head.is_some());
        let head_order = limit.borrow().head.clone().unwrap();
        assert_eq!(head_order.borrow().order_id, String::from("ORDER0"));
        // removing the first order from the limit node.
        lob.remove("ORDER0".into());

        assert_eq!(lob.ask_list.len(), 1);
        assert_eq!(lob.bid_list.len(), 0);
        assert_eq!(lob.ask_map.len(), 1);
        assert_eq!(lob.bid_map.len(), 0);
        assert_eq!(lob.ord_map.len(), 9);

        assert_eq!(limit.borrow().vol, 90);
        let head_order = limit.borrow().head.clone().unwrap();
        assert_eq!(head_order.borrow().order_id, String::from("ORDER1"));
    }
}
