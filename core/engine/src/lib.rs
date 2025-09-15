use anyhow::{anyhow, Ok};
use core_utils::{ExecuteMessage, Execution, RawOrder, Side};
use crossbeam::channel::Receiver;
use lob::LimitOrderBook;
use memmap::MmapQueue;

pub fn tmp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("mmap_queue_{}.dat", name))
}

pub struct MatchingEngine {
    pub quote: String,
    pub inbound_queue: *mut MmapQueue,
    pub outbound_queue: *mut MmapQueue,
}



impl MatchingEngine {
    pub fn new(quote: String) -> anyhow::Result<Self> {
        let inbound = MmapQueue::open(tmp_path(&format!("{}-inbound", quote)))?;
        let outbound = MmapQueue::open(tmp_path(&format!("{}-outbound", quote)))?;

        
        Ok(Self {
            quote: quote.clone(),
            inbound_queue: Box::into_raw(Box::new(inbound)),
            outbound_queue: Box::into_raw(Box::new(outbound)),
        })
    }

    pub fn get_inbound(&self)->anyhow::Result<&mut MmapQueue>{
        if let Some(queue)=unsafe{self.inbound_queue.as_mut()}{
            return Ok(queue)
        }

        Err(anyhow!("Inbound queue is null pointer"))
    }

    pub fn get_outbound(&self)->anyhow::Result<&mut MmapQueue>{
        if let Some(queue)=unsafe{self.outbound_queue.as_mut()}{
            return Ok(queue)
        }

        Err(anyhow!("Inbound queue is null pointer"))
    }

    pub fn run(&self, rx: Receiver<RawOrder>) -> anyhow::Result<()> {
        if self.outbound_queue.is_null() {
            return Err(anyhow!("Outbound queue is a null pointer"));
        }
        let outbound_queue = unsafe { self.outbound_queue.as_mut() }.unwrap();

        let quote = self.quote.clone();
        std::thread::spawn(move || {
            let mut lob = LimitOrderBook::from(quote);
            for mut seq_order in rx {
                let mut outorder_execution =
                    ExecuteMessage::new(seq_order.seq_id, Execution::INSERTED);
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
                            let quantity_to_trade =
                                std::cmp::min(order.borrow().size, seq_order.size);
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

        Ok(())
    }
}

impl Drop for MatchingEngine {
    fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(self.inbound_queue) };
        let _ = unsafe { Box::from_raw(self.outbound_queue) };
    }
}
