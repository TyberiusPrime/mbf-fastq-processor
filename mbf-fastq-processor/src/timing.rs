use std::time::Duration;

/// Get thread CPU time (actual CPU time consumed, excluding I/O wait)
#[cfg(unix)]
fn get_thread_cpu_time() -> Duration {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    unsafe {
        libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut ts);
    }
    Duration::from_secs(ts.tv_sec as u64) + Duration::from_nanos(ts.tv_nsec as u64)
}

#[cfg(not(unix))]
fn get_thread_cpu_time() -> Duration {
    // Fallback for non-Unix systems - return zero
    // Windows would use GetThreadTimes(), but for now we'll just return zero
    Duration::ZERO
}

/// A single timing measurement for a step processing a block
#[derive(Debug, Clone)]
pub struct StepTiming {
    pub step_no: usize,
    pub step_type: String,
    pub wall_duration: Duration,
    pub cpu_duration: Duration,
}

impl StepTiming {
    /// Start timing a step - returns start times for both wall and CPU
    pub fn start() -> (std::time::Instant, Duration) {
        let wall_start = std::time::Instant::now();
        let cpu_start = get_thread_cpu_time();
        (wall_start, cpu_start)
    }

    /// Create a timing record from start times
    pub fn from_start(
        step_no: usize,
        step_type: String,
        wall_start: std::time::Instant,
        cpu_start: Duration,
    ) -> Self {
        let wall_duration = wall_start.elapsed();
        let cpu_end = get_thread_cpu_time();
        let cpu_duration = cpu_end.saturating_sub(cpu_start);

        StepTiming {
            step_no,
            step_type,
            wall_duration,
            cpu_duration,
        }
    }
}

/// Aggregated statistics for a step (JSON-serializable)
#[derive(Debug, serde::Serialize)]
pub struct StepTimingStats {
    pub step_no: usize,
    pub step_type: String,
    #[serde(serialize_with = "serialize_duration_ms")]
    pub wall_cumulative: Duration,
    #[serde(serialize_with = "serialize_duration_ms")]
    pub wall_avg: Duration,
    #[serde(serialize_with = "serialize_duration_ms")]
    pub wall_stddev: Duration,
    #[serde(serialize_with = "serialize_duration_ms")]
    pub wall_median: Duration,
    #[serde(serialize_with = "serialize_duration_ms")]
    pub cpu_cumulative: Duration,
    #[serde(serialize_with = "serialize_duration_ms")]
    pub cpu_avg: Duration,
    #[serde(serialize_with = "serialize_duration_ms")]
    pub cpu_stddev: Duration,
    #[serde(serialize_with = "serialize_duration_ms")]
    pub cpu_median: Duration,
    pub count: usize,
}

/// Serialize Duration as milliseconds (f64)
fn serialize_duration_ms<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_f64(duration.as_secs_f64() * 1000.0)
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

        // Wall time statistics
        let wall_cumulative: Duration = step_timings.iter().map(|t| t.wall_duration).sum();
        let wall_avg = wall_cumulative / (count as u32);
        let wall_mean_nanos = wall_avg.as_nanos() as f64;
        let wall_variance: f64 = step_timings
            .iter()
            .map(|t| {
                let diff = t.wall_duration.as_nanos() as f64 - wall_mean_nanos;
                diff * diff
            })
            .sum::<f64>()
            / (count as f64);
        let wall_stddev = Duration::from_nanos(wall_variance.sqrt() as u64);
        let mut wall_durations: Vec<Duration> =
            step_timings.iter().map(|t| t.wall_duration).collect();
        wall_durations.sort();
        let wall_median = if count % 2 == 0 {
            let mid = count / 2;
            (wall_durations[mid - 1] + wall_durations[mid]) / 2
        } else {
            wall_durations[count / 2]
        };

        // CPU time statistics
        let cpu_cumulative: Duration = step_timings.iter().map(|t| t.cpu_duration).sum();
        let cpu_avg = cpu_cumulative / (count as u32);
        let cpu_mean_nanos = cpu_avg.as_nanos() as f64;
        let cpu_variance: f64 = step_timings
            .iter()
            .map(|t| {
                let diff = t.cpu_duration.as_nanos() as f64 - cpu_mean_nanos;
                diff * diff
            })
            .sum::<f64>()
            / (count as f64);
        let cpu_stddev = Duration::from_nanos(cpu_variance.sqrt() as u64);
        let mut cpu_durations: Vec<Duration> =
            step_timings.iter().map(|t| t.cpu_duration).collect();
        cpu_durations.sort();
        let cpu_median = if count % 2 == 0 {
            let mid = count / 2;
            (cpu_durations[mid - 1] + cpu_durations[mid]) / 2
        } else {
            cpu_durations[count / 2]
        };

        stats.push(StepTimingStats {
            step_no,
            step_type,
            wall_cumulative,
            wall_avg,
            wall_stddev,
            wall_median,
            cpu_cumulative,
            cpu_avg,
            cpu_stddev,
            cpu_median,
            count,
        });
    }

    stats
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
                wall_duration: Duration::from_millis(100),
                cpu_duration: Duration::from_millis(90),
            },
            StepTiming {
                step_no: 0,
                step_type: "Head".to_string(),
                wall_duration: Duration::from_millis(200),
                cpu_duration: Duration::from_millis(180),
            },
            StepTiming {
                step_no: 0,
                step_type: "Head".to_string(),
                wall_duration: Duration::from_millis(300),
                cpu_duration: Duration::from_millis(270),
            },
        ];

        let stats = aggregate_timings(timings);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].step_no, 0);
        assert_eq!(stats[0].step_type, "Head");
        assert_eq!(stats[0].count, 3);
        assert_eq!(stats[0].wall_cumulative, Duration::from_millis(600));
        assert_eq!(stats[0].wall_avg, Duration::from_millis(200));
        assert_eq!(stats[0].wall_median, Duration::from_millis(200));
        assert_eq!(stats[0].cpu_cumulative, Duration::from_millis(540));
        assert_eq!(stats[0].cpu_avg, Duration::from_millis(180));
    }

    #[test]
    fn test_aggregate_multiple_steps() {
        let timings = vec![
            StepTiming {
                step_no: 0,
                step_type: "Head".to_string(),
                wall_duration: Duration::from_millis(100),
                cpu_duration: Duration::from_millis(95),
            },
            StepTiming {
                step_no: 1,
                step_type: "Filter".to_string(),
                wall_duration: Duration::from_millis(50),
                cpu_duration: Duration::from_millis(30),
            },
            StepTiming {
                step_no: 0,
                step_type: "Head".to_string(),
                wall_duration: Duration::from_millis(150),
                cpu_duration: Duration::from_millis(140),
            },
            StepTiming {
                step_no: 1,
                step_type: "Filter".to_string(),
                wall_duration: Duration::from_millis(75),
                cpu_duration: Duration::from_millis(50),
            },
        ];

        let stats = aggregate_timings(timings);
        assert_eq!(stats.len(), 2);

        // Step 0
        assert_eq!(stats[0].step_no, 0);
        assert_eq!(stats[0].count, 2);
        assert_eq!(stats[0].wall_cumulative, Duration::from_millis(250));
        assert_eq!(stats[0].cpu_cumulative, Duration::from_millis(235));

        // Step 1 (has more I/O wait)
        assert_eq!(stats[1].step_no, 1);
        assert_eq!(stats[1].count, 2);
        assert_eq!(stats[1].wall_cumulative, Duration::from_millis(125));
        assert_eq!(stats[1].cpu_cumulative, Duration::from_millis(80));
    }
}
