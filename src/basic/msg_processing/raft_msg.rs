use crate::basic::messages::{OutputMessage, RaftMessage};
use crate::basic::state_common::NodeState;


/// Process message that IS specified in Raft paper
pub fn process_msg(
    msg: RaftMessage,
    mut node_state: NodeState,
) -> (NodeState, Option<OutputMessage>) {
    unimplemented!()
}