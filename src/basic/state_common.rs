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
        election_timeout: Duration,
    ) -> Self {
        Self::FollowerState(follower::FollowerState {
            node_state: Common {
                term: 0u64,
                last_committed_log_idx: 0u64,
                other_nodes,
                election_favorite_this_term: None,
                last_time_received_message: Instant::now(),
                this_node_address,
                election_timeout,
            }
        })
    }
}

pub struct Common {
    pub term: u64,
    pub last_committed_log_idx: u64,

    // a raft node that this node voted for
    pub election_favorite_this_term: Option<NodeAddress>,

    // last time this node received any message from others
    pub last_time_received_message: Instant,

    // node starts election if it haven't been receiving any messages for this time
    pub election_timeout: Duration,

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
