use std::fmt::{Debug, Formatter};
use crate::message::{InputMessage, OutputMessage};
use crate::state::{RaftNode, Types};

mod state;
mod msg_process;
mod message;


impl<TTypes: Types> RaftNode<TTypes> {
    pub fn process_message(&mut self, message: InputMessage<TTypes>) -> Vec<OutputMessage<TTypes>> {
        let state = self.state.take().expect("State MUST ALWAYS be initialized");
        let (new_state, out_msg) = msg_process::process_msg(state, message);

        self.state = Some(new_state);

        out_msg
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::net::{IpAddr, Ipv4Addr};
    use crate::message::{InputMessage, OutputMessage, RaftRpcReq, TimerMessage};
    use crate::state::{Command, InitParams, LogEntry, NodeId, PersistentCommonState, RaftNode, RaftTerm, State, StateMachine, Types};

    struct TestTypes;

    struct TestStateMachine;

    impl Command for u8 {}

    impl Types for TestTypes {
        type TCmd = u8;
        type TStateMachine = TestStateMachine;
    }

    impl StateMachine<TestTypes> for TestStateMachine {
        fn apply_commands(&mut self, commands: &[LogEntry<<TestTypes as Types>::TCmd>]) {
            println!("applied: {:?}", commands.iter().map(|&cmd| cmd.cmd).collect::<Vec<u8>>())
        }
    }


    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    fn test_nodes() -> (RaftNode<TestTypes>, RaftNode<TestTypes>, RaftNode<TestTypes>) {
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        let node1_addr = NodeId { address: localhost, cluster_id: 0 };
        let node2_addr = node1_addr.clone();
        let node3_addr = node1_addr.clone();


        let node1 = RaftNode::new(params(node3_addr, vec![node2_addr, node3_addr]));
        let node2 = RaftNode::new(params(node3_addr, vec![node1_addr, node3_addr]));
        let node3 = RaftNode::new(params(node3_addr, vec![node2_addr, node1_addr]));


        return (node1, node2, node3);

        fn params(node: NodeId, mut others: Vec<NodeId>) -> InitParams<TestTypes> {
            others.push(node);

            let state = PersistentCommonState {
                this_node_id: node,
                current_term: RaftTerm(0),
                voted_for: None,
                log: vec![],
                last_msg_term: RaftTerm(0),
                state_machine: TestStateMachine,
                cluster_nodes: HashSet::from_iter(others),
            };

            InitParams {
                persisted_state: state,
                node_id: node,
            }
        }
    }

    #[test]
    fn base_log_replication() {
        use InputMessage as In;

        let (mut node1, mut node2, mut node3) = test_nodes();
        let trigger_election_msg = In::TimerMsg(TimerMessage::TriggerElections);

        let out_msg = node1.process_message(trigger_election_msg);

        assert_eq!(out_msg.len(), 1);

        let msg = out_msg.first().unwrap();

        assert!( match msg {
            OutputMessage::RaftReq(RaftRpcReq::ReqVote(_)) => true,
            _ => false
        });
    }
}
