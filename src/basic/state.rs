use std::net::IpAddr;
use crate::basic::common_types::Term;
use crate::basic::log::RaftLog;
use crate::basic::messages::OutputMessage;


// ========= DATA ===========

pub enum NodeState {
    LeaderState(LeaderState),
    CandidateState(CandidateState),
    FollowerState(FollowerState),
}


pub struct LeaderState{
    pub common: Common,
}

pub struct CandidateState{
    // count of votes that this node got in the current term
    pub votes_count: usize,
    pub leader_address: Option<NodeAddress>,

    // whether to react to next election timeout trigger
    // for example, if a follower got a message from leader, it writes here "true", and
    // next time election timeout trigger is fired it will NOT start election
    pub ignore_next_election_timeout_trigger: bool,

    pub common: Common
}

pub struct FollowerState{
    // a raft node that this node voted for
    pub election_favorite_this_term: Option<NodeAddress>,
    pub leader_address: Option<NodeAddress>,

    // whether to react to next election timeout trigger
    // for example, if a follower got a message from leader, it writes here "true", and
    // next time election timeout trigger is fired it will NOT start election
    pub ignore_next_election_timeout_trigger: bool,

    pub common: Common
}

pub struct Common {
    pub term: Term,
    pub log: RaftLog,

    pub other_nodes: OtherNodes,
    pub this_node_address: NodeAddress,
}


pub struct OtherNodes {
    addresses: Vec<NodeAddress>,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct NodeAddress {
    ip_address: IpAddr,
}

pub enum Either<TLeft: Sized, TRight: Sized> {
    Left(TLeft),
    Right(TRight)
}




// ========= LOGIC ===========

impl OtherNodes {
    // todo: think about usize
    /// returns number of other nodes in cluster. Total number of nodes is count() + 1
    pub fn count(&self) -> usize {
        return self.addresses.len()
    }
}


impl FollowerState {
    fn init(
        other_nodes: OtherNodes,
        this_node_address: NodeAddress,
    ) -> Self {
        FollowerState {
            election_favorite_this_term: None,
            leader_address: None,
            ignore_next_election_timeout_trigger: false,
            common: Common::init(
                other_nodes,
                this_node_address,
            ),
        }
    }
}

impl Common {
    fn init(
        other_nodes: OtherNodes,
        this_node_address: NodeAddress,
    ) -> Self {
        Common {
            term: Term::default(),
            other_nodes,
            this_node_address,
            log: RaftLog::default(),
        }
    }
}

impl NodeState {
    // every node starts as follower according to the paper
    pub fn init(
        other_nodes: OtherNodes,
        this_node_address: NodeAddress,
    ) -> Self {
        Self::FollowerState(FollowerState::init(
            other_nodes,
            this_node_address,
        ))
    }
}


// ===== From impls =============

impl From<LeaderState> for FollowerState {
    fn from(leader: LeaderState) -> Self {
        FollowerState{
            election_favorite_this_term: None,
            leader_address: None,
            ignore_next_election_timeout_trigger: false,
            common: leader.common
        }
    }
}

impl From<CandidateState> for FollowerState {
    fn from(candidate: CandidateState) -> Self {
        FollowerState{
            election_favorite_this_term: None,
            leader_address: None,
            ignore_next_election_timeout_trigger: false,
            common: candidate.common
        }
    }
}