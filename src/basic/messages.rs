use crate::basic::state;

/// Interaction of Raft node (state machine) with outer world, including timer
pub trait MessageQueue {
    fn read_message(&self) -> Option<InputMessage>;
    fn push_message(&self, message: OutputMessage) -> ();
}

pub enum InputMessage {
    RaftMsg(RaftMessage),
    ControlMessage(ControlMessage),
}

pub enum OutputMessage {
    RaftMsg { message: RaftMessage },
}

/// These messages are described in the Raft paper
pub enum RaftMessage {
    AppendEntries{ sender_term: u64, },
    RequestVote { sender_term: u64,  sender_addr: state::NodeAddress },
}

pub enum ControlMessage {
    TriggerElection,    // raft node starts election if its internal state allows it
    TriggerHeartbeat,   // raft node sends heartbeat message if its internal state allows it
}