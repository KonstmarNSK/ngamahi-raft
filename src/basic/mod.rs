use std::net::IpAddr;
use std::time::Instant;

use crossbeam::channel;

use messaging::node_internal::*;
use timer::TimerOutputMsg;
use crate::basic::messaging::between_nodes::NodesConnection;
use crate::basic::state_common::NodeState;

use crate::timer;

mod leader;
mod follower;
mod candidate;
mod state_common;
mod messaging;


pub struct ThisNode<NC: NodesConnection> {
    pub state: NodeState,                // current state of this node

    pub timer_channels: TimerChannels,              // channels used to interact with timer
    pub this_node_channels: ThisNodeCmdChannels,    // channels used to interact this node thread with other treads (within this process)
    pub nodes_connection: NC,                       // thing that is used to interact with other Raft nodes
}


pub enum NodeErr {}


impl <NC: NodesConnection> ThisNode<NC> {
    fn new(
        other_nodes: state_common::OtherNodes,
        this_node_address: state_common::NodeAddress,
        election_timeout: std::time::Duration,
        timer_channels: TimerChannels,
        this_node_channels: ThisNodeCmdChannels,
        nodes_connection: NC,
    ) -> Self {
        ThisNode {
            state: state_common::NodeState::init(other_nodes, this_node_address, election_timeout),

            timer_channels,
            this_node_channels,
            nodes_connection,
        }
    }


    fn run(mut self) -> Result<(), NodeErr> {
        let mut internal_messages = self.this_node_channels.input.try_iter();
        let mut timer_messages = self.timer_channels.timer_output.try_iter();

        loop {

            // check for commands from other thread
            for cmd in &mut internal_messages {
                return match cmd {
                    // stopping node
                    ThisNodeInputCmd::Stop => Ok(())
                };
            }

            // process commands from timer
            for timer_msg in &mut timer_messages {
                self.state = process_timer_msg(timer_msg, self.state, &self.nodes_connection)?
            }
        }
    }
}


fn process_timer_msg<NC: NodesConnection>(
    timer_msg: TimerOutputMsg,
    mut node_state: NodeState,
    nodes_connection: &NC,
) -> Result<NodeState, NodeErr> {
    return match timer_msg {
        TimerOutputMsg::HeartbeatTimeTrigger => {
            process_heartbeat_timer_msg(&node_state);
            Ok(node_state)
        }

        TimerOutputMsg::ElectionTimeTrigger => {
            let new_state = process_election_time_trigger(node_state, nodes_connection);
            Ok(new_state)
        }
    };


    fn process_heartbeat_timer_msg(node_state: &NodeState) -> () {
        // only leader sends heartbeat messages
        match node_state {
            NodeState::LeaderState(_) => send_heartbeat_msg(),
            _ => ()
        }

    }

    fn process_election_time_trigger<NC: NodesConnection>(mut node_state: NodeState, nodes_connection: &NC) -> NodeState {
        match node_state {

            // if a follower doesn't get any messages within election interval, it becomes a candidate
            NodeState::FollowerState(s) => {
                if can_start_election(&s) {
                    NodeState::CandidateState(start_election(s, nodes_connection))
                } else {
                    // return same state if some messages from other nodes were got recently
                    NodeState::FollowerState(s)
                }
            }

            // if a candidate doesn't win an election within election interval, it starts new election
            NodeState::CandidateState(ref mut s) => {
                unimplemented!()
            }

            // leader never starts new elections
            NodeState::LeaderState(_) => node_state
        }
    }
}

fn can_start_election(node_state: &follower::FollowerState) -> bool {
    node_state.node_state.last_time_received_message.elapsed() > node_state.node_state.election_timeout
}

///     Starts election.
///
///     When a Raft node starts election, it becomes a candidate, votes for itself,
///     increments its term and sends 'RequestVote' message
///
fn start_election<NC: NodesConnection>(mut node_state: follower::FollowerState, nodes_connection: &NC) -> candidate::CandidateState {
    let mut new_node_state = to_candidate_state(node_state);
    new_node_state.node_state.election_favorite_this_term = Some(new_node_state.node_state.this_node_address);
    new_node_state.node_state.term = new_node_state.node_state.term + 1;

    nodes_connection.send_message(
        &new_node_state.node_state.other_nodes,
        messaging::between_nodes::NodesMessages::RequestVote { sender_addr: new_node_state.node_state.this_node_address },
    );

    new_node_state
}

fn to_candidate_state(state: follower::FollowerState) -> candidate::CandidateState {
    let state = state.node_state;

    candidate::CandidateState {
        node_state: state_common::Common {
            ..state
        }
    }
}


fn send_heartbeat_msg() -> () {
    unimplemented!()
}

fn send_start_election_msg() -> () {
    unimplemented!()
}
