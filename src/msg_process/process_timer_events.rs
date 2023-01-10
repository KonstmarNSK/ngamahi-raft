use crate::message::{AppendEntriesReq, OutputMessage, RaftRpcReq, RequestVoteReq, TimerMessage};
use crate::message::OutputMessage::RaftResp;
use crate::message::RaftRpcResp::RequestVote;
use crate::state::{Candidate, Follower, Leader, LogEntry, NodeId, State, Types};

pub fn process_msg<TTypes: Types>(mut state: State<TTypes>, message: TimerMessage)
                                  -> (State<TTypes>, Vec<OutputMessage<TTypes>>) {
    match message {
        TimerMessage::TriggerElections => {
            match state {
                State::Follower(follower) => process_election_trigger(FollowerOrCandidate::Follower(follower)),
                State::Candidate(candidate) => process_election_trigger(FollowerOrCandidate::Candidate(candidate)),
                State::Leader(leader) => (State::Leader(leader), vec![]),
            }
        },

        TimerMessage::TriggerHeartbeat => {
            match state {
                State::Leader(leader) => {
                    let out_msg =  process_heartbeat_trigger(&leader);
                    (State::Leader(leader), out_msg)
                },

                state => (state, vec![]),
            }
        }
    }
}


fn process_heartbeat_trigger<TTypes: Types>(state: &Leader<TTypes>) -> Vec<OutputMessage<TTypes>> {
    use crate::msg_process::heartbeat_msg;

    let last_log_idx = state.common_state.common_persistent.log.len();

    // looking for nodes whose log indexes aren't up-to-date
    let nodes: Vec<OutputMessage<TTypes>> = state.next_idx.iter()
        .map(|(&id, &idx)| {
            /*
                if leader thinks that this node's next index is not up-to-date,
                it sends an appendEntries msg with entries and sends an empty appendEntries otherwise
                for heartbeat
             */
            if idx <= last_log_idx {
                // send log entries
                let entries_to_send = &state.common_state.common_persistent.log[idx..last_log_idx];
                let entries_to_send = entries_to_send.to_vec();

                OutputMessage::RaftReq(RaftRpcReq::AppendEntries {
                    addressee: id,
                    req: AppendEntriesReq {
                        leader_id: state.common_state.common_persistent.this_node_id,
                        term: state.common_state.common_persistent.current_term,
                        leader_commit_idx: state.common_state.common_volatile.committed_idx,
                        prev_log_idx: state.common_state.common_persistent.log.len(),
                        prev_log_term: state.common_state.common_persistent.last_msg_term,
                        entries_to_append: entries_to_send,
                    },
                })
            } else {
                OutputMessage::RaftReq(RaftRpcReq::AppendEntries {
                    addressee: id,
                    req: heartbeat_msg(state),
                })
            }
        })
        .collect();


    nodes
}


enum FollowerOrCandidate<TTypes: Types> {
    Follower(Follower<TTypes>),
    Candidate(Candidate<TTypes>),
}

fn process_election_trigger<TTypes: Types>(state: FollowerOrCandidate<TTypes>) -> (State<TTypes>, Vec<OutputMessage<TTypes>>) {
    use FollowerOrCandidate::*;

    match state {
        Follower(follower) => follower_elect(follower),
        Candidate(candidate) => start_election(candidate),
    }
}

// todo: add pre-vote step
fn follower_elect<TTypes: Types>(state: Follower<TTypes>) -> (State<TTypes>, Vec<OutputMessage<TTypes>>) {
    // convert to a candidate
    let mut state = state.into_candidate();

    // start election
    start_election(state)
}

fn start_election<TTypes: Types>(mut state: Candidate<TTypes>) -> (State<TTypes>, Vec<OutputMessage<TTypes>>) {
    use crate::message::RaftRpcReq::ReqVote;

    /*
        Increment currentTerm
    • Vote for self
    • Reset election timer
    • Send RequestVote RPCs to all other servers
     */

    let this_node_id = state.common_state.common_persistent.this_node_id;
    let new_term = state.common_state.common_persistent.current_term + 1;

    state.common_state.common_persistent.voted_for = Some(this_node_id);
    state.common_state.common_persistent.current_term = new_term;

    let last_log_index = state.common_state.common_persistent.log.len();
    let last_log_term = state.common_state.common_persistent.last_msg_term;

    let req_vote_msg = ReqVote(RequestVoteReq {
        term: new_term,
        candidate_id: this_node_id,
        last_log_index,
        last_log_term,
    });

    return (State::Candidate(state), vec![OutputMessage::RaftReq(req_vote_msg)]);
}