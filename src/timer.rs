pub trait Timer {
    fn start(args: InitialArgs) -> Self;
}

pub struct InitialArgs {
    election_timeout: std::time::Duration,
    heartbeat_timeout: std::time::Duration,

    input_channel: crossbeam::channel::Receiver<TimerInputMsg>,
    output_channel: crossbeam::channel::Sender<TimerOutputMsg>,
}

pub enum TimerInputMsg {
    NewElectionTimeout(std::time::Duration),
    NewHeartbeatTimeout(std::time::Duration),
    Stop,
}

pub enum TimerOutputMsg {
    ElectionTimeTrigger,
    HeartbeatTimeTrigger,
}