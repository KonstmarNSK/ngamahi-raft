use std::ops::Not;
use crate::basic::messages::{ControlMessage, OutputMessage, RaftMessage};
use crate::basic::state::{CandidateState, Common, Either, FollowerState, LeaderState, NodeState};


/// Process message that isn't specified in Raft paper
pub fn process_msg(
    msg: ControlMessage,
    mut node_state: NodeState,
) -> (NodeState, Option<OutputMessage>) {
    return match msg {
        ControlMessage::TriggerHeartbeat => {
            let msg = process_heartbeat_trigger(&node_state);
            (node_state, msg)
        }

        ControlMessage::TriggerElection => process_election_trigger(node_state)
    };
}


fn process_heartbeat_trigger(node_state: &NodeState) -> Option<OutputMessage> {
    return match node_state {
        NodeState::LeaderState(ref leader_state) =>
            Some(OutputMessage::RaftMsg { message: RaftMessage::AppendEntries{sender_term: leader_state.node_state.term} }),

        _ => None
    };
}


fn process_election_trigger(mut node_state: NodeState) -> (NodeState, Option<OutputMessage>) {
    return match node_state {

        NodeState::FollowerState(follower_state) => {

            let (mut follower_or_candidate, msg) =
                follower_state.process_election_trigger();

            let out_msg = msg.map(|m| OutputMessage::RaftMsg {
                message: m
            });

            match follower_or_candidate {

                // still a follower, didn't start election
                Either::Left(f_state) => {
                    let state = NodeState::FollowerState(f_state);
                    (state, out_msg)
                }

                // became a candidate and started election
                Either::Right(mut c_state) => {
                    let new_state = NodeState::CandidateState(c_state);
                    (new_state, out_msg)
                }
            }
        }


        NodeState::CandidateState(mut candidate_state) => {
            let msg = candidate_state.process_election_trigger();
            let out_msg = Some(OutputMessage::RaftMsg { message: msg });
            let new_state = NodeState::CandidateState(candidate_state);

            (new_state, out_msg)
        }

        // leader doesn't react on election timeout trigger
        NodeState::LeaderState(_) => {
            (node_state, None)
        }
    };

}






// ========== IMPLS ==========



// FOLLOWER

impl FollowerState {
    // todo: pre-vote stage
    /// Starts election and becomes a candidate unless it got a valid message from leader recently
    fn process_election_trigger(self) -> (Either<FollowerState, CandidateState>, Option<RaftMessage>) {
        return match &self.ignore_next_election_timeout_trigger {
            true => (Either::Left(self), None),
            false => {
                let (new_state, msg) = elect(self);
                return (Either::Right(new_state), Some(msg));
            }
        };

        fn elect(follower_state: FollowerState) -> (CandidateState, RaftMessage) {

            // transfer to candidate state
            let mut candidate_state: CandidateState = follower_state.into();
            let msg = start_election(&mut candidate_state);

            (candidate_state, msg)
        }
    }
}




// CANDIDATE

/// transfer to candidate state
impl From<FollowerState> for CandidateState {
    fn from(follower_state: FollowerState) -> Self {
        CandidateState {
            ignore_next_election_timeout_trigger: follower_state.ignore_next_election_timeout_trigger,
            leader_address: follower_state.leader_address,
            votes_count: 0,
            node_state: follower_state.node_state
        }
    }
}

impl CandidateState {
    /// If a candidate didn't get enough votes (N/2 + 1), it starts a new election.
    ///
    /// A candidate can get an ElectionTimeoutTrigger only if it didn't get enough messages within
    /// an election timeout (otherwise it would be a leader or follower already),
    /// so we just start a new  election without any checks
    fn process_election_trigger(&mut self) -> RaftMessage {
        start_election(self)
    }
}

fn start_election(candidate_state: &mut CandidateState) -> RaftMessage {

    // increment term:
    candidate_state.node_state.term += 1;

    // vote for self:
    candidate_state.votes_count = 1;

    // and send RequestVote message to other nodes
    RaftMessage::RequestVote {
        sender_term: candidate_state.node_state.term,
        sender_addr: candidate_state.node_state.this_node_address
    }
}





// LEADER