//! 合成ルートが中核へ渡すポートの束（ARC-006）。

use alloc::boxed::Box;

use crate::clock_port::ClockPort;
use crate::journal_port::JournalPort;

/// ジャーナルと壁時計。増えたらここに足し、`DocketScreen::open` の引数は増やさない。
pub struct Ports {
    journal: Box<dyn JournalPort>,
    clock: Box<dyn ClockPort>,
}

impl Ports {
    /// 実装を束ねる。
    #[must_use]
    pub fn new(journal: Box<dyn JournalPort>, clock: Box<dyn ClockPort>) -> Self {
        Self { journal, clock }
    }

    /// ジャーナル（読むだけ）。
    #[must_use]
    pub fn journal(&self) -> &dyn JournalPort {
        self.journal.as_ref()
    }

    /// ジャーナル（追記・同期・読み直し）。
    pub fn journal_mut(&mut self) -> &mut dyn JournalPort {
        self.journal.as_mut()
    }

    /// 壁時計。
    #[must_use]
    pub fn clock(&self) -> &dyn ClockPort {
        self.clock.as_ref()
    }
}

impl core::fmt::Debug for Ports {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("Ports { journal, clock }")
    }
}
