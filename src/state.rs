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
    type TStateMachine: StateMachine<Self>;
}

pub type RaftLog<TTypes: Types> = Vec<LogEntry<TTypes::TCmd>>;

pub struct PersistentCommonState<TTypes: Types> {
    pub this_node_id: NodeId,
    pub current_term: RaftTerm,
    pub voted_for: Option<NodeId>,
    pub log: RaftLog<TTypes>,

    pub last_msg_term: RaftTerm, // raft paper doesn't contain this. Here we remember term of last msg in log
    pub state_machine: TTypes::TStateMachine,
}

impl <TTypes: Types> PersistentCommonState<TTypes> {
    pub fn apply_commands(&mut self, start: usize, end: usize) {
        self.state_machine.apply_commands(&self.log[start..end])
    }
}

impl<TTypes: Types> PersistentCommonState<TTypes> {
    fn new(node_id: NodeId, state_machine: TTypes::TStateMachine) -> Self {
        PersistentCommonState {
            this_node_id: node_id,
            current_term: RaftTerm(0),
            voted_for: None,
            log: vec![],

            last_msg_term: RaftTerm(0),
            state_machine,
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

// underlying state machine (implemented in user code)
pub trait StateMachine <TTypes: Types>{
    fn apply_commands(&mut self, commands: &[TTypes::TCmd]);
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
    pub persisted_state: PersistentCommonState<TTypes>,
    // this node's id
    pub node_id: NodeId,
}


pub struct RaftNode<TTypes: Types> {
    pub state: State<TTypes>,
}

impl<TTypes: Types> RaftNode<TTypes> {
    pub fn new(node_id: NodeId, mut params: InitParams<TTypes>) -> Self {
        RaftNode{
            state: State::new(node_id, params.persisted_state)
        }
    }
}

impl<TTypes: Types> State<TTypes> {
    fn new(node_id: NodeId, persisted_state: PersistentCommonState<TTypes>) -> Self {
        State {
            cluster_role: State::Follower(Follower::new(persisted_state))
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

    pub fn apply_commands(&mut self) {
        let start = self.common().common_volatile.last_applied;
        let end = self.common().common_volatile.committed_idx;

        if start < end {
            self.common_mut().common_persistent.apply_commands(start, end);
        }

        self.common_mut().common_volatile.last_applied = end;
    }
}

impl<TTypes: Types> Follower<TTypes> {
    fn new(persisted_state: PersistentCommonState<TTypes>) -> Self {
        Follower{ common_state: CommonState::new(persisted_state) }
    }
}

impl<TTypes: Types> CommonState<TTypes> {
    fn new(persisted_state: PersistentCommonState<TTypes>) -> Self {
        CommonState {
            common_persistent: persisted_state,
            common_volatile: VolatileCommonState::default(),
        }
    }
}