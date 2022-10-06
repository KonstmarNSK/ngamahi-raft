use std::cmp::Ordering;
use crate::basic::messages::{OutputMessage, RaftMessage};
use crate::basic::state::{CandidateState, Common, FollowerState, LeaderState, NodeState};


/// Process message that IS specified in Raft paper
pub fn process_msg(
    msg: RaftMessage,
    mut node_state: NodeState,
) -> (NodeState, Option<OutputMessage>) {
    return match msg {
        RaftMessage::AppendEntries { sender_term } => todo!(),

        RaftMessage::RequestVote { sender_term, sender_addr } => todo!()
    };
}


/// Compare message sender's term with this node's one
fn check_term(msg: &RaftMessage, mut node_state: NodeState) -> (NodeState, Option<OutputMessage>) {
    return match node_state {

        // if leader gets a message with higher term than its own, it immediately becomes a follower
        NodeState::LeaderState(leader_state) => {
            match leader_state.node_state.term < get_term(msg) {
                true => (leader_to_follower(leader_state), None),
                false => (NodeState::LeaderState(leader_state), None)
            }
        }

        // if a candidate gets a message with term that is equal to or greater than its own,
        // it immediately becomes a follower
        NodeState::CandidateState(candidate_state) => {
            match candidate_state.node_state.term <= get_term(msg) {
                true => (candidate_to_follower(candidate_state), None),
                false => (NodeState::CandidateState(candidate_state), None)
            }
        }

        NodeState::FollowerState(_) => (node_state, None)
    };


    fn leader_to_follower(leader_state: LeaderState) -> NodeState {
        NodeState::FollowerState(
            FollowerState {
                node_state: leader_state.node_state,

                leader_address: None,
                election_favorite_this_term: None,
                ignore_next_election_timeout_trigger: false,
            }
        )
    }

    fn candidate_to_follower(candidate_state: CandidateState) -> NodeState {
        NodeState::FollowerState(
            FollowerState {
                node_state: candidate_state.node_state,

                leader_address: None,
                election_favorite_this_term: None,
                ignore_next_election_timeout_trigger: false,
            }
        )
    }
}


fn get_term(msg: &RaftMessage) -> u64 {
    return match msg {
        RaftMessage::RequestVote { sender_term, .. }
        | RaftMessage::AppendEntries { sender_term } => sender_term.to_owned()
    };
}

// todo: check terms and committed msg idx
fn process_heartbeat_msg(mut node_state: NodeState) -> NodeState {
    match node_state {

        // follower remembers that it got a heartbeat message
        NodeState::FollowerState(ref mut follower_state) => {
            follower_state.ignore_next_election_timeout_trigger = true;

            node_state
        }

        NodeState::CandidateState(ref mut candidate_state) => {
            candidate_state.ignore_next_election_timeout_trigger = true;

            node_state
        }

        // leader ignores heartbeats
        NodeState::LeaderState(_) => node_state
    }
}