use anyhow::Result;
use memmap::MmapQueue;
use std::fs;

fn tmp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("mmap_queue_{}.dat", name))
}

#[test]
fn producer_consumer_roundtrip() -> Result<()> {
    let p = tmp_path("test1");
    let _ = fs::remove_file(&p);
    // capacity 8 slots, payload 128 bytes max
    let mut prod = MmapQueue::create(&p, 8, 128)?;
    let mut cons = MmapQueue::open(&p)?;

    // push some messages
    for i in 0..5u8 {
        let msg = vec![i; (i + 1) as usize];
        prod.enqueue(&msg)?;
    }

    // pop messages
    for i in 0..5u8 {
        let got = cons.dequeue()?.expect("expected message");
        assert_eq!(got, vec![i; (i + 1) as usize]);
    }

    // empty now
    assert!(cons.dequeue()?.is_none());

    let _ = fs::remove_file(&p);
    Ok(())
}
