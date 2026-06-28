//! Minimal FSRS-6 stub for spaced-repetition scheduling.
//!
//! This module implements a simplified version of the FSRS-6 algorithm.
//! A production deployment should integrate the `fsrs-rs` crate for the
//! full parameter-fitted algorithm. This stub is API-compatible and sufficient
//! for integration testing and cold-start scheduling.
//!
//! # FSRS-6 Simplified Formula
//!
//! - Stability (`s`): days until 90% retention
//! - Difficulty (`d`): [1,10], LWW-merged
//! - Interval: next review in `s * growth_factor` days
//! - Lapses increase stability decay

use crate::types::{CardState, FSRSCard};
use chrono::{DateTime, Duration, Utc};

/// Rating for an FSRS review answer, matching FSRS-6 conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rating {
    /// Forgot — again. Triggers a lapse.
    Again = 1,
    /// Correct but hard.
    Hard = 2,
    /// Correct and comfortable.
    Good = 3,
    /// Correct and easy.
    Easy = 4,
}

/// Compute the next review schedule given a card, a rating, and the current time.
///
/// Returns a new `FSRSCard` with updated state, stability, and due date.
/// The input card is not mutated.
///
/// # Simplified FSRS-6 Parameters Used
///
/// | Rating | Stability multiplier | State transition |
/// |--------|----------------------|-----------------|
/// | Again  | × 0.5 (lapse)       | → Relearning    |
/// | Hard   | × 0.9               | → Review        |
/// | Good   | × 1.2               | → Review        |
/// | Easy   | × 1.5               | → Review        |
///
/// Stability minimum is clamped at 0.1 days.
/// Interval minimum is 1 day.
pub fn next_review(card: &FSRSCard, rating: Rating, now: DateTime<Utc>) -> FSRSCard {
    let mut next = card.clone();
    next.last_review = Some(now);
    next.reps += 1;

    let interval_days: i64 = match rating {
        Rating::Again => {
            next.lapses += 1;
            next.state = CardState::Relearning;
            1
        }
        Rating::Hard => {
            next.state = CardState::Review;
            ((card.stability * 0.8) as i64).max(1)
        }
        Rating::Good => {
            next.state = CardState::Review;
            (card.stability as i64).max(1)
        }
        Rating::Easy => {
            next.state = CardState::Review;
            ((card.stability * 1.3) as i64).max(1)
        }
    };

    // Stability growth (simplified FSRS-6 multipliers)
    next.stability = match rating {
        Rating::Again => card.stability * 0.5,
        Rating::Hard => card.stability * 0.9,
        Rating::Good => card.stability * 1.2,
        Rating::Easy => card.stability * 1.5,
    }
    .max(0.1);

    next.due = now + Duration::days(interval_days);
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_card() -> FSRSCard {
        FSRSCard {
            stability: 4.0,
            difficulty: 5.0,
            due: Utc::now(),
            state: CardState::Review,
            reps: 3,
            lapses: 0,
            last_review: None,
        }
    }

    #[test]
    fn again_triggers_lapse_and_relearning_state() {
        let card = new_card();
        let now = Utc::now();
        let next = next_review(&card, Rating::Again, now);

        assert_eq!(next.state, CardState::Relearning);
        assert_eq!(next.lapses, card.lapses + 1);
        assert_eq!(next.reps, card.reps + 1);
        // stability decays
        assert!(next.stability < card.stability);
        // next review is tomorrow
        let delta = next.due - now;
        assert_eq!(delta.num_days(), 1);
    }

    #[test]
    fn hard_moves_to_review_with_shorter_interval() {
        let card = new_card();
        let now = Utc::now();
        let next = next_review(&card, Rating::Hard, now);

        assert_eq!(next.state, CardState::Review);
        assert_eq!(next.lapses, card.lapses);
        // Interval ≈ stability * 0.8 = 3 days
        let delta = next.due - now;
        assert!(delta.num_days() >= 1);
        assert!(delta.num_days() <= (card.stability * 0.8 + 1.0) as i64);
    }

    #[test]
    fn good_stays_in_review_with_same_interval() {
        let card = new_card();
        let now = Utc::now();
        let next = next_review(&card, Rating::Good, now);

        assert_eq!(next.state, CardState::Review);
        let delta = next.due - now;
        assert_eq!(delta.num_days(), card.stability as i64);
    }

    #[test]
    fn easy_extends_interval_and_stability() {
        let card = new_card();
        let now = Utc::now();
        let next = next_review(&card, Rating::Easy, now);

        assert_eq!(next.state, CardState::Review);
        assert!(next.stability > card.stability);
        let delta = next.due - now;
        assert!(delta.num_days() >= card.stability as i64);
    }

    #[test]
    fn stability_minimum_is_0_1() {
        let mut card = new_card();
        card.stability = 0.1;
        let now = Utc::now();
        let next = next_review(&card, Rating::Again, now);
        assert!(next.stability >= 0.1);
    }

    #[test]
    fn last_review_is_set_to_now() {
        let card = new_card();
        let now = Utc::now();
        let next = next_review(&card, Rating::Good, now);
        assert_eq!(next.last_review, Some(now));
    }

    #[test]
    fn reps_always_increments() {
        let card = new_card();
        let now = Utc::now();
        for rating in [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy] {
            let next = next_review(&card, rating, now);
            assert_eq!(next.reps, card.reps + 1);
        }
    }
}
