//! Which days were written on.
//!
//! The dotted calendar asks the session history one question — "was there
//! writing that day?" — and the answer is a set of local calendar days. Which
//! days get a dot is a presentation matter, so the fold lives up here in the
//! app and `esse-core` keeps its one way of reading the file (design.md, D3).

use std::collections::HashSet;

use chrono::{Days, Local, NaiveDate};
use esse_core::Session;

/// How far the dotted line reaches back, today included. A constant, like the
/// session target: settings are an anti-feature (session-calendar spec).
pub const DEPTH: usize = 28;

/// The days at least one session started on, in the machine's own time zone.
///
/// A session belongs to the day the writing *started*: one that ran past
/// midnight marks a single day, the way a person remembers "I wrote on
/// Tuesday" (design.md, D2).
pub fn days_written(sessions: &[Session]) -> HashSet<NaiveDate> {
    sessions
        .iter()
        .map(|session| session.started_at.with_timezone(&Local).date_naive())
        .collect()
}

/// The last [`DEPTH`] days ending on `today`, oldest first, each written on or
/// not — or `None` when none of them was, which means no calendar at all
/// rather than a row of empty dots (session-calendar spec).
pub fn strip(days: &HashSet<NaiveDate>, today: NaiveDate) -> Option<Vec<bool>> {
    let strip: Vec<bool> = (0..DEPTH)
        .rev()
        .map(|back| {
            today
                .checked_sub_days(Days::new(back as u64))
                .is_some_and(|day| days.contains(&day))
        })
        .collect();

    strip.contains(&true).then_some(strip)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;
    use esse_core::model::Timestamp;

    use super::*;

    /// A local wall-clock time, `YYYY-MM-DD HH:MM`, as the store would have
    /// written it. Built through `Local` so the test asks about the same zone
    /// the fold answers in, whatever machine it runs on.
    fn at(local: &str) -> Timestamp {
        NaiveDateTime::parse_from_str(local, "%Y-%m-%d %H:%M")
            .expect("a well-formed local time")
            .and_local_timezone(Local)
            .single()
            .expect("a time that exists exactly once locally")
            .fixed_offset()
    }

    fn session(started_at: &str, duration_min: u32) -> Session {
        Session {
            essay_slug: "pochemu-esse".to_string(),
            started_at: at(started_at),
            duration_min,
        }
    }

    fn day(local: &str) -> NaiveDate {
        NaiveDate::parse_from_str(local, "%Y-%m-%d").expect("a well-formed date")
    }

    #[test]
    fn no_sessions_means_no_days() {
        assert!(days_written(&[]).is_empty());
    }

    #[test]
    fn two_sessions_on_one_day_are_one_day() {
        let days = days_written(&[
            session("2026-06-15 09:30", 25),
            session("2026-06-15 21:05", 40),
        ]);

        assert_eq!(days, HashSet::from([day("2026-06-15")]));
    }

    #[test]
    fn a_session_past_midnight_marks_only_the_day_it_started_on() {
        let days = days_written(&[session("2026-06-15 23:50", 30)]);

        assert_eq!(days, HashSet::from([day("2026-06-15")]));
        assert!(!days.contains(&day("2026-06-16")));
    }

    #[test]
    fn the_strip_ends_on_today_and_reaches_back_four_weeks() {
        let today = day("2026-06-15");
        let days = HashSet::from([
            today,
            day("2026-06-14"),
            // The oldest day still on the line, and the first one off it.
            day("2026-05-19"),
            day("2026-05-18"),
        ]);

        let strip = strip(&days, today).expect("a written month shows a line");

        assert_eq!(strip.len(), DEPTH);
        assert!(strip[DEPTH - 1], "today is the rightmost dot");
        assert!(strip[DEPTH - 2]);
        assert!(strip[0], "the oldest day on the line is 27 days back");
        assert_eq!(strip.iter().filter(|written| **written).count(), 3);
    }

    #[test]
    fn a_month_without_writing_has_no_line_at_all() {
        let today = day("2026-06-15");

        assert!(strip(&HashSet::new(), today).is_none());
        // Writing older than the line does not bring it back.
        assert!(strip(&HashSet::from([day("2026-05-18")]), today).is_none());
    }
}
