use std::cmp::Ordering;
use crate::basic::common_types::{LogMsgIdx, Term};
use crate::basic::messages::{AppendEntriesMsg, AppendEntriesResp, get_term, OutputMessage, RaftMessage};
use crate::basic::state::{CandidateState, Common, FollowerState, LeaderState, NodeState};
use crate::basic::log::{entries_to_append, not_empty_slice};


/// Process message that IS specified in Raft paper
pub fn process_msg(
    msg: RaftMessage,
    mut node_state: NodeState,
) -> (NodeState, Option<OutputMessage>) {
    return match msg {
        RaftMessage::AppendEntries( m) => todo!(),
        RaftMessage::AppendEntriesResp( m ) => todo!(),
        RaftMessage::RequestVote ( m ) => todo!()
    };
}


impl FollowerState {

    fn process_append_entries_msg(&mut self, msg: &AppendEntriesMsg) -> RaftMessage {

        self.common.term = match check_term(self, msg) {
            Err(resp) => return RaftMessage::AppendEntriesResp(resp),
            Ok(term) => term
        };

        return RaftMessage::AppendEntriesResp(match is_log_consistent(self, msg) {
            true => append_entries(self, msg),
            false => AppendEntriesResp {
                sender_term: self.common.term,
                success: false,
            }
        });




        /// Leader sent its term in the AppendEntries message
        /// Leader's term must be at least as big as follower's
        ///
        /// returns either new term if check succeeded or message to leader with negative response
        fn check_term(state: &FollowerState, msg: &AppendEntriesMsg) -> Result<Term, AppendEntriesResp> {
            use Ordering::*;

            match msg.sender_term.cmp(&state.common.term) {
                Less => Err(AppendEntriesResp {
                    sender_term: state.common.term,
                    success: false,
                }),

                Equal | Greater => Ok(msg.sender_term)
            }
        }

        // If this node's log contains an entry with same term and index as those of an entry in
        // leader's log right before those new that the leader wants to append (prev_log_term and prev_log_index),
        // the log consistency check is passed and failed otherwise.
        fn is_log_consistent(state: &FollowerState, msg: &AppendEntriesMsg) -> bool {
            match state.common.log.get_entry_term_by_idx(msg.prev_log_index) {
                Some(term) if term == msg.prev_log_term => true,
                _ => false
            }
        }

        fn append_entries(state: &mut FollowerState, msg: &AppendEntriesMsg) -> AppendEntriesResp {

            if let Some(entries) = not_empty_slice::NotEmptySlice::new(&msg.content) {
                let entries = entries_to_append::EntriesToAppend::new(
                    entries, msg.prev_log_term, msg.prev_log_index
                ).unwrap(); // todo: process result properly

                state.common.log.force_append(entries);
            }

            AppendEntriesResp {
                success: true,
                sender_term: state.common.term
            }
        }

        /*
        3. If an existing entry conflicts with a new one (same index
        but different terms), delete the existing entry and all that
        follow it (§5.3)
        4. Append any new entries not already in the log
        5. If leaderCommit > commitIndex, set commitIndex =
        min(leaderCommit, index of last new entry)
         */
    }

}