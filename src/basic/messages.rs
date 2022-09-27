use crate::basic::state_common;

/// Interaction of Raft node (state machine) with outer world, including timer
pub trait MessageQueue {
    fn read_message(&self) -> Option<InputMessage>;
    fn push_message(&self, message: OutputMessage) -> ();
}

pub enum InputMessage {
    RaftMsg { message: RaftMessage },
    ControlMessage(ControlMessage)
}

pub enum OutputMessage {
    RaftMsg { message: RaftMessage },
}

/// These messages are described in the Raft paper
/// (one exception: AppendEntries is always used to append entries in log,
///  for heartbeat there is a Heartbeat message)
pub enum RaftMessage {
    Heartbeat,
    AppendEntries,
    RequestVote { sender_addr: state_common::NodeAddress },
}

pub enum ControlMessage {
    // Stop,               // stops the Raft node
    TriggerElection,    // raft node starts elections if its internal state allows it
    TriggerHeartbeat,   // raft node sends heartbeat message if its internal state allows it
}