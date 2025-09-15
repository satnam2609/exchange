use core_utils::{ExecuteMessage, OrderType, RawOrder, Side};
use matching_engine::{tmp_path, MatchingEngine};
use memmap::MmapQueue;
use std::fs::remove_file;

fn create_queues() {
    let _ = MmapQueue::create(
        tmp_path("TEST-inbound"),
        1024,
        std::mem::size_of::<RawOrder>(),
    );
    let _ = MmapQueue::create(
        tmp_path("TEST-outbound"),
        1024,
        std::mem::size_of::<ExecuteMessage>(),
    );
}

#[test]
fn test_engine() {
    create_queues();

    
    let (tx, rx) = crossbeam::channel::unbounded::<RawOrder>();
    let engine = MatchingEngine::new("TEST".into());

    assert!(engine.is_ok());

    let engine = engine.unwrap();

    let inbound = engine.get_inbound();

    assert!(inbound.is_ok());

    

    let order = RawOrder::default()
        .with_seq_id(1)
        .with_order_id("ORDER".into())
        .with_quote("TEST".into())
        .with_price(100.10)
        .with_size(10)
        .with_side(Side::ASK)
        .with_order_type(OrderType::LIMIT)
        .to_owned();

    let send = tx.send(order);
    assert!(send.is_ok());

    let _ = engine.run(rx);

    let outbound = engine.get_outbound();

    assert!(outbound.is_ok());

    let outbound=outbound.unwrap();

    let data = outbound.dequeue();

    assert!(data.is_ok());

    let data = data.unwrap();

    assert!(data.is_some());

    let _ = remove_file(tmp_path("TEST-inbound"));
}
