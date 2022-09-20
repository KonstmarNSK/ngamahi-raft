use std::net::IpAddr;

// todo: rename
pub enum NodeRoleState{
    LeaderState(LeaderState),
    CandidateState(CandidateState),
    FollowerState(FollowerState),
}

pub struct LeaderState{
    pub node_state: NodeState
}

pub struct FollowerState{
    pub node_state: NodeState
}

pub struct CandidateState{
    pub node_state: NodeState
}

pub struct NodeState {
    // status: NodeRole,    checked at type level
    pub term: u64,
    pub last_committed_log_idx: u64,

    pub other_nodes: OtherNodes,
}



pub struct OtherNodes {
    addresses: Vec<NodeAddress>,
}

pub struct NodeAddress {
    ip_address: IpAddr,
    idx: u8,    // used to distinguish nodes with same ip addr (on 1 host). 0 is default value
}
