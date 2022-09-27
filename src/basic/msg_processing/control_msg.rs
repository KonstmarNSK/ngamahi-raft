use std::ops::Not;
use crate::basic::messages::{ControlMessage, OutputMessage, RaftMessage};
use crate::basic::state::{CandidateState, Common, FollowerState, NodeState};


/// Process message that isn't specified in Raft paper
pub fn process_msg(
    msg: ControlMessage,
    mut node_state: NodeState,
) -> (NodeState, Option<OutputMessage>) {
    return match msg {
        ControlMessage::TriggerHeartbeat => {
            let msg = process_heartbeat_msg(&node_state);
            (node_state, msg)
        }

        ControlMessage::TriggerElection => process_election_trigger(node_state)
    };
}


fn process_heartbeat_msg(node_state: &NodeState) -> Option<OutputMessage> {
    return match node_state {
        NodeState::LeaderState(_) =>
            Some(OutputMessage::RaftMsg { message: RaftMessage::Heartbeat }),

        _ => None
    };
}


fn process_election_trigger(mut node_state: NodeState) -> (NodeState, Option<OutputMessage>) {
    return match node_state {

        NodeState::FollowerState(follower_state) => {
            match follower_state.node_state.ignore_next_election_timeout_trigger {
                // if follower didn't get any message from leader or candidate
                // since election trigger was fired last time
                // it becomes a candidate and starts election:
                false => {
                    let (candidate_state, raft_msg) = follower_to_candidate(follower_state);
                    let new_state = NodeState::CandidateState(candidate_state);
                    let msg = OutputMessage::RaftMsg { message: raft_msg };

                    (new_state, Some(msg))
                }

                // if a follower got a message recently, it ignores election trigger:
                true => (NodeState::FollowerState(follower_state), None)
            }
        }

        NodeState::CandidateState(candidate_state) => {
            match candidate_state.node_state.ignore_next_election_timeout_trigger {

                // if a candidate didn't win election in election timeout, it starts new election:
                false => {
                    let (new_state, msg) = start_election(candidate_state.node_state);
                    let new_state = CandidateState { node_state: new_state };
                    let new_state = NodeState::CandidateState(new_state);
                    let msg = OutputMessage::RaftMsg { message: msg };

                    (new_state, Some(msg))
                }

                true => (NodeState::CandidateState(candidate_state), None)
            }
        }

        NodeState::LeaderState(_) => {
            // leader doesn't react on election timeout trigger
            (node_state, None)
        }
    };
}


fn follower_to_candidate(follower_state: FollowerState) -> (CandidateState, RaftMessage) {
    let (new_state, msg) = start_election(follower_state.node_state);

    (CandidateState { node_state: new_state }, msg)
}


fn start_election(mut state: Common) -> (Common, RaftMessage) {
    // increment term:
    state.term = state.term + 1;

    // and send RequestVote message to other nodes
    let out_msg = RaftMessage::RequestVote {
        sender_addr: state.this_node_address
    };

    (state, out_msg)
}