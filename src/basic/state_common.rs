use std::net::IpAddr;
use std::time::{Duration, Instant};

use super::{candidate, follower, leader};

// todo: rename
pub enum NodeState {
    LeaderState(leader::LeaderState),
    CandidateState(candidate::CandidateState),
    FollowerState(follower::FollowerState),
}


impl NodeState {
    // every node starts as follower according to the paper
    pub fn init(
        other_nodes: OtherNodes,
        this_node_address: NodeAddress,
    ) -> Self {
        Self::FollowerState(follower::FollowerState {
            node_state: Common {
                term: 0u64,
                last_committed_log_idx: 0u64,
                other_nodes,
                election_favorite_this_term: None,
                this_node_address,
                ignore_next_election_timeout_trigger: false,
            }
        })
    }
}

pub struct Common {
    pub term: u64,
    pub last_committed_log_idx: u64,

    // a raft node that this node voted for
    pub election_favorite_this_term: Option<NodeAddress>,

    // whether to react to next election timeout trigger
    // for example, if a follower got a message from leader, it writes here "true", and
    // next time election timeout trigger is fired it will NOT start election
    pub ignore_next_election_timeout_trigger: bool,

    pub other_nodes: OtherNodes,
    pub this_node_address: NodeAddress,
}


pub struct OtherNodes {
    addresses: Vec<NodeAddress>,
}

#[derive(Copy, Clone)]
pub struct NodeAddress {
    ip_address: IpAddr,
    idx: u8,    // used to distinguish nodes with same ip addr (on 1 host). 0 is default value
}
