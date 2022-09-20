use std::net::IpAddr;
use crossbeam::channel;
use timer::TimerOutputMsg;
use crate::timer;
use crate::state;


pub struct TimerChannels {
    timer_input: channel::Sender<timer::TimerInputMsg>,
    timer_output: channel::Receiver<TimerOutputMsg>,
}

pub struct ThisNodeCmdChannels {
    input: channel::Receiver<ThisNodeInputCmd>,
    output: channel::Sender<ThisNodeOutputCmd>,
}

// used for interacting with other threads, NOT with other raft nodes
pub enum ThisNodeInputCmd {
    Stop,
}

// used for interacting with other threads, NOT with other raft nodes
pub enum ThisNodeOutputCmd {}


pub struct ThisNode {
    state: state::NodeRoleState,

    timer_channels: TimerChannels,
    this_node_channels: ThisNodeCmdChannels,
}


pub enum NodeErr {}

// todo: rename
pub trait NodesConnection {
    fn send_message(&self, others: &state::OtherNodes, message: NodesMessages) -> ();
    fn poll_message(&self) -> Option<NodesMessages>;
}

enum NodesMessages {
    Heartbeat,
    AppendEntries,
    RequestVote { sender_addr: state::NodeAddress },
}

impl ThisNode {
    fn new(other_nodes: state::OtherNodes, timer_channels: TimerChannels, this_node_channels: ThisNodeCmdChannels) -> Self {
        ThisNode {
            state: state::NodeRoleState::FollowerState(state::FollowerState {
                node_state: state::NodeState {
                    term: 0u64,
                    last_committed_log_idx: 0u64,
                    other_nodes,
                }
            }),

            timer_channels,
            this_node_channels,
        }
    }


    fn run(mut self) -> Result<(), NodeErr> {
        let mut cmd_iter = self.this_node_channels.input.try_iter();
        let mut timer_msg_iter = self.timer_channels.timer_output.try_iter();

        loop {

            // check for commands from other thread
            for cmd in &mut cmd_iter {
                return match cmd {
                    // stopping node
                    ThisNodeInputCmd::Stop => Ok(())
                };
            }

            // process commands from timer
            for timer_msg in &mut timer_msg_iter {
                self.state = process_timer_msg(timer_msg, self.state)?
            }
        }
    }
}

fn process_timer_msg(timer_msg: TimerOutputMsg, mut node_state: state::NodeRoleState) -> Result<state::NodeRoleState, NodeErr> {
    match timer_msg {
        TimerOutputMsg::HeartbeatTimeTrigger => {
            send_heartbeat_msg();
            Ok(node_state)
        }

        TimerOutputMsg::ElectionTimeTrigger => {
            let new_state = match node_state {
                state::NodeRoleState::CandidateState(_) => { node_state }
                state::NodeRoleState::LeaderState(_) => { node_state }
                state::NodeRoleState::FollowerState(s) => {
                    let new_state = to_candidate_state(s)?;
                    send_start_election_msg();

                    state::NodeRoleState::CandidateState(new_state)
                }
            };

            Ok(new_state)
        }
    }
}

fn to_candidate_state(state: state::FollowerState) -> Result<state::CandidateState, NodeErr> {
    let state = state.node_state;

    Ok(state::CandidateState {
        node_state: state::NodeState {
            term: state.term + 1,
            ..state
        }
    })
}


fn send_heartbeat_msg() -> () {
    unimplemented!()
}

fn send_start_election_msg() -> () {
    unimplemented!()
}

