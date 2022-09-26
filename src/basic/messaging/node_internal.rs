use crossbeam::channel;
use crate::timer::{TimerInputMsg, TimerOutputMsg};

// used for interacting with other threads, NOT with other raft nodes


// messages from/to the timer
pub struct TimerChannels {
    pub timer_input: channel::Sender<TimerInputMsg>,
    pub timer_output: channel::Receiver<TimerOutputMsg>,
}


// generic messages that the node accepts and produces
pub struct ThisNodeCmdChannels {
    pub input: channel::Receiver<ThisNodeInputCmd>,
    pub output: channel::Sender<ThisNodeOutputCmd>,
}


pub enum ThisNodeInputCmd {
    Stop,
}

pub enum ThisNodeOutputCmd {}