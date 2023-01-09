use crate::state::{LogEntry, NodeId, RaftTerm, Types};

pub enum Message<TTypes: Types> {
    Input(InputMessage<TTypes>),
    Output(OutputMessage<TTypes>),
}

pub enum InputMessage<TTypes: Types> {
    RaftRequest(RaftRpcReq<TTypes>),
    RaftResponse(RaftRpcResp),
    TimerMsg(TimerMessage),
}

pub enum OutputMessage<TTypes: Types> {
    RaftReq(RaftRpcReq<TTypes>),
    RaftResp(RaftRpcResp),
}


pub enum TimerMessage {
    TriggerHeartbeat,
    TriggerElections,
}


pub enum RaftRpcReq<TTypes: Types> {
    AppendEntries { addressee: NodeId, req: AppendEntriesReq<TTypes> },
    ReqVote(RequestVoteReq),
}

pub enum RaftRpcResp {
    AppendEntries(AppendEntriesResp),
    RequestVote(ReqVoteResp),
}

pub struct AppendEntriesResp {
    pub term: RaftTerm,
    pub success: bool,
    pub sender_id: NodeId,
    pub appended_entries_count: usize,
}

pub struct ReqVoteResp {
    pub term: RaftTerm,
    pub vote_granted: bool,
    pub sender_id: NodeId,
}


pub struct AppendEntriesReq<TTypes: Types> {
    pub leader_id: NodeId,
    pub term: RaftTerm,
    pub leader_commit_idx: usize,

    pub prev_log_idx: usize,
    pub prev_log_term: RaftTerm,

    pub entries_to_append: Vec<LogEntry<TTypes::TCmd>>,
}

pub struct RequestVoteReq {
    pub term: RaftTerm,
    pub candidate_id: NodeId,
    pub last_log_index: usize,
    pub last_log_term: RaftTerm,
}