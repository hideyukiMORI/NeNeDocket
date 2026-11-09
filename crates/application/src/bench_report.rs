//! 計測の報告（`--bench`・Issue #111）。施主が端末から gate-proofs や Issue に貼れる、決まった形の数行。

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;

use crate::bench_phase::BenchPhase;
use crate::bench_seed::OPS_PER_ITEM;
use crate::frame_lap::FrameLap;

/// 台帳の作成と起動の時間と、フレームごとの区間。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchReport {
    items: u32,
    seed: Duration,
    first_open: Duration,
    checkpoint: Duration,
    second_open: Duration,
    laps: Vec<(BenchPhase, FrameLap)>,
}

impl BenchReport {
    /// 件数と（作成・初回の起動・スナップショットの書き・2 回目の起動）から。フレームはまだ無い。
    #[must_use]
    pub const fn new(items: u32, opening: (Duration, Duration, Duration, Duration)) -> Self {
        let (seed, first_open, checkpoint, second_open) = opening;
        Self {
            items,
            seed,
            first_open,
            checkpoint,
            second_open,
            laps: Vec::new(),
        }
    }

    /// UI が測ったフレームを添える。
    #[must_use]
    pub fn with_laps(self, laps: Vec<(BenchPhase, FrameLap)>) -> Self {
        Self { laps, ..self }
    }

    /// 報告の文。`platform` は `x11` / `windows`、`bounds` は窓の大きさの説明。
    #[must_use]
    pub fn text(&self, platform: &str, bounds: &str) -> String {
        let operations = u64::from(self.items).saturating_mul(u64::from(OPS_PER_ITEM));
        let head = format!(
            "bench ({platform}, window {bounds}): {} items x {OPS_PER_ITEM} ops = {operations} ops, 1 account\n\
             seed (append + 1 sync)        {}\n\
             first open (read all + merge) {}\n\
             snapshot write                {}\n\
             second open (from snapshot)   {}\n\
             frame (median / max)          step | encode | present\n",
            self.items,
            seconds(self.seed),
            seconds(self.first_open),
            seconds(self.checkpoint),
            seconds(self.second_open),
        );
        let frames: String = [(BenchPhase::Idle, "idle  "), (BenchPhase::Search, "search")]
            .into_iter()
            .map(|(phase, label)| self.frame_line(phase, label))
            .collect();
        head + &frames
    }

    /// 1 つの段の中央値と最大。
    fn frame_line(&self, phase: BenchPhase, label: &str) -> String {
        let laps: Vec<FrameLap> = self
            .laps
            .iter()
            .filter(|(measured, _)| *measured == phase)
            .map(|(_, lap)| *lap)
            .collect();
        format!(
            "  {label} x{:<3}                  {} | {} | {}\n",
            laps.len(),
            spread(laps.iter().map(FrameLap::step)),
            spread(laps.iter().map(FrameLap::encode)),
            spread(laps.iter().map(FrameLap::present)),
        )
    }
}

/// 秒（小数 3 桁）。
fn seconds(duration: Duration) -> String {
    format!("{:.3} s", duration.as_secs_f64())
}

/// 中央値と最大（ミリ秒・小数 2 桁）。無ければ `-`。
fn spread<I: Iterator<Item = Duration>>(durations: I) -> String {
    let mut sorted: Vec<Duration> = durations.collect();
    sorted.sort_unstable();
    match (sorted.get(sorted.len() / 2), sorted.last()) {
        (Some(median), Some(max)) => format!("{} / {}", millis(*median), millis(*max)),
        (None, _) | (_, None) => String::from("-"),
    }
}

/// ミリ秒（小数 2 桁）。
fn millis(duration: Duration) -> String {
    format!("{:.2} ms", duration.as_secs_f64() * 1000.0_f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    #[test]
    fn the_report_names_the_scale_and_splits_each_frame_into_three_spans() {
        let laps = Vec::from([
            (BenchPhase::Warmup, FrameLap::new(ms(90), ms(90), ms(90))),
            (BenchPhase::Idle, FrameLap::new(ms(1), ms(2), ms(16))),
            (BenchPhase::Idle, FrameLap::new(ms(3), ms(4), ms(17))),
            (BenchPhase::Idle, FrameLap::new(ms(2), ms(3), ms(18))),
            (BenchPhase::Search, FrameLap::new(ms(5), ms(6), ms(7))),
        ]);
        let text = BenchReport::new(10, (ms(1500), ms(250), ms(30), ms(40)))
            .with_laps(laps)
            .text("x11", "1120x600");
        assert!(
            text.starts_with(
                "bench (x11, window 1120x600): 10 items x 5 ops = 50 ops, 1 account\n"
            ),
            "{text}"
        );
        assert!(
            text.contains("seed (append + 1 sync)        1.500 s\n"),
            "{text}"
        );
        assert!(
            text.contains("second open (from snapshot)   0.040 s\n"),
            "{text}"
        );
        assert!(
            text.contains(
                "  idle   x3                    2.00 ms / 3.00 ms | 3.00 ms / 4.00 ms | 17.00 ms / 18.00 ms\n"
            ),
            "{text}"
        );
        assert!(
            text.contains(
                "  search x1                    5.00 ms / 5.00 ms | 6.00 ms / 6.00 ms | 7.00 ms / 7.00 ms\n"
            ),
            "{text}"
        );
        assert!(
            !text.contains("90.00"),
            "warm-up frames are not counted: {text}"
        );
    }

    #[test]
    fn a_phase_without_frames_shows_a_dash() {
        let text = BenchReport::new(1, (ms(0), ms(0), ms(0), ms(0))).text("windows", "800x600");
        assert!(
            text.contains("  idle   x0                    - | - | -\n"),
            "{text}"
        );
    }
}
