use crate::basic::messages::{OutputMessage, RaftMessage};
use crate::basic::state::NodeState;


/// Process message that IS specified in Raft paper
pub fn process_msg(
    msg: RaftMessage,
    mut node_state: NodeState,
) -> (NodeState, Option<OutputMessage>) {
    match msg {
        RaftMessage::Heartbeat => (process_heartbeat_msg(node_state), None),

        RaftMessage::AppendEntries => unimplemented!(),

        RaftMessage::RequestVote{ sender_addr } => unimplemented!()
    }
}


fn process_heartbeat_msg(mut node_state: NodeState) -> NodeState {
    match node_state {

        // follower remembers that it got a heartbeat message
        NodeState::FollowerState(ref mut follower_state) => {
            follower_state.node_state.ignore_next_election_timeout_trigger = true;

            node_state
        }

        NodeState::CandidateState(ref mut candidate_state) => {
            candidate_state.node_state.ignore_next_election_timeout_trigger = true;

            node_state
        }

        // leader ignores heartbeats
        NodeState::LeaderState(_) => node_state
    }
}