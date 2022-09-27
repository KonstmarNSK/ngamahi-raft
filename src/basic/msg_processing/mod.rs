use crate::basic::messages::{InputMessage, OutputMessage};
use crate::basic::state_common::NodeState;

mod control_msg;
mod raft_msg;


pub fn process_msg(
    msg: InputMessage,
    mut node_state: NodeState,
) -> (NodeState, Option<OutputMessage>) {

    return match msg {
        InputMessage::ControlMessage(ctrl_msg) => {
            control_msg::process_msg(
                ctrl_msg,
                node_state,
            )
        }

        InputMessage::RaftMsg {message} => {
            raft_msg::process_msg(
                message,
                node_state,
            )
        }
    };
}