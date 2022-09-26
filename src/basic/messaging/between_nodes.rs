use crate::basic::state_common;

// todo: rename
pub trait NodesConnection {
    fn send_message(&self, others: &state_common::OtherNodes, message: NodesMessages) -> ();
    fn poll_message(&self) -> Option<NodesMessages>;
}

pub enum NodesMessages {
    Heartbeat,
    AppendEntries,
    RequestVote { sender_addr: state_common::NodeAddress },
}