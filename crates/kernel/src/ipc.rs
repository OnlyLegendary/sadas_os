use crate::scheduler::TaskId;

#[derive(Clone, Copy)]
pub struct Message {
    pub from: TaskId,
    pub to: TaskId,
    pub payload: [u8; 32],
}

#[derive(Debug, Eq, PartialEq)]
pub enum IpcError {
    PermissionDenied,
}
