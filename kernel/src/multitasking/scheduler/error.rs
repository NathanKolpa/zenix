#[derive(Debug)]
pub enum SchedulerError {
    OutOfMemory,
    ThreadLimit,
    SlotTaken,
}
