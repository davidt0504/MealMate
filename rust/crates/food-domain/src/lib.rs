//! Food domain types (PRD v3 §8). Depends on the household kernel, never the reverse.
#![forbid(unsafe_code)]

pub mod preference;
pub mod recipe;
pub mod restriction;

use household_core::HouseholdId;
use jiff::Span;
use thiserror::Error;

pub use jiff::civil::Date as CivilDate; // re-exported so dependents need no jiff dependency
pub use preference::*;
pub use recipe::*;
pub use restriction::*;

pub const DEFAULT_CYCLE_DAYS: u32 = 7;
pub const MIN_CYCLE_DAYS: u32 = 1;
pub const MAX_CYCLE_DAYS: u32 = 31; // largest cycle expressible inside one calendar month

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlanningError {
    #[error("cycle length must be {MIN_CYCLE_DAYS}..={MAX_CYCLE_DAYS} days, got {0}")]
    CycleLengthOutOfRange(u32),
    #[error("a planning cycle must enable at least one meal slot")]
    EmptyMealScope,
    #[error("unknown meal slot {0:?}")]
    UnknownMealSlot(String),
    #[error("{0:?} is not a civil date in YYYY-MM-DD form")]
    InvalidDate(String),
    #[error("a {length_days}-day cycle from {anchor} runs past the end of the calendar")]
    CycleOverflow { anchor: String, length_days: u32 },
}

/// Meal slots in chronological order within a day. Declaration order is the canonical
/// order every read and DTO uses, so a round-trip never depends on insertion order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MealSlot {
    Breakfast,
    Lunch,
    Dinner,
}

impl MealSlot {
    pub const ALL: [MealSlot; 3] = [MealSlot::Breakfast, MealSlot::Lunch, MealSlot::Dinner];

    pub fn as_str(self) -> &'static str {
        match self {
            MealSlot::Breakfast => "breakfast",
            MealSlot::Lunch => "lunch",
            MealSlot::Dinner => "dinner",
        }
    }

    /// Matches exactly and does not trim, mirroring `id_newtype!`'s "accepts verbatim,
    /// never trims" rule in `household-core`.
    pub fn parse(raw: &str) -> Result<Self, PlanningError> {
        MealSlot::ALL
            .into_iter()
            .find(|slot| slot.as_str() == raw)
            .ok_or_else(|| PlanningError::UnknownMealSlot(raw.to_owned()))
    }
}

/// The meal slots a household plans. Non-empty by construction — the inner `Vec` is private,
/// so `new` and `dinner_only` are the only ways in. A cycle with no slots would generate
/// nothing, and "this week is off" is a per-occurrence concept (MVP-012), not a scope.
/// Held sorted in `MealSlot` order and deduplicated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MealScope(Vec<MealSlot>);

impl MealScope {
    /// AC-3: dinner defaults on, breakfast and lunch are opt-in (PRD v2 §7.3, card constraints).
    pub fn dinner_only() -> Self {
        Self(vec![MealSlot::Dinner])
    }

    pub fn new(slots: impl IntoIterator<Item = MealSlot>) -> Result<Self, PlanningError> {
        let mut slots: Vec<MealSlot> = slots.into_iter().collect();
        slots.sort_unstable();
        slots.dedup();
        if slots.is_empty() {
            return Err(PlanningError::EmptyMealScope);
        }
        Ok(Self(slots))
    }

    pub fn slots(&self) -> &[MealSlot] {
        &self.0
    }

    pub fn contains(&self, slot: MealSlot) -> bool {
        self.0.contains(&slot)
    }
}

/// Parses exactly `YYYY-MM-DD` and nothing else. The shape is checked here, before the
/// underlying parser is consulted, because jiff follows the Temporal grammar — measured at
/// 0.2.35, it also accepts `20260829` basic format, `2026-08-29T00:00:00-05:00` and
/// `2026-08-29 00:00:00`, silently discarding the time. Accepting an offset datetime would be
/// exactly the UTC reinterpretation the card's stop condition forbids, so the wrapper rejects
/// it regardless of what jiff would do. Out-of-range but correctly shaped dates
/// (`2026-13-01`, `2026-02-30`) are jiff's to reject, and it does.
pub fn parse_civil_date(raw: &str) -> Result<CivilDate, PlanningError> {
    let shaped = raw.len() == 10
        && raw.as_bytes().iter().enumerate().all(|(i, b)| match i {
            4 | 7 => *b == b'-',
            _ => b.is_ascii_digit(),
        });
    if !shaped {
        return Err(PlanningError::InvalidDate(raw.to_owned()));
    }
    raw.parse()
        .map_err(|_| PlanningError::InvalidDate(raw.to_owned()))
}

/// Always ISO `YYYY-MM-DD`, never a locale format.
pub fn format_civil_date(date: CivilDate) -> String {
    date.to_string()
}

/// A household's planning rhythm. The anchor is a civil date, never an instant: cycle dates
/// come from calendar arithmetic only, so no cycle date can shift under a timezone or DST
/// conversion (PRD v3 §8, invariant 4).
///
/// Fields are private because the anchor/length pair carries a cross-field invariant — the
/// whole cycle must fit the calendar — that no per-field newtype can express. `new` and
/// `default_for` are the only constructors, so a value of this type is always valid and
/// `dates()` cannot fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningCycle {
    household_id: HouseholdId,
    anchor: CivilDate,
    length_days: u32,
    scope: MealScope,
}

impl PlanningCycle {
    /// AC-3: dinner-only, seven days, anchored at whatever civil date the caller is in —
    /// never a weekday assumption (invariant 4, card constraint 1).
    pub fn default_for(
        household_id: HouseholdId,
        anchor: CivilDate,
    ) -> Result<Self, PlanningError> {
        Self::new(
            household_id,
            anchor,
            DEFAULT_CYCLE_DAYS,
            MealScope::dinner_only(),
        )
    }

    pub fn new(
        household_id: HouseholdId,
        anchor: CivilDate,
        length_days: u32,
        scope: MealScope,
    ) -> Result<Self, PlanningError> {
        if !(MIN_CYCLE_DAYS..=MAX_CYCLE_DAYS).contains(&length_days) {
            return Err(PlanningError::CycleLengthOutOfRange(length_days));
        }
        // The cross-field invariant: the cycle's last date must be representable. Proving it
        // here is what lets `dates()` be infallible and what stops storage ever writing a
        // cycle whose reads would fail.
        if anchor
            .checked_add(Span::new().days(i64::from(length_days) - 1))
            .is_err()
        {
            return Err(PlanningError::CycleOverflow {
                anchor: format_civil_date(anchor),
                length_days,
            });
        }
        Ok(Self {
            household_id,
            anchor,
            length_days,
            scope,
        })
    }

    pub fn household_id(&self) -> &HouseholdId {
        &self.household_id
    }

    pub fn anchor(&self) -> CivilDate {
        self.anchor
    }

    pub fn length_days(&self) -> u32 {
        self.length_days
    }

    pub fn scope(&self) -> &MealScope {
        &self.scope
    }

    /// The cycle's consecutive civil dates, `length_days` of them starting at the anchor.
    /// Infallible: the constructors already proved the range fits.
    pub fn dates(&self) -> Vec<CivilDate> {
        let mut dates = Vec::with_capacity(self.length_days as usize);
        let mut date = self.anchor;
        for step in 0..self.length_days {
            dates.push(date);
            if step + 1 < self.length_days {
                date = date
                    .tomorrow()
                    .expect("constructors proved the whole cycle fits the calendar");
            }
        }
        dates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_strings_round_trip() {
        assert!(!MealSlot::ALL.is_empty());
        for slot in MealSlot::ALL {
            assert_eq!(MealSlot::parse(slot.as_str()).unwrap(), slot);
        }
        assert_eq!(
            MealSlot::ALL.map(MealSlot::as_str),
            ["breakfast", "lunch", "dinner"]
        );
    }

    #[test]
    fn unknown_slot_is_rejected() {
        for raw in ["supper", "", " dinner", "Dinner"] {
            assert_eq!(
                MealSlot::parse(raw).unwrap_err(),
                PlanningError::UnknownMealSlot(raw.to_owned()),
                "{raw:?} must not parse"
            );
        }
    }

    #[test]
    fn dinner_only_is_the_default_scope() {
        assert_eq!(MealScope::dinner_only().slots(), [MealSlot::Dinner]);
        assert!(MealScope::dinner_only().contains(MealSlot::Dinner));
        assert!(!MealScope::dinner_only().contains(MealSlot::Lunch));
    }

    #[test]
    fn scope_is_canonical_and_deduplicated() {
        let scope =
            MealScope::new([MealSlot::Dinner, MealSlot::Breakfast, MealSlot::Dinner]).unwrap();
        assert_eq!(scope.slots(), [MealSlot::Breakfast, MealSlot::Dinner]);
    }

    #[test]
    fn empty_scope_is_rejected() {
        assert_eq!(
            MealScope::new([]).unwrap_err(),
            PlanningError::EmptyMealScope
        );
    }

    // --- Step 4: PlanningCycle and civil-date generation -------------------------------

    fn hid(raw: &str) -> HouseholdId {
        HouseholdId::new(raw).unwrap()
    }

    fn cycle(anchor: &str, length_days: u32) -> PlanningCycle {
        PlanningCycle::new(
            hid("h"),
            parse_civil_date(anchor).unwrap(),
            length_days,
            MealScope::dinner_only(),
        )
        .unwrap()
    }

    /// The cycle's dates as ISO strings, so every boundary case can be asserted literally.
    fn dates_of(anchor: &str, length_days: u32) -> Vec<String> {
        cycle(anchor, length_days)
            .dates()
            .into_iter()
            .map(format_civil_date)
            .collect()
    }

    #[test]
    fn default_cycle_is_seven_dinner_only_days_from_the_anchor() {
        let anchor = parse_civil_date("2026-08-29").unwrap();
        let c = PlanningCycle::default_for(hid("h"), anchor).unwrap();
        assert_eq!(c.length_days(), 7);
        assert_eq!(c.scope().slots(), [MealSlot::Dinner]);
        assert_eq!(c.dates().first().copied(), Some(anchor));
    }

    #[test]
    fn dates_are_consecutive_and_length_days_long() {
        for length in [1, 3, 7, 14, 31] {
            let dates = cycle("2026-08-29", length).dates();
            assert_eq!(dates.len() as u32, length);
            for pair in dates.windows(2) {
                assert!(pair[1] > pair[0]);
                assert_eq!(pair[0].tomorrow().unwrap(), pair[1]);
            }
        }
    }

    #[test]
    fn accessors_report_what_was_constructed() {
        let anchor = parse_civil_date("2026-08-29").unwrap();
        let scope = MealScope::new([MealSlot::Breakfast, MealSlot::Dinner]).unwrap();
        let c = PlanningCycle::new(hid("h-1"), anchor, 14, scope.clone()).unwrap();
        assert_eq!(c.household_id().as_str(), "h-1");
        assert_eq!(c.anchor(), anchor);
        assert_eq!(c.length_days(), 14);
        assert_eq!(c.scope(), &scope);
    }

    #[test]
    fn cycle_crosses_a_month_boundary() {
        assert_eq!(
            dates_of("2026-08-29", 7),
            [
                "2026-08-29",
                "2026-08-30",
                "2026-08-31",
                "2026-09-01",
                "2026-09-02",
                "2026-09-03",
                "2026-09-04",
            ]
        );
    }

    #[test]
    fn cycle_crosses_a_year_boundary() {
        assert_eq!(
            dates_of("2026-12-28", 7),
            [
                "2026-12-28",
                "2026-12-29",
                "2026-12-30",
                "2026-12-31",
                "2027-01-01",
                "2027-01-02",
                "2027-01-03",
            ]
        );
    }

    #[test]
    fn cycle_crosses_february_in_a_common_year() {
        assert_eq!(
            dates_of("2026-02-26", 4),
            ["2026-02-26", "2026-02-27", "2026-02-28", "2026-03-01"]
        );
    }

    #[test]
    fn cycle_crosses_february_in_a_leap_year() {
        assert_eq!(
            dates_of("2028-02-26", 5),
            [
                "2028-02-26",
                "2028-02-27",
                "2028-02-28",
                "2028-02-29",
                "2028-03-01",
            ]
        );
    }

    /// 2100 is divisible by 4 but not 400, so it is not a leap year.
    #[test]
    fn cycle_crosses_a_century_non_leap_year() {
        assert_eq!(
            dates_of("2100-02-27", 3),
            ["2100-02-27", "2100-02-28", "2100-03-01"]
        );
    }

    #[test]
    fn length_out_of_range_is_rejected() {
        let anchor = parse_civil_date("2026-08-29").unwrap();
        for bad in [0, 32] {
            assert_eq!(
                PlanningCycle::new(hid("h"), anchor, bad, MealScope::dinner_only()).unwrap_err(),
                PlanningError::CycleLengthOutOfRange(bad)
            );
        }
        for good in [MIN_CYCLE_DAYS, MAX_CYCLE_DAYS] {
            assert!(PlanningCycle::new(hid("h"), anchor, good, MealScope::dinner_only()).is_ok());
        }
    }

    /// A representation round-tripping through an instant in a DST-observing zone would
    /// drop 2026-03-08's missing hour and duplicate or skip a date here.
    #[test]
    fn cycle_spanning_spring_forward_has_no_missing_day() {
        assert_eq!(
            dates_of("2026-03-08", 7),
            [
                "2026-03-08",
                "2026-03-09",
                "2026-03-10",
                "2026-03-11",
                "2026-03-12",
                "2026-03-13",
                "2026-03-14",
            ]
        );
    }

    #[test]
    fn cycle_spanning_fall_back_has_no_repeated_day() {
        assert_eq!(
            dates_of("2026-11-01", 7),
            [
                "2026-11-01",
                "2026-11-02",
                "2026-11-03",
                "2026-11-04",
                "2026-11-05",
                "2026-11-06",
                "2026-11-07",
            ]
        );
    }

    /// Australia's DST start. The northern cases alone would not catch a hard-coded
    /// northern-hemisphere rule.
    #[test]
    fn cycle_spanning_a_southern_hemisphere_dst_change() {
        assert_eq!(
            dates_of("2026-10-04", 7),
            [
                "2026-10-04",
                "2026-10-05",
                "2026-10-06",
                "2026-10-07",
                "2026-10-08",
                "2026-10-09",
                "2026-10-10",
            ]
        );
    }

    #[test]
    fn date_strings_are_iso_regardless_of_ambient_locale() {
        assert_eq!(
            format_civil_date(parse_civil_date("2026-01-05").unwrap()),
            "2026-01-05"
        );
        assert_eq!(
            parse_civil_date("05/01/2026").unwrap_err(),
            PlanningError::InvalidDate("05/01/2026".to_owned())
        );
    }

    #[test]
    fn parse_rejects_malformed_and_impossible_dates() {
        // The last two are rejected by the wrapper's shape check: jiff 0.2.35 accepts
        // `20260203` basic format, and rejects `2026-2-3` on its own.
        for raw in ["", "2026-13-01", "2026-02-30", "2026-2-3", "20260203"] {
            assert_eq!(
                parse_civil_date(raw).unwrap_err(),
                PlanningError::InvalidDate(raw.to_owned()),
                "{raw:?} must not parse"
            );
        }
    }

    /// The card's stop condition in its purest form: an instant must never be silently
    /// reinterpreted as a civil date. jiff 0.2.35 accepts the offset and space-separated
    /// forms on its own, so this is the wrapper's shape check being load-bearing.
    #[test]
    fn parse_rejects_a_datetime_or_instant() {
        for raw in [
            "2026-08-29T23:00:00Z",
            "2026-08-29T00:00:00-05:00",
            "2026-08-29 00:00:00",
        ] {
            assert_eq!(
                parse_civil_date(raw).unwrap_err(),
                PlanningError::InvalidDate(raw.to_owned()),
                "{raw:?} must not parse"
            );
        }
    }

    #[test]
    fn constructors_reject_a_cycle_that_runs_past_the_calendar() {
        let anchor = parse_civil_date("9999-12-31").unwrap();
        assert_eq!(
            PlanningCycle::new(hid("h"), anchor, 31, MealScope::dinner_only()).unwrap_err(),
            PlanningError::CycleOverflow {
                anchor: "9999-12-31".to_owned(),
                length_days: 31,
            }
        );
        assert!(matches!(
            PlanningCycle::default_for(hid("h"), anchor),
            Err(PlanningError::CycleOverflow { .. })
        ));
        // A one-day cycle ends on its own anchor, so the last representable date still works.
        assert_eq!(
            PlanningCycle::new(hid("h"), anchor, 1, MealScope::dinner_only())
                .unwrap()
                .dates(),
            vec![anchor]
        );
    }
}
