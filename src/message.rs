use crate::state::{LogEntry, NodeId, RaftTerm, Types};

pub enum InputMessage<TTypes: Types>{
    RaftMessage(RaftRpcReq<TTypes>)
}

pub enum OutputMessage<TTypes: Types> {
    RaftMessage(RaftRpcResp)
}

pub enum RaftRpcReq<TTypes: Types>{
    AppendEntries(AppendEntriesReq<TTypes>)
}

pub struct AppendEntriesReq<TTypes: Types>{
    pub leader_id: NodeId,
    pub term: RaftTerm,
    pub leader_commit_idx: usize,

    pub prev_log_idx: usize,
    pub prev_log_term: RaftTerm,

    pub entries_to_append: Vec<LogEntry<TTypes::TCmd>>,
}

pub enum RaftRpcResp{
    AppendEntries{term: RaftTerm, success: bool}
}