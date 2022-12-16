use crate::message::{InputMessage, OutputMessage, RaftRpcReq};
use crate::state::{State, Types};

mod process_append_entries;

pub fn process_msg<TTypes: Types>(state: State<TTypes>, message: InputMessage<TTypes>)
                                  -> (State<TTypes>, OutputMessage<TTypes>) {
    match message {
        InputMessage::RaftMessage(RaftRpcReq::AppendEntries(append_entr_req)) => {
            process_append_entries::
        },

        _ => {}
    }
}