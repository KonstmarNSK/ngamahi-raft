use crate::message::{OutputMessage, RaftRpcResp, RequestVoteReq, ReqVoteResp};
use crate::state::{Follower, NodeId, RaftLog, RaftTerm, State, Types};

pub fn process_msg<TTypes: Types>(mut state: State<TTypes>, mut message: RequestVoteReq)
                                  -> (State<TTypes>, Vec<OutputMessage<TTypes>>) {
    use std::cmp::Ordering::*;

    let node_id = state.common().common_persistent.this_node_id;
    let curr_term = state.common().common_persistent.current_term;

    let follower = match state {
        State::Leader(leader) => {
            match curr_term.cmp(&message.term) {
                Greater | Equal => return (
                    State::Leader(leader),
                    vec![OutputMessage::RaftResp(reply_false(curr_term, node_id))]
                ),

                Less => state.into_follower()
            }
        }

        State::Candidate(candidate) => {
            match curr_term.cmp(&message.term) {
                Greater | Equal => return (
                    State::Candidate(candidate),
                    vec![OutputMessage::RaftResp(reply_false(curr_term, node_id))]
                ),

                Less => state.into_follower()
            }
        }

        State::Follower(follower) => {
            match curr_term.cmp(&message.term) {
                Greater | Equal => return (
                    State::Follower(follower),
                    vec![OutputMessage::RaftResp(reply_false(curr_term, node_id))]
                ),

                Less => follower
            }
        }
    };


    // check if log of sender is up-to-date
    match is_log_up_to_date(&follower, &message) {
        true => (State::Follower(follower), vec![OutputMessage::RaftResp(reply_true(curr_term, node_id))]),
        false => (State::Follower(follower), vec![OutputMessage::RaftResp(reply_false(curr_term, node_id))])
    }
}


fn reply_false(curr_term: RaftTerm, sender: NodeId) -> RaftRpcResp {
    RaftRpcResp::RequestVote(ReqVoteResp { term: curr_term, vote_granted: false, sender_id: sender })
}

fn reply_true(curr_term: RaftTerm, sender: NodeId) -> RaftRpcResp {
    RaftRpcResp::RequestVote(ReqVoteResp { term: curr_term, vote_granted: true, sender_id: sender })
}

/// checks whether the msg's sender's log is up to date from this node (receiver's) perspective
fn is_log_up_to_date<TTypes: Types>(state: &Follower<TTypes>, req: &RequestVoteReq) -> bool {

    // compare terms
    let msg_log_term = req.last_log_term;
    let this_node_log_term = state.common_state.common_persistent.last_msg_term;

    if this_node_log_term > msg_log_term {
        return false;
    }

    // compare indexes
    let this_node_log_idx = state.common_state.common_persistent.log.len();
    let msg_log_idx = req.last_log_index;

    if this_node_log_idx > msg_log_idx {
        return false;
    }


    true
}
