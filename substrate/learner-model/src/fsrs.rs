//! FSRS scheduling for spaced repetition, through the `rs-fsrs` crate
//! (change-rah-009; README "FSRS dependency decision").
//!
//! This module is an adapter. The card this crate persists (`FSRSCard`) is
//! mapped to `rs_fsrs::Card`, the scheduler computes the next state with the
//! crate's default parameters (FSRS-5, 19 weights, request retention 0.9,
//! short-term scheduler on, fuzz off), and the result is mapped back. Every
//! review reads and updates `difficulty` through FSRS's mean-reversion rule,
//! which the previous stub never did.
//!
//! The scheduler is deterministic for a given (card, rating, now): the same
//! observation set folds to the same card on every replica (`store::fold_concept`).

use crate::types::{CardState, FSRSCard};
use chrono::{DateTime, Utc};
use rs_fsrs::{Card, State, FSRS};
use serde::{Deserialize, Serialize};

/// Rating for an FSRS review answer, matching FSRS conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

impl From<Rating> for rs_fsrs::Rating {
    fn from(r: Rating) -> Self {
        match r {
            Rating::Again => rs_fsrs::Rating::Again,
            Rating::Hard => rs_fsrs::Rating::Hard,
            Rating::Good => rs_fsrs::Rating::Good,
            Rating::Easy => rs_fsrs::Rating::Easy,
        }
    }
}

fn to_state(s: &CardState) -> State {
    match s {
        CardState::New => State::New,
        CardState::Learning => State::Learning,
        CardState::Review => State::Review,
        CardState::Relearning => State::Relearning,
    }
}

fn from_state(s: State) -> CardState {
    match s {
        State::New => CardState::New,
        State::Learning => CardState::Learning,
        State::Review => CardState::Review,
        State::Relearning => CardState::Relearning,
    }
}

fn to_rs_card(card: &FSRSCard, now: DateTime<Utc>) -> Card {
    // A card that was never reviewed has no last_review; rs-fsrs computes
    // elapsed days from it only for non-New states, so `now` is a safe stand-in.
    let last_review = card.last_review.unwrap_or(now);
    let elapsed_days = match card.state {
        CardState::New => 0,
        _ => (now - last_review).num_days().max(0),
    };
    let scheduled_days = match card.last_review {
        Some(lr) => (card.due - lr).num_days().max(0),
        None => 0,
    };
    Card {
        due: card.due,
        stability: card.stability,
        difficulty: card.difficulty,
        elapsed_days,
        scheduled_days,
        reps: i32::try_from(card.reps).unwrap_or(i32::MAX),
        lapses: i32::try_from(card.lapses).unwrap_or(i32::MAX),
        state: to_state(&card.state),
        last_review,
    }
}

fn from_rs_card(card: &Card) -> FSRSCard {
    FSRSCard {
        stability: card.stability,
        difficulty: card.difficulty,
        due: card.due,
        state: from_state(card.state),
        reps: u32::try_from(card.reps).unwrap_or(0),
        lapses: u32::try_from(card.lapses).unwrap_or(0),
        last_review: Some(card.last_review),
    }
}

/// Compute the next review schedule given a card, a rating, and the review time.
///
/// Returns a new `FSRSCard`; the input is not mutated. `stability`,
/// `difficulty`, `due`, `state`, `reps`, `lapses`, and `last_review` all come
/// from the scheduler.
pub fn next_review(card: &FSRSCard, rating: Rating, now: DateTime<Utc>) -> FSRSCard {
    let scheduled = FSRS::default().next(to_rs_card(card, now), now, rating.into());
    from_rs_card(&scheduled.card)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn new_card(now: DateTime<Utc>) -> FSRSCard {
        FSRSCard {
            stability: 1.0,
            difficulty: 5.0,
            due: now,
            state: CardState::New,
            reps: 0,
            lapses: 0,
            last_review: None,
        }
    }

    #[test]
    fn again_triggers_lapse_after_a_review() {
        let now = Utc::now();
        let first = next_review(&new_card(now), Rating::Good, now);
        let later = now + Duration::days(3);
        let next = next_review(&first, Rating::Again, later);
        assert_eq!(next.lapses, first.lapses + 1);
        assert_eq!(next.reps, first.reps + 1);
        assert!(next.stability < first.stability, "a lapse lowers stability");
        assert_eq!(next.last_review, Some(later));
    }

    #[test]
    fn difficulty_is_read_and_updated_on_every_review() {
        let now = Utc::now();
        let card = new_card(now);
        let good = next_review(&card, Rating::Good, now);
        let hard = next_review(&card, Rating::Hard, now);
        assert_ne!(
            good.difficulty, hard.difficulty,
            "rating changes difficulty"
        );
        assert!(
            hard.difficulty > good.difficulty,
            "Hard is harder than Good"
        );
        let again = next_review(&good, Rating::Again, now + Duration::days(2));
        assert!(
            again.difficulty > good.difficulty,
            "a lapse raises difficulty"
        );
        assert!((1.0..=10.0).contains(&again.difficulty));
    }

    #[test]
    fn easy_is_more_stable_than_good() {
        let now = Utc::now();
        let card = new_card(now);
        let good = next_review(&card, Rating::Good, now);
        let easy = next_review(&card, Rating::Easy, now);
        assert!(easy.stability > good.stability);
        assert!(easy.due >= good.due);
    }

    #[test]
    fn reps_always_increment_and_state_leaves_new() {
        let now = Utc::now();
        let card = new_card(now);
        for rating in [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy] {
            let next = next_review(&card, rating, now);
            assert_eq!(next.reps, card.reps + 1);
            assert_ne!(next.state, CardState::New);
        }
    }
}
