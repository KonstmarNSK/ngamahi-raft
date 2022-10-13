use std::ops::AddAssign;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct LogMsgIdx(pub u64);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Term(pub u64);


impl Term {
    pub fn increment(&mut self) {
        self.0 = self.0 + 1
    }
}