use crate::message::{AppendEntriesReq, OutputMessage, RaftRpcReq, RequestVoteReq, TimerMessage};
use crate::message::OutputMessage::RaftResp;
use crate::message::RaftRpcResp::RequestVote;
use crate::state::{Candidate, Follower, Leader, NodeId, State, Types};

pub fn process_msg<TTypes: Types>(mut state: State<TTypes>, message: TimerMessage)
                                  -> (State<TTypes>, Vec<OutputMessage<TTypes>>) {
    todo!()
}


fn process_heartbeat_trigger<TTypes: Types>(state: &Leader<TTypes>) -> OutputMessage<TTypes> {
    use crate::msg_process::heartbeat_msg;

    let last_log_idx = state.common_state.common_persistent.log.len();

    // looking for nodes whose log indexes aren't up-to-date
    let nodes: Vec<(NodeId, usize)> = state.next_idx.iter()
        .filter_map(|(&id, &idx)| {
            if idx <= last_log_idx {
                Some((id, idx))
            } else {
                None
            }
        } )
        .collect();

    todo!()

    // nodes
    //
    // OutputMessage::RaftReq(RaftRpcReq::AppendEntries(heartbeat_msg(state)))
}


enum FollowerOrCandidate<TTypes: Types> {
    Follower(Follower<TTypes>),
    Candidate(Candidate<TTypes>),
}

fn process_election_trigger<TTypes: Types>(state: FollowerOrCandidate<TTypes>) -> (State<TTypes>, OutputMessage<TTypes>) {
    use FollowerOrCandidate::*;

    match state {
        Follower(follower) => follower_elect(follower),
        Candidate(candidate) => start_election(candidate),
    }
}

// todo: add pre-vote step
fn follower_elect<TTypes: Types>(state: Follower<TTypes>) -> (State<TTypes>, OutputMessage<TTypes>) {
    // convert to a candidate
    let mut state = state.into_candidate();

    // start election
    start_election(state)
}

fn start_election<TTypes: Types>(mut state: Candidate<TTypes>) -> (State<TTypes>, OutputMessage<TTypes>) {
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

    return (State::Candidate(state), OutputMessage::RaftReq(req_vote_msg));
}