use crate::message::{AppendEntriesReq, AppendEntriesResp, InputMessage, OutputMessage, RaftRpcReq, RaftRpcResp, ReqVoteResp};
use crate::state::{Candidate, Leader, NodeId, RaftTerm, State, StateMachine, Types};

mod process_append_entries;
mod process_request_vote;
mod process_timer_events;


pub fn process_msg<TTypes: Types>(state: State<TTypes>, message: InputMessage<TTypes>)
                                  -> (State<TTypes>, Vec<OutputMessage<TTypes>>) {
    let (mut state, msg) = match message {
        InputMessage::RaftRequest(req) => {
            match req {
                RaftRpcReq::AppendEntries { addressee, req } => {
                    let state = check_term(state, &req.term);
                    process_append_entries::process_msg(state, req)
                }

                RaftRpcReq::ReqVote(req_vote) => {
                    let state = check_term(state, &req_vote.term);
                    process_request_vote::process_msg(state, req_vote)
                }
            }
        }

        InputMessage::RaftResponse(resp) => {
            match resp {
                RaftRpcResp::AppendEntries(append) => {
                    match check_term(state, &append.term) {
                        State::Leader(state) =>
                            process_append_entries_response(
                                state,
                                append,
                            ),

                        state => (state, vec![])
                    }
                }

                RaftRpcResp::RequestVote(req_vote) => {
                    match check_term(state, &req_vote.term) {
                        State::Candidate(state) =>
                            (process_request_vote_response(
                                state,
                                req_vote,
                            ), vec![]),

                        state => (state, vec![])
                    }
                }
            }
        }

        InputMessage::TimerMsg(msg) =>
            process_timer_events::process_msg(state, msg),
    };

    // if lastApplied < lastCommitted
    state.apply_commands();

    (state, msg)
}


fn process_append_entries_response<TTypes: Types>(
    mut state: Leader<TTypes>,
    resp: AppendEntriesResp,
)
    -> (State<TTypes>, Vec<OutputMessage<TTypes>>) {
    match resp.success {
        true => {
            state.next_idx.entry(resp.sender_id)
                .and_modify(|&mut old_idx| { old_idx + resp.appended_entries_count; });

            state.match_idx.entry(resp.sender_id)
                .and_modify(|&mut old_idx| { old_idx + resp.appended_entries_count; });
        }
        false => {
            state.next_idx.entry(resp.sender_id)
                .and_modify(|&mut old_idx| { old_idx - 1; });
        }
    };

    (State::Leader(state), vec![])
}


fn process_request_vote_response<TTypes: Types>(
    mut state: Candidate<TTypes>,
    resp: ReqVoteResp,
)
    -> State<TTypes> {
    state.followers_voted.insert(resp.sender_id);

    // turn into leader if got enough votes
    if state.followers_voted.len() > state.common_state.common_persistent.cluster_nodes.len() / 2 {
        let leader = state.into();
        State::Leader(leader)
    } else {
        State::Candidate(state)
    }
}


/// turns state to a follower if this node's term is lower than one in message
fn check_term<TTypes: Types>(mut state: State<TTypes>, msg_term: &RaftTerm) -> State<TTypes> {
    let state_common = state.common();
    let curr_term = state_common.common_persistent.current_term;

    // if term in message is greater than this node's one
    if curr_term <= *msg_term {
        state = state.into_follower();
        state.common_mut().common_persistent.current_term = msg_term.clone();
        state.common_mut().common_persistent.voted_for = None;
    }

    return state;
}

pub fn heartbeat_msg<TTypes: Types>(state: &Leader<TTypes>) -> AppendEntriesReq<TTypes> {
    AppendEntriesReq {
        leader_id: state.common_state.common_persistent.this_node_id,
        term: state.common_state.common_persistent.current_term,
        leader_commit_idx: state.common_state.common_volatile.committed_idx,
        prev_log_idx: state.common_state.common_persistent.log.len(),
        prev_log_term: state.common_state.common_persistent.last_msg_term,
        entries_to_append: vec![],
    }
}