use std::time::Duration;

/// A single timing measurement for a step processing a block
#[derive(Debug, Clone)]
pub struct StepTiming {
    pub step_no: usize,
    pub step_type: String,
    pub duration: Duration,
}

/// Aggregated statistics for a step
#[derive(Debug)]
pub struct StepTimingStats {
    pub step_no: usize,
    pub step_type: String,
    pub cumulative_runtime: Duration,
    pub avg: Duration,
    pub stddev: Duration,
    pub median: Duration,
    pub count: usize,
}

impl StepTimingStats {
    /// Formats the duration in a human-readable way (ms or s)
    fn format_duration(d: &Duration) -> String {
        let ms = d.as_secs_f64() * 1000.0;
        if ms < 1000.0 {
            format!("{:.2}ms", ms)
        } else {
            format!("{:.2}s", d.as_secs_f64())
        }
    }

    /// Returns a table header
    pub fn table_header() -> String {
        format!(
            "{:<8} {:<30} {:<15} {:<15} {:<15} {:<15} {:<10}",
            "Step", "Type", "Cumulative", "Avg", "StdDev", "Median", "Count"
        )
    }

    /// Formats this stat as a table row
    pub fn table_row(&self) -> String {
        format!(
            "{:<8} {:<30} {:<15} {:<15} {:<15} {:<15} {:<10}",
            self.step_no,
            self.step_type,
            Self::format_duration(&self.cumulative_runtime),
            Self::format_duration(&self.avg),
            Self::format_duration(&self.stddev),
            Self::format_duration(&self.median),
            self.count
        )
    }
}

/// Aggregates timing data into statistics per step
pub fn aggregate_timings(timings: Vec<StepTiming>) -> Vec<StepTimingStats> {
    if timings.is_empty() {
        return Vec::new();
    }

    // Group timings by step number
    let mut grouped: std::collections::BTreeMap<usize, Vec<StepTiming>> =
        std::collections::BTreeMap::new();

    for timing in timings {
        grouped.entry(timing.step_no).or_default().push(timing);
    }

    // Calculate statistics for each step
    let mut stats = Vec::new();
    for (step_no, step_timings) in grouped {
        if step_timings.is_empty() {
            continue;
        }

        let step_type = step_timings[0].step_type.clone();
        let count = step_timings.len();

        // Calculate cumulative runtime
        let cumulative_runtime: Duration = step_timings.iter().map(|t| t.duration).sum();

        // Calculate average
        let avg = cumulative_runtime / (count as u32);

        // Calculate standard deviation
        let mean_nanos = avg.as_nanos() as f64;
        let variance: f64 = step_timings
            .iter()
            .map(|t| {
                let diff = t.duration.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>()
            / (count as f64);
        let stddev = Duration::from_nanos(variance.sqrt() as u64);

        // Calculate median
        let mut durations: Vec<Duration> = step_timings.iter().map(|t| t.duration).collect();
        durations.sort();
        let median = if count % 2 == 0 {
            let mid = count / 2;
            (durations[mid - 1] + durations[mid]) / 2
        } else {
            durations[count / 2]
        };

        stats.push(StepTimingStats {
            step_no,
            step_type,
            cumulative_runtime,
            avg,
            stddev,
            median,
            count,
        });
    }

    stats
}

/// Formats timing statistics as a table
pub fn format_timing_table(stats: &[StepTimingStats]) -> String {
    if stats.is_empty() {
        return String::from("No timing data available");
    }

    let mut table = String::new();
    table.push_str(&StepTimingStats::table_header());
    table.push('\n');
    table.push_str(&"-".repeat(125));
    table.push('\n');

    for stat in stats {
        table.push_str(&stat.table_row());
        table.push('\n');
    }

    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_empty() {
        let timings = Vec::new();
        let stats = aggregate_timings(timings);
        assert!(stats.is_empty());
    }

    #[test]
    fn test_aggregate_single_step() {
        let timings = vec![
            StepTiming {
                step_no: 0,
                step_type: "Head".to_string(),
                duration: Duration::from_millis(100),
            },
            StepTiming {
                step_no: 0,
                step_type: "Head".to_string(),
                duration: Duration::from_millis(200),
            },
            StepTiming {
                step_no: 0,
                step_type: "Head".to_string(),
                duration: Duration::from_millis(300),
            },
        ];

        let stats = aggregate_timings(timings);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].step_no, 0);
        assert_eq!(stats[0].step_type, "Head");
        assert_eq!(stats[0].count, 3);
        assert_eq!(stats[0].cumulative_runtime, Duration::from_millis(600));
        assert_eq!(stats[0].avg, Duration::from_millis(200));
        assert_eq!(stats[0].median, Duration::from_millis(200));
    }

    #[test]
    fn test_aggregate_multiple_steps() {
        let timings = vec![
            StepTiming {
                step_no: 0,
                step_type: "Head".to_string(),
                duration: Duration::from_millis(100),
            },
            StepTiming {
                step_no: 1,
                step_type: "Filter".to_string(),
                duration: Duration::from_millis(50),
            },
            StepTiming {
                step_no: 0,
                step_type: "Head".to_string(),
                duration: Duration::from_millis(150),
            },
            StepTiming {
                step_no: 1,
                step_type: "Filter".to_string(),
                duration: Duration::from_millis(75),
            },
        ];

        let stats = aggregate_timings(timings);
        assert_eq!(stats.len(), 2);

        // Step 0
        assert_eq!(stats[0].step_no, 0);
        assert_eq!(stats[0].count, 2);
        assert_eq!(stats[0].cumulative_runtime, Duration::from_millis(250));

        // Step 1
        assert_eq!(stats[1].step_no, 1);
        assert_eq!(stats[1].count, 2);
        assert_eq!(stats[1].cumulative_runtime, Duration::from_millis(125));
    }

    #[test]
    fn test_format_timing_table() {
        let stats = vec![StepTimingStats {
            step_no: 0,
            step_type: "Head".to_string(),
            cumulative_runtime: Duration::from_millis(600),
            avg: Duration::from_millis(200),
            stddev: Duration::from_millis(50),
            median: Duration::from_millis(200),
            count: 3,
        }];

        let table = format_timing_table(&stats);
        assert!(table.contains("Step"));
        assert!(table.contains("Type"));
        assert!(table.contains("Cumulative"));
        assert!(table.contains("Head"));
    }
}
