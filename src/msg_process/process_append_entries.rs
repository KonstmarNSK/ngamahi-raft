use crate::message::{AppendEntriesReq, AppendEntriesResp, OutputMessage, RaftRpcResp};
use crate::state::{NodeId, RaftTerm, State, Types};
use core::cmp::Ordering::*;
use std::cmp::Ordering;
use std::fmt::format;
use crate::state::State::*;
use crate::state::{Leader, Follower, Candidate};

pub fn process_msg<TTypes: Types>(mut state: State<TTypes>, mut message: AppendEntriesReq<TTypes>)
                                  -> (State<TTypes>, Vec<OutputMessage<TTypes>>) {
    let node_id = state.common().common_persistent.this_node_id;

    let curr_term;

    // checking term
    /*
        Any node that gets a message AppendEntriesReq either ignores it due to its term or
        will process it as a follower (if the node wasn't a follower, it becomes a follower.)
     */
    let state_common = state.common();
    curr_term = state_common.common_persistent.current_term;

    let mut follower = match state {
        Leader(mut leader) => {
            match curr_term.cmp(&message.term) {
                Greater => {
                    return (
                        State::Leader(leader),
                        vec![OutputMessage::RaftResp(reply_false(curr_term, 0, node_id))]
                    );
                }

                Less => {
                    leader.common_state.common_persistent.current_term = message.term;
                    leader.common_state.common_persistent.voted_for = None;

                    Follower::from(leader)
                },

                Equal => panic!("{}", format!(
                    "Two leaders in one term!!! Term: {:?}, leaders: {:?} and {:?}. Probably this raft algorithm implementation is broken.",
                    curr_term,
                    node_id,
                    message.leader_id
                ))
            }
        }

        /*
         AppendEntries rpc can be sent only by a leader.
         If a candidate gets such message with term same as its own, it immediately becomes
         a follower because there's an existing leader.
        */
        Candidate(mut candidate) => {
            match curr_term.cmp(&message.term) {
                Less | Equal => {
                    candidate.common_state.common_persistent.current_term = message.term;
                    candidate.common_state.common_persistent.voted_for = None;

                    Follower::from(candidate)
                },

                Greater => return (
                    State::Candidate(candidate),
                    vec![OutputMessage::RaftResp(reply_false(curr_term, 0, node_id))]
                )
            }
        }

        Follower(mut follower) => {
            match curr_term.cmp(&message.term) {
                Less => {
                    follower.common_state.common_persistent.current_term = message.term;
                    follower.common_state.common_persistent.voted_for = None;
                }

                Greater => return (
                    State::Follower(follower),
                    vec![OutputMessage::RaftResp(reply_false(curr_term, 0, node_id))]
                ),

                Equal => ()
            };



            follower
        }
    };

    // recently (now) got AppendEntries rpc from viable leader, so don't start election next time timer triggers it
    follower.trigger_election_next_time = false;

    follower.common_state.common_volatile.known_leader = Some(message.leader_id);


    // Reply false if log doesn't contain an entry at prevLogIndex
    // whose term matches prevLogTerm
    match follower.common_state.common_persistent.log.get(message.prev_log_idx) {
        Some(log_entry) if log_entry.term == message.prev_log_term => (),
        _ => return (
            State::Follower(follower),
            vec![OutputMessage::RaftResp(reply_false(curr_term, 0, node_id))]
        )
    }

    /*
        If an existing entry conflicts with a new one (same index
        but different terms), delete the existing entry and all that
        follow it.
        Append any new entries not already in the log
     */

    follower.common_state.common_persistent.log.truncate(message.prev_log_idx + 1);
    follower.common_state.common_persistent.log.append(&mut message.entries_to_append);

    if message.leader_commit_idx > follower.common_state.common_volatile.committed_idx {
        follower.common_state.common_volatile.committed_idx = message.leader_commit_idx;
    }

    // remember term of last message in log
    if let Some(entry) = follower.common_state.common_persistent.log.last() {
        follower.common_state.common_persistent.last_msg_term = entry.term;
    };

    return (
        State::Follower(follower),
        vec![OutputMessage::RaftResp(reply_true(curr_term, message.entries_to_append.len(), node_id))]
    );
}


fn reply_false(curr_term: RaftTerm, appended: usize, sender: NodeId) -> RaftRpcResp {
    RaftRpcResp::AppendEntries(AppendEntriesResp { term: curr_term, appended_entries_count: appended, success: false, sender_id: sender })
}

fn reply_true(curr_term: RaftTerm, appended: usize, sender: NodeId) -> RaftRpcResp {
    RaftRpcResp::AppendEntries(AppendEntriesResp { term: curr_term, appended_entries_count: appended, success: true, sender_id: sender })
}
