use crate::message::{AppendEntriesReq, OutputMessage, RaftRpcResp};
use crate::state::{NodeId, RaftTerm, State, Types};
use core::cmp::Ordering::*;
use crate::state::State::*;
use crate::state::{Leader, Follower, Candidate};

pub fn process_msg<TTypes: Types>(mut state: State<TTypes>, mut message: AppendEntriesReq<TTypes>)
                                  -> (State<TTypes>, OutputMessage<TTypes>) {

    let node_id = state.common().common_persistent.this_node_id;

    let curr_term;

    // checking term
    /*
        Any node that gets a message AppendEntriesReq either ignores it due to its term or
        will process it as a follower (if the node wasn't a follower, it becomes a follower.)
     */
    {
        let state_common = state.common();
        curr_term = state_common.common_persistent.current_term;

        /*
            todo: check special case: 2 or more leaders have same term.
                  It would mean that there is a mistake in the algorithm or its implementation
                  => this implementation of raft doesn't guarantee anything.
        */

        // if term in message is less than this node's one
        if &curr_term > &message.term {
            return (state, OutputMessage::RaftResp(reply_false(curr_term, node_id)))
        }

        // if term in message is greater than this node's one
        if &curr_term <= &message.term {
            state = state.into_follower();
            state.common_mut().common_persistent.current_term = message.term;
            state.common_mut().common_persistent.voted_for = None;
        }
    }

    // Reply false if log doesn't contain an entry at prevLogIndex
    // whose term matches prevLogTerm
    match state.common_mut().common_persistent.log.get(message.prev_log_idx) {
         Some(log_entry) if log_entry.term == message.prev_log_term => (),
         _ => return (state, OutputMessage::RaftResp(reply_false(curr_term, node_id)))
    }

    /*
        If an existing entry conflicts with a new one (same index
        but different terms), delete the existing entry and all that
        follow it.
        Append any new entries not already in the log
     */

    state.common_mut().common_persistent.log.truncate(message.prev_log_idx + 1);
    state.common_mut().common_persistent.log.append(&mut message.entries_to_append);

    if message.leader_commit_idx > state.common().common_volatile.committed_idx {
        state.common_mut().common_volatile.committed_idx = message.leader_commit_idx;
    }

    return (state, OutputMessage::RaftResp(reply_true(curr_term, node_id)))
}


fn reply_false(curr_term: RaftTerm, sender: NodeId) -> RaftRpcResp {
    RaftRpcResp::AppendEntries { term: curr_term, success: false, sender_id: sender}
}

fn reply_true(curr_term: RaftTerm, sender: NodeId) -> RaftRpcResp {
    RaftRpcResp::AppendEntries { term: curr_term, success: true, sender_id: sender}
}
