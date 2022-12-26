use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct RaftTerm(u64);

pub struct NodeId {
    pub address: IpAddr,
    pub cluster_id: u64,
}

pub struct LogEntry<TCmd: Command> {
    pub cmd: TCmd,
    pub term: RaftTerm,
}

pub trait Command {}

pub trait Types {
    type TCmd: Command;
}

pub type RaftLog<TTypes: Types> = Vec<LogEntry<TTypes::TCmd>>;

pub struct PersistentCommonState<TTypes: Types> {
    pub this_node_id: NodeId,
    pub current_term: RaftTerm,
    pub voted_for: Option<NodeId>,
    pub log: RaftLog<TTypes>,

    pub last_msg_term: RaftTerm, // raft paper doesn't contain this. Here we remember term of last msg in log
}

impl<TTypes: Types> PersistentCommonState<TTypes> {
    fn new(node_id: NodeId) -> Self {
        PersistentCommonState {
            this_node_id: node_id,
            current_term: RaftTerm(0),
            voted_for: None,
            log: vec![],

            last_msg_term: RaftTerm(0)
        }
    }
}

#[derive(Default)]
pub struct VolatileCommonState {
    pub committed_idx: usize,
    pub last_applied: usize,
}

pub struct CommonState<TTypes: Types> {
    pub common_persistent: PersistentCommonState<TTypes>,
    pub common_volatile: VolatileCommonState,
}

pub enum State<TTypes: Types> {
    Follower(Follower<TTypes>),
    Candidate(Candidate<TTypes>),
    Leader(Leader<TTypes>),
}

pub struct Follower<TTypes: Types>{
    pub common_state: CommonState<TTypes>
}
pub struct Candidate <TTypes: Types> {
    pub common_state: CommonState<TTypes>
}

pub struct Leader <TTypes: Types> {
    pub common_state: CommonState<TTypes>,

    // following fields must be re-initialized after elections
    pub next_idx: HashMap<NodeId, usize>,
    pub match_idx: HashMap<NodeId, usize>,
}


pub struct InitParams<TTypes: Types> {
    // read from hdd
    pub persisted_state: Option<PersistentCommonState<TTypes>>,
    // this node's id
    pub node_id: NodeId,
}


pub struct RaftNode<TTypes: Types> {
    pub state: State<TTypes>,
}

impl<TTypes: Types> RaftNode<TTypes> {
    pub fn new(node_id: NodeId, mut params: InitParams<TTypes>) -> Self {
        RaftNode{
            state: State::new(node_id, params.persisted_state.take())
        }
    }
}

impl<TTypes: Types> State<TTypes> {
    fn new(node_id: NodeId, persisted_state: Option<PersistentCommonState<TTypes>>) -> Self {
        State {
            cluster_role: State::Follower(Follower::new(node_id, persisted_state))
        }
    }

    pub fn common(&self) -> &CommonState<TTypes> {
        match self {
            Self::Leader(ref state) => &state.common_state,
            Self::Follower(ref state) => &state.common_state,
            Self::Candidate(ref state) => &state.common_state
        }
    }

    //todo: remove code duplication
    pub fn common_mut(&mut self) -> &mut CommonState<TTypes> {
        match self {
            Self::Leader(ref mut state) => &mut state.common_state,
            Self::Follower(ref mut state) => &mut state.common_state,
            Self::Candidate(ref mut state) => &mut state.common_state
        }
    }

    pub fn into_follower(self) -> Self {
        match self {
            Self::Leader(leader_state) => Self::Follower(
                Follower{ common_state: leader_state.common_state }
            ),

            Self::Candidate(candidate_state) => Self::Follower(
                Follower{ common_state: candidate_state.common_state }
            ),

            Self::Follower(_) => Self
        }
    }
}

impl<TTypes: Types> Follower<TTypes> {
    fn new(node_id: NodeId, persisted_state: Option<PersistentCommonState<TTypes>>) -> Self {
        Follower{ common_state: CommonState::new(node_id, persisted_state) }
    }
}

impl<TTypes: Types> CommonState<TTypes> {
    fn new(node_id: NodeId, persisted_state: Option<PersistentCommonState<TTypes>>) -> Self {
        CommonState {
            common_persistent: persisted_state.unwrap_or(PersistentCommonState::new(node_id)),
            common_volatile: VolatileCommonState::default(),
        }
    }
}