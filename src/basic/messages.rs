use crate::basic::common_types::{LogMsgIdx, Term};
use crate::basic::log::LogEntry;
use crate::basic::state;
use crate::basic::state::NodeAddress;

/// Interaction of Raft node (state machine) with outer world, including timer
pub trait MessageQueue {
    fn read_message(&self) -> Option<InputMessage>;
    fn push_message(&self, message: OutputMessage) -> ();
}


pub enum OutputMessage {
    RaftMsg { message: RaftMessage },
}

pub enum InputMessage {
    ControlMessage(ControlMessage),
    RaftMsg(RaftMessage),
}


pub enum ControlMessage {
    TriggerElection,    // raft node starts election if its internal state allows it
    TriggerHeartbeat,   // raft node sends heartbeat message if its internal state allows it
}


/// These messages are described in the Raft paper
pub enum RaftMessage {
    AppendEntries(AppendEntriesMsg),
    AppendEntriesResp(AppendEntriesResp),    // response to AppendEntries

    RequestVote(RequestVoteMsg)
}

pub struct AppendEntriesResp {
    pub success: bool,
    pub sender_term: Term
}

pub struct AppendEntriesMsg {
    pub sender_term: Term,
    pub leader_id: NodeAddress,
    pub content: Vec<LogEntry>,
    pub prev_log_index: LogMsgIdx,
    pub prev_log_term: Term,
    pub leader_commit: LogMsgIdx,
}

pub struct RequestVoteMsg {
    pub sender_term: Term,
    pub sender_addr: NodeAddress,
}


pub fn get_term(msg: &RaftMessage) -> Term {
    return match msg {
        RaftMessage::RequestVote ( RequestVoteMsg{ sender_term, .. } )
        | RaftMessage::AppendEntriesResp( AppendEntriesResp {sender_term, ..})
        | RaftMessage::AppendEntries( AppendEntriesMsg { sender_term, .. } ) => sender_term.to_owned()
    };
}

/*
Arguments:
term leader’s term
leaderId so follower can redirect clients
prevLogIndex index of log entry immediately preceding
new ones
prevLogTerm term of prevLogIndex entry
entries[] log entries to store (empty for heartbeat;
may send more than one for efficiency)
leaderCommit leader’s commitIndex
 */