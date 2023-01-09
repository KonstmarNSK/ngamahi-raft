use crate::message::{InputMessage, OutputMessage, RaftRpcReq};
use crate::state::{RaftTerm, State, StateMachine, Types};

mod process_append_entries;
mod process_request_vote;
mod process_timer_events;

pub fn process_msg<TTypes: Types>(state: State<TTypes>, message: InputMessage<TTypes>)
                                  -> (State<TTypes>, OutputMessage<TTypes>) {

    let (mut state, msg) = match message {
        InputMessage::RaftMessage(RaftRpcReq::AppendEntries(req)) => {
            let state = check_term(state, &req.term);
            process_append_entries::process_msg(state, req)
        },

        InputMessage::RaftMessage(RaftRpcReq::ReqVote(req)) => {
            let state = check_term(state, &req.term);
            process_request_vote::process_msg(state, req)
        },

        InputMessage::TimerMsg(msg) =>
            process_timer_events::process_msg(state, msg),
    };

    // if lastApplied < lastCommitted
    state.apply_commands();

    (state, msg)
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