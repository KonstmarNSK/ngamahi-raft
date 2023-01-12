use crate::message::{AppendEntriesReq, AppendEntriesResp, ClientMessageReq, ClientMessageResp, InputMessage, OutputMessage, RaftRpcReq, RaftRpcResp, ReqVoteResp};
use crate::state::{Candidate, Leader, LogEntry, NodeId, RaftTerm, State, StateMachine, Types};

mod process_append_entries;
mod process_request_vote;
mod process_timer_events;


pub fn process_msg<TTypes: Types>(state: State<TTypes>, message: InputMessage<TTypes>)
                                  -> (State<TTypes>, Vec<OutputMessage<TTypes>>) {

    let (mut state, msg) = match message {
        InputMessage::RaftRequest(req) => {
            match req {
                RaftRpcReq::AppendEntries { addressee, req } => {
                    process_append_entries::process_msg(state, req)
                }

                RaftRpcReq::ReqVote(req_vote) => {
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

        InputMessage::ClientMsg(client_req) => {
            match state {
                State::Leader(leader) => process_client_request(leader, client_req),

                // todo: code duplication
                State::Follower(follower) => {
                    let leader = follower.common_state.common_volatile.known_leader;
                    let req_uid = client_req.unique_request_id;

                    (State::Follower(follower), vec![OutputMessage::ClientMsg(client_resp_false::<TTypes>(req_uid, leader))])
                }

                State::Candidate(candidate) => {
                    let leader = candidate.common_state.common_volatile.known_leader;
                    let req_uid = client_req.unique_request_id;

                    (State::Candidate(candidate), vec![OutputMessage::ClientMsg(client_resp_false::<TTypes>(req_uid, leader))])
                }
            }
        }

        InputMessage::TimerMsg(msg) =>
            process_timer_events::process_msg(state, msg),
    };

    // if lastApplied < lastCommitted
    state.apply_commands();

    return (state, msg);
}


fn client_resp_false<TTypes: Types>(req_uid: u128, leader: Option<NodeId>) -> ClientMessageResp {
    ClientMessageResp {
        success: false,
        leader_address: leader,
        unique_request_id: req_uid,
    }
}


fn process_client_request<TTypes: Types>(mut state: Leader<TTypes>, mut req: ClientMessageReq<TTypes>)
    -> (State<TTypes>, Vec<OutputMessage<TTypes>>) {

    // add entries to local (leader's) log:
    state.common_state.common_persistent.log.append(
        &mut req.entries_to_add.iter()
            .map(|&entry|
                LogEntry{term: state.common_state.common_persistent.current_term, cmd: entry}
            )
            .collect()
    );

    // send all other nodes an AppendEntries msg with those entries
    let append_entries = process_timer_events::process_heartbeat_trigger(&state);

    (State::Leader(state), append_entries)
}

// fixme: committed_idx update
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

            /*
                If there exists an N such that N > commitIndex, a majority
                of matchIndex[i] ≥ N, and log[N].term == currentTerm:
                set commitIndex = N (§5.3, §5.4).
             */

            let committed_idx = state.common_state.common_volatile.committed_idx;
            let curr_term = state.common_state.common_persistent.current_term;

            match state.common_state.common_persistent.log.last() {
                Some(entry) if entry.term == curr_term => {
                    // find the biggest matchIndex that most of nodes have:
                    let mut match_indexes = state.match_idx.iter()
                        .map(|(&id, &idx)| idx)
                        .collect::<Vec<usize>>();


                    /*
                        sort match_indexes and take a half + 1 values from bigger side,
                        then take the least value of them - that will be the biggest match_idx that
                        most of nodes have
                     */
                    match_indexes.sort();
                    let biggest_match_idx =  match_indexes.iter().rev()
                        .take((match_indexes.len()/2) + 1)
                        .min();

                    if let Some(&match_idx) = biggest_match_idx {
                        if committed_idx < match_idx {
                            state.common_state.common_volatile.committed_idx = match_idx;
                        }
                    }
                },

                _ => (),
            };
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