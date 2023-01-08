use crate::message::{OutputMessage, RaftRpcResp, RequestVoteReq};
use crate::state::{RaftLog, RaftTerm, State, Types};

pub fn process_msg<TTypes: Types>(mut state: State<TTypes>, mut message: RequestVoteReq)
                                  -> (State<TTypes>, OutputMessage<TTypes>) {
    let curr_term;

    // checking term
    /*
        Any node that gets a message RequestVoteReq either ignores it due to its term or
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
        if curr_term > &message.term {
            return (state, OutputMessage::RaftResp(reply_false(curr_term)))
        }

        // if term in message is greater than this node's one
        if curr_term <= &message.term {
            state = state.into_follower();
            state.common_mut().common_persistent.current_term = message.term;
            state.common_mut().common_persistent.voted_for = None;
        }
    }

    // check if log of sender is up-to-date
    match is_log_up_to_date(&state, &message) {
        true => (state, OutputMessage::RaftResp(reply_true(curr_term))),
        false => (state, OutputMessage::RaftResp(reply_false(curr_term)))
    }


}


fn reply_false(curr_term: RaftTerm) -> RaftRpcResp {
    RaftRpcResp::RequestVote { term: curr_term, vote_granted: false}
}

fn reply_true(curr_term: RaftTerm) -> RaftRpcResp {
    RaftRpcResp::RequestVote { term: curr_term, vote_granted: true}
}

/// checks whether the msg's sender's log is up to date from this node (receiver's) perspective
fn is_log_up_to_date<TTypes: Types>(state: &State<TTypes>, req: &RequestVoteReq) -> bool {

    // compare terms
    let msg_log_term = req.last_log_term;
    let this_node_log_term = state.common().common_persistent.last_msg_term;

    if this_node_log_term > msg_log_term {
        return false;
    }

    // compare indexes
    let this_node_log_idx = state.common().common_persistent.log.len();
    let msg_log_idx = req.last_log_index;

    if this_node_log_idx > msg_log_idx {
        return false;
    }


    true
}
