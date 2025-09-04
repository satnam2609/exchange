use anyhow::Ok;
use bincode;
use memmap::MmapQueue;
use sequencer::ExecuteMessage;

use std::fs;
use std::result::Result;
use std::thread::sleep;
use std::time::Duration;

use memmap::engseq::RawSequencedOrder;

fn tmp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("mmap_queue_{}.dat", name))
}

fn size_of_raworder() -> usize {
    std::mem::size_of::<RawSequencedOrder>()
}



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut inbound_queue = MmapQueue::create(tmp_path("inbound"), 1024, size_of_raworder())?;
    let mut outbound_queue = MmapQueue::create(tmp_path("outbound"), 1024, size_of_raworder())?;

    let raw_order = RawSequencedOrder::default()
        .with_seq_id(1000)
        .with_order_id("ORDER1000".into())
        .with_quote("BTCETH".into())
        .with_price(10000.99)
        .with_size(12)
        .with_side(true)
        .with_order_type(true)
        .to_owned();

    let _ = inbound_queue.enqueue(&bincode::serialize(&raw_order).unwrap());

    sleep(Duration::from_millis(10000));

    if let Result::Ok(Some(v)) = outbound_queue.dequeue() {
        let event = bincode::deserialize::<ExecuteMessage>(&v).unwrap();
        println!("Event: {:?}",event);
    }

    Ok(())
}
