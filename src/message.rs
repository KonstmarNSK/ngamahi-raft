use crate::state::{LogEntry, NodeId, RaftTerm, Types};

#[derive(Clone)]
pub enum Message<TTypes: Types> {
    Input(InputMessage<TTypes>),
    Output(OutputMessage<TTypes>),
}

#[derive(Clone)]
pub enum InputMessage<TTypes: Types> {
    RaftRequest(RaftRpcReq<TTypes>),
    RaftResponse(RaftRpcResp),
    TimerMsg(TimerMessage),
}

#[derive(Clone)]
pub enum OutputMessage<TTypes: Types> {
    RaftReq(RaftRpcReq<TTypes>),
    RaftResp(RaftRpcResp),
}

#[derive(Clone)]
pub enum TimerMessage {
    TriggerHeartbeat,
    TriggerElections,
}

#[derive(Clone)]
pub enum RaftRpcReq<TTypes: Types> {
    AppendEntries { addressee: NodeId, req: AppendEntriesReq<TTypes> },
    ReqVote(RequestVoteReq),
}

#[derive(Clone)]
pub enum RaftRpcResp {
    AppendEntries(AppendEntriesResp),
    RequestVote(ReqVoteResp),
}

#[derive(Clone)]
pub struct AppendEntriesResp {
    pub term: RaftTerm,
    pub success: bool,
    pub sender_id: NodeId,
    pub appended_entries_count: usize,
}

#[derive(Clone)]
pub struct ReqVoteResp {
    pub term: RaftTerm,
    pub vote_granted: bool,
    pub sender_id: NodeId,
}

#[derive(Clone)]
pub struct AppendEntriesReq<TTypes: Types> {
    pub leader_id: NodeId,
    pub term: RaftTerm,
    pub leader_commit_idx: usize,

    pub prev_log_idx: usize,
    pub prev_log_term: RaftTerm,

    pub entries_to_append: Vec<LogEntry<TTypes::TCmd>>,
}

#[derive(Clone)]
pub struct RequestVoteReq {
    pub term: RaftTerm,
    pub candidate_id: NodeId,
    pub last_log_index: usize,
    pub last_log_term: RaftTerm,
}