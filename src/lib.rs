extern crate core;

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
    use InputMessage as In;
    use OutputMessage as Out;
    use std::collections::HashSet;
    use std::net::{IpAddr, Ipv4Addr};
    use crate::message::{InputMessage, OutputMessage, RaftRpcReq, RaftRpcResp, RequestVoteReq, TimerMessage};
    use crate::state::{Command, InitParams, LogEntry, NodeId, PersistentCommonState, RaftNode, RaftTerm, State, StateMachine, Types};

    #[derive(Clone)]
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

    /// Whether a node sends a RequestVote message when timer triggers election
    #[test]
    fn election_triggers() {
        let (mut node1, mut node2, mut node3) = test_nodes();
        let trigger_election_msg = In::TimerMsg(TimerMessage::TriggerElections);

        let out_msg = node1.process_message(trigger_election_msg);

        assert_eq!(out_msg.len(), 1);

        let msg = out_msg.first().unwrap();

        match msg {
            OutputMessage::RaftReq(
                RaftRpcReq::ReqVote(
                    RequestVoteReq {
                        term,
                        candidate_id,
                        last_log_index,
                        last_log_term
                    }
                )) => {
                assert_eq!(*term, RaftTerm(1));
                assert_eq!(*candidate_id, node1.state.unwrap().common_mut().common_persistent.this_node_id);
                assert_eq!(*last_log_index, 0);
                assert_eq!(*last_log_term, RaftTerm(0))
            }

            _ => assert!(false),
        };
    }

    #[test]
    fn election_succeeds() {
        let (mut node1, mut node2, mut node3) = test_nodes();
        let trigger_election_msg = In::TimerMsg(TimerMessage::TriggerElections);
        let mut req_vote_msg = node1.process_message(trigger_election_msg);
        let req_vote_msg = req_vote_msg.remove(0);

        let req_vote_msg: In<TestTypes> = match req_vote_msg {
            Out::RaftReq(req) => In::RaftRequest(req),
            _ => {
                assert!(false);
                unreachable!()
            }
        };

        let mut node2_response = node2.process_message(req_vote_msg.clone());
        let mut node3_response = node3.process_message(req_vote_msg);

        assert_eq!(node2_response.len(), 1);
        assert_eq!(node3_response.len(), 1);

        let node2_response = node2_response.remove(0);
        let node3_response = node3_response.remove(0);

        // check followers' reqVote responses
        match (node2_response.clone(), node3_response.clone()) {
            (
                Out::RaftResp(RaftRpcResp::RequestVote(resp_2)),
                Out::RaftResp(RaftRpcResp::RequestVote(resp_3)),
            ) => {
                assert_eq!(resp_2.term, RaftTerm(1));
                assert_eq!(resp_3.term, RaftTerm(1));

                assert_eq!(resp_2.sender_id, node2.state.unwrap().common().common_persistent.this_node_id);
                assert_eq!(resp_3.sender_id, node3.state.unwrap().common().common_persistent.this_node_id);

                assert!(resp_2.vote_granted);
                assert!(resp_3.vote_granted);
            }

            _ => assert!(false),
        };

        // check if the candidate node is actually a candidate
        match &node1.state.as_ref().unwrap() {
            State::Candidate(candidate) => {
                assert_eq!(candidate.followers_voted.len(), 1)
            },
            _ => assert!(false),
        };


        // now followers' responses are sent back to the candidate. It must then become a leader:
        let (node3_response_in, node2_response_in) : (In<TestTypes>, In<TestTypes>) =
            match (node2_response, node3_response) {
                (Out::RaftResp(resp2), Out::RaftResp(resp3)) => {
                    (In::RaftResponse(resp2), In::RaftResponse(resp3))
                }

                _ => {
                    assert!(false);
                    unreachable!()
                }
            };

        assert_eq!(node1.process_message(node2_response_in).len(), 0);

        match &node1.state.as_ref().unwrap() {
            State::Leader(leader) => (),
            State::Candidate(candidate) => (),
            State::Follower(fol) => {
                println!("Follower!")
            },
        };
    }
}
