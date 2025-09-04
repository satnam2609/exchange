///
/// This struct is will be used to send data between sequencer and the order manager.
/// As the Order manager takes the order request from the clients and do it's job, and after that enqueue 
/// the order into the memmory mapped queue and the sequencer just dequeue's it and does the work.
//
pub struct RawMessage {}
