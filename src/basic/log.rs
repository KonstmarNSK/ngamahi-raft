use std::ops::Deref;
use crate::basic::common_types::{LogMsgIdx, Term};
use crate::basic::log::entries_to_append::EntriesToAppend;
use crate::basic::log::not_empty_slice::NotEmptySlice;

#[derive(Clone)]
pub struct LogEntry{
    term: Term,
    idx: LogMsgIdx,
    s: String,
}

pub mod entries_to_append{
    use std::cmp::max;
    use std::ops::Deref;
    use crate::basic::common_types::{LogMsgIdx, Term};
    use crate::basic::log::LogEntry;
    use crate::basic::log::not_empty_slice::NotEmptySlice;


    /// Ensures that its entries are not empty and sorted in ascending order
    pub struct EntriesToAppend<'a> {
        entries: NotEmptySlice<'a, LogEntry>,
        prev_term: Term,
        prev_idx: LogMsgIdx,
    }

    impl <'a> EntriesToAppend<'a> {
        // todo: use an enum instead of String
        pub fn new(entries: NotEmptySlice<'a, LogEntry>, prev_term: Term, prev_idx: LogMsgIdx)
            -> Result<Self, String>
        {
            return match Self::check_sorted(&entries) {
                false => Err("Log entries are out-of-order!".to_owned()),  // must NEVER happen
                true => Ok(Self{entries, prev_term, prev_idx})
            };
        }

        // true -> sorted, ascending
        fn check_sorted(entries: &NotEmptySlice<'a, LogEntry>) -> bool {
            let mut last_seen_idx = LogMsgIdx(0);

            for &LogEntry{idx, ..} in entries.deref() {
                if last_seen_idx > idx {
                    return false
                }

                last_seen_idx = idx;
            }

            return true;
        }

        pub fn prev_term(&self) -> Term { self.prev_term }
        pub fn prev_idx(&self) -> LogMsgIdx { self.prev_idx }
        pub fn entries(&'a self) -> &'a NotEmptySlice<'a, LogEntry> { &self.entries }
    }
}


pub mod not_empty_slice {
    use std::ops::Deref;

    pub struct NotEmptySlice<'a, TElement> {
        slice: &'a [TElement]
    }

    impl <'a, TElement> NotEmptySlice<'a, TElement> {
        /// Some if slice is not empty, None otherwise
        pub fn new(slice: &'a [TElement]) -> Option<Self> {
            match slice.is_empty() {
                true => None,
                false => Some(NotEmptySlice { slice })
            }
        }

        pub fn first(&self) -> &TElement {
            // never panics because at creation time we ensure that slice is not empty
            self.slice.first().unwrap()
        }

        pub fn last(&self) -> &TElement {
            self.slice.last().unwrap()
        }

    }

    impl <'a, TElement> Deref for NotEmptySlice<'a, TElement>{
        type Target = [TElement];

        fn deref(&self) -> &Self::Target {
            self.slice
        }
    }
}


pub struct RaftLog {
    entries: Vec<LogEntry>,

    last_idx: LogMsgIdx,
    last_term: Term,

    last_committed_idx: LogMsgIdx,
    last_committed_term: Term,
}

impl RaftLog {

    // idx and term of the last message in the log
    pub fn get_last_msg_info(&self) -> (LogMsgIdx, Term) {
        (self.last_idx, self.last_term)
    }

    // idx and term of the last COMMITTED msg in the log
    pub fn get_last_committed_info(&self) -> (LogMsgIdx, Term) {
        (self.last_committed_idx, self.last_committed_term)
    }

    pub fn get_entry_term_by_idx(&self, idx: LogMsgIdx) -> Option<(Term)> {
        self.entries.iter().rev()
            .find(|e| e.idx == idx)
            .map(|entry| entry.term)
    }

    pub fn force_append(&mut self, mut entries_to_append: EntriesToAppend) {

        truncate_log(self, &entries_to_append);
        self.entries.extend_from_slice(entries_to_append.entries());

        let last_index


        // AppendEntries message contains prev_term and prev_idx. Those are term and index of a
        // message that immediately precedes those that must be appended to log.
        // If this node contains messages after that one, those messages will be deleted from the log.
        fn truncate_log(log: &mut RaftLog, entries_to_append: &EntriesToAppend) {
            let prev_idx = entries_to_append.prev_idx();
            let prev_term = entries_to_append.prev_term();

            let previous_entry = log.entries.iter().enumerate().rev()
                .find(|(num, entry)| entry.idx == prev_idx && entry.term == prev_term)
                .map(|(num, _)| num)
                // todo: handle properly
                .expect(&format!("No entry in log with idx {:?} and term {:?}", prev_idx, prev_term));

            log.entries.truncate(previous_entry + 1)
        }
    }
}


impl Default for RaftLog {
    fn default() -> Self {
        RaftLog {
            entries: vec![],
            last_idx: LogMsgIdx::default(),
            last_term: Term::default(),
            last_committed_idx: LogMsgIdx::default(),
            last_committed_term: Term::default(),
        }
    }
}