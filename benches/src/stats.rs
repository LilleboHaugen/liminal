//! Sample statistics (spec §6 "Statistics", BENCH-R020 … R024): median, median absolute
//! deviation, minimum, maximum and throughput.

/// Summary statistics of one case's samples, each sample being the time per iteration in
/// nanoseconds.
///
/// # Examples
///
/// ```
/// use liminal_benches::Summary;
///
/// let summary = Summary::from_samples(&[4.0, 1.0, 3.0, 2.0, 10.0]).expect("five samples");
/// assert_eq!(summary.samples, 5);
/// assert_eq!(summary.median_ns, 3.0);
/// assert_eq!(summary.mad_ns, 1.0); // deviations 1, 2, 0, 1, 7 -> median 1
/// assert_eq!((summary.min_ns, summary.max_ns), (1.0, 10.0));
/// assert_eq!(Summary::from_samples(&[]), None);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Summary {
    /// Number of samples.
    pub samples: u32,
    /// Median time per iteration, ns (BENCH-R020).
    pub median_ns: f64,
    /// Median absolute deviation from the median, ns, unscaled (BENCH-R021).
    pub mad_ns: f64,
    /// Fastest sample, ns per iteration (BENCH-R022).
    pub min_ns: f64,
    /// Slowest sample, ns per iteration (BENCH-R022).
    pub max_ns: f64,
}

impl Summary {
    /// Computes the summary of `samples` (ns per iteration). Returns `None` for an empty slice,
    /// or if a sample is NaN or infinite (BENCH-R024).
    #[must_use]
    pub fn from_samples(samples: &[f64]) -> Option<Summary> {
        if samples.is_empty() || samples.iter().any(|s| !s.is_finite()) {
            return None;
        }
        let count = u32::try_from(samples.len()).ok()?;
        let mut sorted = samples.to_vec();
        sorted.sort_by(f64::total_cmp);
        let median_ns = median_of_sorted(&sorted);
        let mut deviations: Vec<f64> = sorted.iter().map(|s| (s - median_ns).abs()).collect();
        deviations.sort_by(f64::total_cmp);
        Some(Summary {
            samples: count,
            median_ns,
            mad_ns: median_of_sorted(&deviations),
            min_ns: sorted[0],
            max_ns: sorted[sorted.len() - 1],
        })
    }

    /// Elements per second at the median time: `elements × 10⁹ / median_ns` (BENCH-R023).
    /// `None` if `elements` is `None` or the median is not positive.
    #[must_use]
    pub fn throughput_per_s(&self, elements: Option<u64>) -> Option<f64> {
        let elements = elements?;
        if self.median_ns > 0.0 {
            #[expect(
                clippy::cast_precision_loss,
                reason = "element counts above 2^53 lose precision in a rate; acceptable"
            )]
            let elements = elements as f64;
            Some(elements * 1e9 / self.median_ns)
        } else {
            None
        }
    }
}

/// The median of a non-empty sorted slice: the middle element, or the mean of the two middle
/// elements for an even length.
fn median_of_sorted(sorted: &[f64]) -> f64 {
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[mid]
    } else {
        f64::midpoint(sorted[mid - 1], sorted[mid])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(samples: &[f64]) -> Summary {
        Summary::from_samples(samples).expect("valid samples")
    }

    /// Verifies: BENCH-R020
    #[test]
    fn median_is_the_middle_value_or_the_mean_of_the_two_middle_values() {
        assert_eq!(summary(&[7.0]).median_ns, 7.0);
        assert_eq!(summary(&[9.0, 1.0, 5.0]).median_ns, 5.0);
        // Sorted: 1, 2, 4, 8 -> (2 + 4) / 2.
        assert_eq!(summary(&[8.0, 2.0, 1.0, 4.0]).median_ns, 3.0);
        assert_eq!(summary(&[2.5, 2.5]).median_ns, 2.5);
    }

    /// Verifies: BENCH-R021
    #[test]
    fn mad_is_the_unscaled_median_of_absolute_deviations() {
        // Median 3; deviations 2, 1, 0, 1, 97 -> sorted 0, 1, 1, 2, 97 -> 1.
        assert_eq!(summary(&[1.0, 2.0, 3.0, 4.0, 100.0]).mad_ns, 1.0);
        // Median 2.5; deviations 1.5, 0.5, 0.5, 1.5 -> (0.5 + 1.5) / 2 = 1.
        assert_eq!(summary(&[1.0, 2.0, 3.0, 4.0]).mad_ns, 1.0);
        assert_eq!(summary(&[6.0, 6.0, 6.0]).mad_ns, 0.0);
        assert_eq!(summary(&[5.0]).mad_ns, 0.0);
    }

    /// Verifies: BENCH-R022
    #[test]
    fn min_max_and_count_come_from_the_samples() {
        let s = summary(&[3.0, 0.5, 12.0, 4.0]);
        assert_eq!((s.min_ns, s.max_ns, s.samples), (0.5, 12.0, 4));
    }

    /// Verifies: BENCH-R023
    #[test]
    fn throughput_is_elements_per_second_at_the_median() {
        // Median 250 ns per iteration, 1000 elements -> 4e9 elements per second.
        let s = summary(&[200.0, 250.0, 300.0]);
        assert_eq!(s.throughput_per_s(Some(1000)), Some(4e9));
        assert_eq!(s.throughput_per_s(None), None);
        assert_eq!(summary(&[0.0]).throughput_per_s(Some(1)), None);
    }

    /// Verifies: BENCH-R024
    #[test]
    fn empty_or_non_finite_samples_give_no_summary() {
        assert_eq!(Summary::from_samples(&[]), None);
        assert_eq!(Summary::from_samples(&[1.0, f64::NAN]), None);
        assert_eq!(Summary::from_samples(&[f64::INFINITY]), None);
    }
}
