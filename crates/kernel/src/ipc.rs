use crate::scheduler::TaskId;

#[derive(Clone, Copy)]
pub struct Message {
    pub from: TaskId,
    pub to: TaskId,
    pub payload: [u8; 64],
    pub secure_channel: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum IpcError {
    PermissionDenied,
    InsecureChannelRequired,
}
