use core_utils::{ExecuteMessage, Execution, RawOrder, Side};
use lob::*;
use memmap::MmapQueue;
use std::thread::sleep;
use std::time::Duration;

fn tmp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("mmap_queue_{}.dat", name))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut inbound_queue = MmapQueue::open(tmp_path("inbound"))?;
    let mut outbound_queue = MmapQueue::open(tmp_path("outbound"))?;
    let (tx, rx) = crossbeam::channel::unbounded::<RawOrder>();

    std::thread::spawn(move || {
        let mut lob = LimitOrderBook::from(String::from("Book"));
        for mut seq_order in rx {
            let mut outorder_execution = ExecuteMessage::new(seq_order.seq_id, Execution::INSERTED);
            let side = seq_order.side;
            let other_side = match seq_order.side {
                Side::BID => lob.best_ask.clone(),
                Side::ASK => lob.best_bid.clone(),
            };

            match other_side {
                Some(order) => {
                    let is_match = match order.borrow().side {
                        Side::ASK => seq_order.price >= order.borrow().price,
                        Side::BID => order.borrow().price >= seq_order.price,
                    };
                    // if match found
                    if is_match {
                        // Evalute the quantity to trade
                        let quantity_to_trade = std::cmp::min(order.borrow().size, seq_order.size);
                        let mut inorder_execution = ExecuteMessage::new(
                            order.borrow().seq_id,
                            Execution::PARTIAL(order.borrow().price, quantity_to_trade),
                        );

                        // trade orders
                        seq_order.size -= quantity_to_trade;

                        order.borrow_mut().size -= quantity_to_trade;

                        if order.borrow().size == 0 {
                            lob.remove(order.borrow().order_id.clone());
                            lob.update_best(order.borrow().side);
                            inorder_execution.set_execution(Execution::FILL);
                        }

                        // emit inorder execution
                        let _ = outbound_queue.enqueue(&inorder_execution.as_bytes());
                    }

                    if seq_order.size != 0 {
                        lob.insert(RawOrder::from(seq_order));
                        lob.update_best(side);
                    }
                }
                None => {
                    // Insert order directly
                    lob.insert(RawOrder::from(seq_order));
                    // update the best side order that
                    // belongs to this order's side.
                    lob.update_best(side);

                    outorder_execution.set_execution(Execution::INSERTED);
                }
            }

            // emit execution event.
            let _ = outbound_queue.enqueue(&outorder_execution.as_bytes());
        }
    });

    loop {
        while let Ok(Some(s)) = inbound_queue.dequeue() {
            let msg: RawOrder = bincode::deserialize(&s).unwrap();
            let _ = tx.send(msg);
        }

        sleep(Duration::from_millis(5));
    }
}
