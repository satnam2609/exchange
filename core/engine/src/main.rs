use core_utils::RawOrder;

use std::thread::sleep;
use std::time::Duration;

use crate::matching_engine::MatchingEngine;

pub mod matching_engine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matching_engine = MatchingEngine::new("BTCETH".into())?;
    let (tx, rx) = crossbeam::channel::unbounded::<RawOrder>();

    let inbound_queue = matching_engine.get_inbound()?;

    matching_engine.run(rx.clone())?;

    loop {
        while let Ok(Some(s)) = inbound_queue.dequeue() {
            let msg: RawOrder = bincode::deserialize(&s).unwrap();
            let _ = tx.send(msg);
        }

        sleep(Duration::from_millis(5));
    }
}
