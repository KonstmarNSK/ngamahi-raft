use crate::basic::common_types::{LogMsgIdx, Term};

#[derive(Clone)]
pub struct LogEntry{
    term: Term,
    idx: LogMsgIdx,
    s: String,
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

    // todo: check
    /// entries_to_append MUST BE SORTED BY INDEX!!!
    pub fn force_append(&mut self, entries_to_append: &[LogEntry]) {

        if let Some(LogEntry { idx: LogMsgIdx(i), .. }) = entries_to_append.first() {
            self.entries.truncate(i.to_owned() as usize);
        }

        for entry in entries_to_append {
            self.entries.push(entry.to_owned())
        }

        if let Some(entry) = entries_to_append.last() {
            self.last_idx = entry.idx;
            self.last_term = entry.term
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