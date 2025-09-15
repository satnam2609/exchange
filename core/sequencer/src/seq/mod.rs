use std::mem::size_of;

use anyhow::Ok;
use core_utils::{ExecuteMessage, OrderValue, RawOrder};
use log::info;
use memmap::MmapQueue;

fn tmp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("mmap_queue_{}.dat", name))
}

fn create_queue(path: &str, size: usize) -> anyhow::Result<MmapQueue> {
    MmapQueue::create(tmp_path(path), 1024, size)
}

type WriteHeadLog = *mut MmapQueue; // used specially for a logging

#[derive(Debug)]
pub enum Event {
    In(RawOrder),
    Out(ExecuteMessage),
}

pub struct Sequencer {
    pub quote: String,
    pub inbound_engine: *mut MmapQueue,
    pub outbound_engine: *mut MmapQueue,
    pub inbound_manager: *mut MmapQueue,
    pub write_head_log: WriteHeadLog,
    pub outbound_manager: *mut MmapQueue,
    seq: u128,
}

impl Sequencer {
    pub fn new(quote: &str) -> anyhow::Result<Self> {
        let inbound_engine = create_queue(&format!("{}-inbound", quote), size_of::<RawOrder>())?;
        let outbound_engine =
            create_queue(&format!("{}-outbound", quote), size_of::<ExecuteMessage>())?;
        let inbound_manager = create_queue(
            &format!("{}-inbound-manager", quote),
            size_of::<OrderValue>(),
        )?;

        let outbound_manager = create_queue(
            &format!("{}-outbound-manager", quote),
            size_of::<ExecuteMessage>(),
        )?;

        let write_head_log =
            MmapQueue::create(format!("{}.orders.dat", quote), 4096, size_of::<RawOrder>())?;

        Ok(Sequencer {
            quote: quote.to_string(),
            inbound_engine: Box::into_raw(Box::new(inbound_engine)),
            outbound_engine: Box::into_raw(Box::new(outbound_engine)),
            inbound_manager: Box::into_raw(Box::new(inbound_manager)),
            write_head_log: Box::into_raw(Box::new(write_head_log)),
            outbound_manager: Box::into_raw(Box::new(outbound_manager)),
            seq: 0,
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let inbound_manager = unsafe { self.inbound_manager.as_mut().unwrap() };
        let outbound_manager = unsafe { self.outbound_manager.as_mut().unwrap() };
        let inbound_engine = unsafe { self.inbound_engine.as_mut().unwrap() };
        let outbound_engine = unsafe { self.outbound_engine.as_mut().unwrap() };
        let event_mmap_log = unsafe { self.write_head_log.as_mut().unwrap() };

        loop {
            if let Result::Ok(Some(v)) = inbound_manager.dequeue() {
                let raw_order = bincode::deserialize::<OrderValue>(&v)?.into_raw(self.seq);
                self.seq += 1;
                let payload = bincode::serialize(&raw_order).unwrap();
                event_mmap_log.enqueue(&payload)?;
                info!("{:?}", Event::In(raw_order));
                inbound_engine.enqueue(&payload)?;
            }

            if let Result::Ok(Some(v)) = outbound_engine.dequeue() {
                let execute_msg = bincode::deserialize::<ExecuteMessage>(&v)?;
                info!("{:?}", Event::Out(execute_msg));
                outbound_manager.enqueue(&v)?;
            }
        }
    }
}
