use std::{
    fs::{File, OpenOptions},
    path::Path,
    ptr,
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{bail, Context, Ok, Result};
use memmap2::{MmapMut, MmapOptions};

const MAGIC: u64 = 0x4D514D50524F4451; // magic number

pub mod engseq;
pub mod seqman;

/// Layout in the mmap file:
/// [ Header (aligned) ] [ slot0 ][ slot1 ]...[ slotN-1 ]
#[repr(C)]
pub struct Header {
    magic: u64,
    capacity: u64,  // size of the queue
    slot_size: u64, // size of element within the queue
    head: AtomicU64, // front index
    tail: AtomicU64, // back index
    mask: u64, // mask for getting the correct index
}

impl Header {
    pub fn size() -> usize {
        std::mem::size_of::<Header>()
    }
}

/// A memory-mapped mmapped single-producer/single-consumer queue.
pub struct MmapQueue {
    pub file: File,
    mmap: MmapMut,
    header_ptr: *mut Header,
    data_offset: usize,
    capacity: usize,
    slot_size: usize,
    mask: usize,
}

unsafe impl Send for MmapQueue {}
unsafe impl Sync for MmapQueue {}

impl MmapQueue {
    /// Create and initialize a new queue file at `path`.
    /// capacity must be a power of two.
    /// slot_payload_size is the max payload size (u32 length prefix is added automatically).
    pub fn create<P: AsRef<Path>>(
        path: P,
        capacity: usize,
        slot_payload_size: usize,
    ) -> Result<Self> {
        if !capacity.is_power_of_two() {
            bail!("capacity must be power of two");
        }

        let slot_size = 4usize + slot_payload_size;
        let header_size = Header::size();
        let total_size = header_size + capacity * slot_size;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.as_ref())
            .with_context(|| format!("create/truncate file {:?}", path.as_ref()))?;

        file.set_len(total_size as u64).context("set_len failed")?;

        let mut mmap = unsafe { MmapOptions::new().len(total_size).map_mut(&file)? };

        let header_ptr = mmap.as_mut_ptr() as *mut Header;

        unsafe {
            ptr::write_bytes(header_ptr as *mut u8, 0, header_size);

            let hdr = &mut *header_ptr;
            hdr.magic = MAGIC;
            hdr.capacity = capacity as u64;
            hdr.slot_size = slot_size as u64;
            hdr.mask = (capacity - 1) as u64;
            // AtomicU64 fields default to zero (head/tail)
            // ensure head/tail are zeroes already
            hdr.head.store(0, Ordering::Relaxed);
            hdr.tail.store(0, Ordering::Relaxed);
        }
        Ok(Self {
            file,
            mmap,
            header_ptr,
            data_offset: header_size,
            capacity,
            slot_size,
            mask: capacity - 1,
        })
    }

    /// Open and get direct access to the memory mapped file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.as_ref())?;
        let metadata = file.metadata()?;
        let total_size = metadata.len() as usize;

        if total_size < Header::size() {
            bail!("file is too small to be a queue")
        }

        let mut mmap = unsafe { MmapOptions::new().len(total_size).map_mut(&file) }?;

        let header_ptr = mmap.as_mut_ptr() as *mut Header;

        unsafe {
            let hdr = &*header_ptr;
            if hdr.magic != MAGIC {
                bail!("magic mismatch; file is not a valid queue or corrupted")
            }

            let capacity = hdr.capacity as usize;
            let slot_size = hdr.slot_size as usize;
            let header_size = Header::size();
            let expected = header_size + capacity * slot_size;
            if expected != total_size {
                bail!(
                    "file size doesn't match header info (expected {}, got {})",
                    expected,
                    total_size
                );
            }

            Ok(Self {
                file,
                mmap,
                header_ptr,
                data_offset: header_size,
                capacity,
                slot_size,
                mask: (hdr.mask as usize),
            })
        }
    }

    #[inline]
    fn header(&self) -> &Header {
        unsafe { &*self.header_ptr }
    }

    /// This method is one of the core logic of this crate, basically
    /// does some validation about the memory mapped file and then just 
    /// stores the data into the tail index and increments till it reaches the `capacity`
    pub fn enqueue(&mut self, payload: &[u8]) -> Result<()> {
        if payload.len() > self.slot_size - 4 {
            bail!("payload is too large for slot (max {})", self.slot_size - 4)
        }

        // load indexes
        let tail = self.header().tail.load(Ordering::Acquire);
        let head = self.header().head.load(Ordering::Acquire);

        let next_tail = tail.wrapping_add(1);

        if next_tail.wrapping_sub(head) as usize > self.capacity {
            bail!("queue is overflowed")
        }

        let idx = (tail as usize) & self.mask;
        let slot_offset = self.data_offset + idx * self.slot_size;

        let len_ptr = unsafe { self.mmap.as_mut_ptr().add(slot_offset) as *mut u32 };
        let buf_ptr = unsafe { self.mmap.as_mut_ptr().add(slot_offset + 4) };

        // write
        unsafe {
            ptr::write_unaligned(len_ptr, payload.len() as u32);

            ptr::copy_nonoverlapping(payload.as_ptr(), buf_ptr, payload.len());

            if payload.len() < self.slot_size - 4 {
                let extra = self.slot_size - 4 - payload.len();
                let rem_ptr = buf_ptr.add(payload.len());
                ptr::write_bytes(rem_ptr, 0, extra);
            }
        }

        // publish by incrementing tail (release)
        self.header().tail.store(next_tail, Ordering::Release);

        Ok(())
    }

    /// This method is the second most important in this crate, it just takes the `head` index and 
    /// tries to get the payload out of the current slot and increments the `head` index till it reaches 
    /// the current `tail` index.
    pub fn dequeue(&mut self) -> Result<Option<Vec<u8>>> {
        let head = self.header().head.load(Ordering::Acquire);
        let tail = self.header().tail.load(Ordering::Acquire);

        if tail == head {
            return Ok(None);
        }

        let idx = (head as usize) & self.mask;
        let slot_offset = self.data_offset + idx * self.slot_size;

        let len_ptr = unsafe { self.mmap.as_mut_ptr().add(slot_offset) as *mut u32 };
        let buf_ptr = unsafe { self.mmap.as_mut_ptr().add(4 + slot_offset) };

        let len = unsafe { ptr::read_unaligned(len_ptr) as u32 } as usize;
        if len > self.slot_size - 4 {
            bail!("corrupted length in slot");
        }

        let mut out = vec![0u8; len];

        unsafe {
            ptr::copy_nonoverlapping(buf_ptr, out.as_mut_ptr(), len);
        }

        let next_head = head.wrapping_add(1);
        self.header().head.store(next_head, Ordering::Release);

        Ok(Some(out))
    }
}
