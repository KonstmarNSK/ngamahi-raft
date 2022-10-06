use crate::basic::messages::*;
use crate::basic::msg_processing::process_msg;
use crate::basic::state::*;

mod state;
mod messages;
mod msg_processing;


/// The Raft node (state machine)
///
/// Reacts to messages, changes its state, sends messages.
///
pub struct ThisNode<MQ: MessageQueue> {
    pub state: NodeState,       // current state of this node
    pub msg_queue: MQ,          // interaction with outer world
}



impl<MQ: MessageQueue> ThisNode<MQ> {

    /// Creates a new node
    fn new(
        other_nodes: OtherNodes,
        this_node_address: NodeAddress,
        msg_queue: MQ,
    ) -> Self {

        ThisNode {
            state: NodeState::init(other_nodes, this_node_address),
            msg_queue,
        }
    }

    /// Runs this node, blocks caller thread.
    /// Node can be stopped by passing a Stop message (ControlMessage::Stop)
    fn run(mut self) -> () {
        loop {
            if let Some(msg) = self.msg_queue.read_message() {
                self.state = send_msg_and_get_new_state(
                    process_msg(msg, self.state),
                    &self.msg_queue,
                )
            }
        }

        fn send_msg_and_get_new_state<MQ: MessageQueue>(
            (node_state, new_msg): (NodeState, Option<OutputMessage>),
            mq: &MQ,
        ) -> NodeState {
            if let Some(msg) = new_msg {
                mq.push_message(msg);
            }

            node_state
        }
    }
}
