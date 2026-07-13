#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisibleRangeBounds {
    pub from: f64,
    pub to: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisibleRangeValidationFailure {
    NonFinite { field: &'static str },
    InvalidOrder { from: f64, to: f64 },
}

pub fn validate_visible_range_bounds(
    from: f64,
    to: f64,
) -> Result<VisibleRangeBounds, VisibleRangeValidationFailure> {
    if !from.is_finite() {
        return Err(VisibleRangeValidationFailure::NonFinite { field: "from" });
    }
    if !to.is_finite() {
        return Err(VisibleRangeValidationFailure::NonFinite { field: "to" });
    }
    if from >= to {
        return Err(VisibleRangeValidationFailure::InvalidOrder { from, to });
    }
    Ok(VisibleRangeBounds { from, to })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoadedRangeObservation {
    pub earliest: f64,
    pub latest: f64,
    pub more_available: bool,
}

impl LoadedRangeObservation {
    pub fn is_valid(self) -> bool {
        self.earliest.is_finite() && self.latest.is_finite() && self.earliest <= self.latest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageStatus {
    Complete,
    Partial,
}

impl CoverageStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
        }
    }
}

pub fn coverage_status(
    bounds: VisibleRangeBounds,
    observation: LoadedRangeObservation,
) -> CoverageStatus {
    if observation.earliest <= bounds.from && observation.latest >= bounds.to {
        CoverageStatus::Complete
    } else {
        CoverageStatus::Partial
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagingStopReason {
    PagingNotNeeded,
    CoverageReached,
    HistoryExhausted,
    NoProgress,
    RequestLimitReached,
    DeadlineElapsed,
}

impl PagingStopReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PagingNotNeeded => "paging_not_needed",
            Self::CoverageReached => "coverage_reached",
            Self::HistoryExhausted => "history_exhausted",
            Self::NoProgress => "no_progress",
            Self::RequestLimitReached => "request_limit_reached",
            Self::DeadlineElapsed => "deadline_elapsed",
        }
    }

    pub fn limit_reached(self) -> bool {
        self == Self::RequestLimitReached
    }

    pub fn timed_out(self) -> bool {
        self == Self::DeadlineElapsed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagingDecision {
    RequestMore,
    Stop(PagingStopReason),
}

pub fn initial_paging_decision(
    bounds: VisibleRangeBounds,
    observation: LoadedRangeObservation,
    deadline_elapsed: bool,
) -> PagingDecision {
    if observation.earliest <= bounds.from {
        return PagingDecision::Stop(PagingStopReason::PagingNotNeeded);
    }
    if !observation.more_available {
        return PagingDecision::Stop(PagingStopReason::HistoryExhausted);
    }
    if deadline_elapsed {
        return PagingDecision::Stop(PagingStopReason::DeadlineElapsed);
    }
    PagingDecision::RequestMore
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressState {
    pub previous_earliest: f64,
    pub current: LoadedRangeObservation,
    pub request_count: usize,
    pub request_limit: usize,
    pub deadline_elapsed: bool,
    pub progress_window_elapsed: bool,
}

pub fn progress_paging_decision(
    bounds: VisibleRangeBounds,
    state: ProgressState,
) -> PagingDecision {
    if state.current.earliest <= bounds.from {
        return PagingDecision::Stop(PagingStopReason::CoverageReached);
    }
    if !state.current.more_available {
        return PagingDecision::Stop(PagingStopReason::HistoryExhausted);
    }
    if state.current.earliest < state.previous_earliest {
        if state.deadline_elapsed {
            return PagingDecision::Stop(PagingStopReason::DeadlineElapsed);
        }
        if state.request_count >= state.request_limit {
            return PagingDecision::Stop(PagingStopReason::RequestLimitReached);
        }
        return PagingDecision::RequestMore;
    }
    if state.deadline_elapsed {
        return PagingDecision::Stop(PagingStopReason::DeadlineElapsed);
    }
    if state.progress_window_elapsed {
        return PagingDecision::Stop(PagingStopReason::NoProgress);
    }
    PagingDecision::RequestMore
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoadedBarPoint {
    pub index: i64,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportApplicationStatus {
    Applied,
    AppliedClamped,
    UnchangedNoOverlap,
    UnchangedNoMatchingBars,
}

impl ViewportApplicationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::AppliedClamped => "applied_clamped",
            Self::UnchangedNoOverlap => "unchanged_no_overlap",
            Self::UnchangedNoMatchingBars => "unchanged_no_matching_bars",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewportDecision {
    pub status: ViewportApplicationStatus,
    pub from_index: Option<i64>,
    pub to_index: Option<i64>,
    pub matching_bar_count: usize,
    pub applied_range: Option<VisibleRangeBounds>,
}

impl ViewportDecision {
    pub fn applied(&self) -> bool {
        matches!(
            self.status,
            ViewportApplicationStatus::Applied | ViewportApplicationStatus::AppliedClamped
        )
    }

    pub fn clamped(&self) -> bool {
        self.status == ViewportApplicationStatus::AppliedClamped
    }
}

pub fn viewport_decision(
    bounds: VisibleRangeBounds,
    loaded: LoadedRangeObservation,
    bars: &[LoadedBarPoint],
) -> ViewportDecision {
    let intersection_from = bounds.from.max(loaded.earliest);
    let intersection_to = bounds.to.min(loaded.latest);
    if intersection_from > intersection_to {
        return ViewportDecision {
            status: ViewportApplicationStatus::UnchangedNoOverlap,
            from_index: None,
            to_index: None,
            matching_bar_count: 0,
            applied_range: None,
        };
    }

    let matching: Vec<_> = bars
        .iter()
        .copied()
        .filter(|bar| {
            bar.timestamp.is_finite()
                && bar.timestamp >= intersection_from
                && bar.timestamp <= intersection_to
        })
        .collect();
    let Some(first) = matching.first() else {
        return ViewportDecision {
            status: ViewportApplicationStatus::UnchangedNoMatchingBars,
            from_index: None,
            to_index: None,
            matching_bar_count: 0,
            applied_range: None,
        };
    };
    let last = matching.last().expect("matching bars has a first element");
    if first.index > last.index {
        return ViewportDecision {
            status: ViewportApplicationStatus::UnchangedNoMatchingBars,
            from_index: None,
            to_index: None,
            matching_bar_count: 0,
            applied_range: None,
        };
    }

    let clamped = intersection_from != bounds.from || intersection_to != bounds.to;
    ViewportDecision {
        status: if clamped {
            ViewportApplicationStatus::AppliedClamped
        } else {
            ViewportApplicationStatus::Applied
        },
        from_index: Some(first.index),
        to_index: Some(last.index),
        matching_bar_count: matching.len(),
        applied_range: Some(VisibleRangeBounds {
            from: first.timestamp,
            to: last.timestamp,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(from: f64, to: f64) -> VisibleRangeBounds {
        validate_visible_range_bounds(from, to).unwrap()
    }

    fn observation(earliest: f64, latest: f64, more_available: bool) -> LoadedRangeObservation {
        LoadedRangeObservation {
            earliest,
            latest,
            more_available,
        }
    }

    #[test]
    fn validates_finite_ordered_bounds() {
        assert_eq!(bounds(1.0, 2.0), VisibleRangeBounds { from: 1.0, to: 2.0 });
        assert_eq!(
            validate_visible_range_bounds(f64::NAN, 2.0),
            Err(VisibleRangeValidationFailure::NonFinite { field: "from" })
        );
        assert_eq!(
            validate_visible_range_bounds(1.0, f64::INFINITY),
            Err(VisibleRangeValidationFailure::NonFinite { field: "to" })
        );
        assert_eq!(
            validate_visible_range_bounds(2.0, 2.0),
            Err(VisibleRangeValidationFailure::InvalidOrder { from: 2.0, to: 2.0 })
        );
    }

    #[test]
    fn initial_decision_separates_left_coverage_from_full_coverage() {
        let request = bounds(150.0, 250.0);
        let loaded = observation(100.0, 200.0, true);

        assert_eq!(
            initial_paging_decision(request, loaded, false),
            PagingDecision::Stop(PagingStopReason::PagingNotNeeded)
        );
        assert_eq!(coverage_status(request, loaded), CoverageStatus::Partial);
    }

    #[test]
    fn initial_decision_prioritizes_coverage_then_exhaustion_then_deadline() {
        let request = bounds(100.0, 200.0);
        assert_eq!(
            initial_paging_decision(request, observation(50.0, 150.0, false), true),
            PagingDecision::Stop(PagingStopReason::PagingNotNeeded)
        );
        assert_eq!(
            initial_paging_decision(request, observation(150.0, 250.0, false), true),
            PagingDecision::Stop(PagingStopReason::HistoryExhausted)
        );
        assert_eq!(
            initial_paging_decision(request, observation(150.0, 250.0, true), true),
            PagingDecision::Stop(PagingStopReason::DeadlineElapsed)
        );
    }

    #[test]
    fn progress_decision_uses_reviewed_terminal_precedence() {
        let request = bounds(100.0, 200.0);
        let base = ProgressState {
            previous_earliest: 200.0,
            current: observation(100.0, 250.0, false),
            request_count: 25,
            request_limit: 25,
            deadline_elapsed: true,
            progress_window_elapsed: true,
        };
        assert_eq!(
            progress_paging_decision(request, base),
            PagingDecision::Stop(PagingStopReason::CoverageReached)
        );
        assert_eq!(
            progress_paging_decision(
                request,
                ProgressState {
                    current: observation(150.0, 250.0, false),
                    ..base
                }
            ),
            PagingDecision::Stop(PagingStopReason::HistoryExhausted)
        );
        assert_eq!(
            progress_paging_decision(
                request,
                ProgressState {
                    current: observation(150.0, 250.0, true),
                    deadline_elapsed: true,
                    ..base
                }
            ),
            PagingDecision::Stop(PagingStopReason::DeadlineElapsed)
        );
        assert_eq!(
            progress_paging_decision(
                request,
                ProgressState {
                    current: observation(150.0, 250.0, true),
                    deadline_elapsed: false,
                    ..base
                }
            ),
            PagingDecision::Stop(PagingStopReason::RequestLimitReached)
        );
        assert_eq!(
            progress_paging_decision(
                request,
                ProgressState {
                    current: observation(200.0, 250.0, true),
                    deadline_elapsed: false,
                    request_count: 1,
                    ..base
                }
            ),
            PagingDecision::Stop(PagingStopReason::NoProgress)
        );
    }

    #[test]
    fn stop_reason_booleans_describe_the_actual_cause() {
        assert!(PagingStopReason::RequestLimitReached.limit_reached());
        assert!(!PagingStopReason::CoverageReached.limit_reached());
        assert!(PagingStopReason::DeadlineElapsed.timed_out());
        assert!(!PagingStopReason::NoProgress.timed_out());
    }

    #[test]
    fn viewport_selects_exact_boundaries_and_one_bar() {
        let loaded = observation(100.0, 300.0, true);
        let bars = [
            LoadedBarPoint {
                index: 4,
                timestamp: 100.0,
            },
            LoadedBarPoint {
                index: 5,
                timestamp: 200.0,
            },
            LoadedBarPoint {
                index: 6,
                timestamp: 300.0,
            },
        ];
        let exact = viewport_decision(bounds(100.0, 300.0), loaded, &bars);
        assert_eq!(exact.status, ViewportApplicationStatus::Applied);
        assert_eq!((exact.from_index, exact.to_index), (Some(4), Some(6)));

        let one = viewport_decision(bounds(200.0, 201.0), loaded, &bars);
        assert_eq!((one.from_index, one.to_index), (Some(5), Some(5)));
        assert_eq!(one.matching_bar_count, 1);
    }

    #[test]
    fn viewport_clamps_left_and_right_overlap() {
        let loaded = observation(100.0, 300.0, true);
        let bars = [
            LoadedBarPoint {
                index: 1,
                timestamp: 100.0,
            },
            LoadedBarPoint {
                index: 2,
                timestamp: 200.0,
            },
            LoadedBarPoint {
                index: 3,
                timestamp: 300.0,
            },
        ];
        let left = viewport_decision(bounds(50.0, 200.0), loaded, &bars);
        assert_eq!(left.status, ViewportApplicationStatus::AppliedClamped);
        assert_eq!((left.from_index, left.to_index), (Some(1), Some(2)));

        let right = viewport_decision(bounds(200.0, 350.0), loaded, &bars);
        assert_eq!(right.status, ViewportApplicationStatus::AppliedClamped);
        assert_eq!((right.from_index, right.to_index), (Some(2), Some(3)));
    }

    #[test]
    fn viewport_leaves_wholly_older_and_newer_ranges_unchanged() {
        let loaded = observation(100.0, 300.0, true);
        let bars = [LoadedBarPoint {
            index: 1,
            timestamp: 100.0,
        }];
        for request in [bounds(1.0, 50.0), bounds(350.0, 400.0)] {
            let decision = viewport_decision(request, loaded, &bars);
            assert_eq!(
                decision.status,
                ViewportApplicationStatus::UnchangedNoOverlap
            );
            assert!(!decision.applied());
            assert_eq!(decision.applied_range, None);
        }
    }

    #[test]
    fn viewport_rejects_weekend_gap_and_reversed_indices() {
        let loaded = observation(100.0, 500.0, true);
        let bars = [
            LoadedBarPoint {
                index: 10,
                timestamp: 100.0,
            },
            LoadedBarPoint {
                index: 11,
                timestamp: 500.0,
            },
        ];
        let gap = viewport_decision(bounds(200.0, 400.0), loaded, &bars);
        assert_eq!(
            gap.status,
            ViewportApplicationStatus::UnchangedNoMatchingBars
        );

        let reversed = [
            LoadedBarPoint {
                index: 11,
                timestamp: 200.0,
            },
            LoadedBarPoint {
                index: 10,
                timestamp: 300.0,
            },
        ];
        let decision = viewport_decision(bounds(200.0, 400.0), loaded, &reversed);
        assert_eq!(
            decision.status,
            ViewportApplicationStatus::UnchangedNoMatchingBars
        );
        assert_eq!(decision.from_index, None);
    }
}
