//! A trigger of a table (`FR-CAT-014`, `FR-CAT-050`).
//!
//! Thirteen properties from a catalogue field list of 22. Four fields are row
//! identity, one is the table the trigger is carried on, four were never
//! observed populated and are not carried under `BR-CAT-005`, and the creation
//! timestamp is volatile: it is a wall-clock time that differs between two
//! containers of the **same** series, so carrying it would make `FR-SRV-026`
//! unsatisfiable outright.
//!
//! *The two row-alias fields say nothing about availability, and the model does
//! not claim they do.* They hold `OLD` and `NEW` on all six rows of the
//! fixture, whatever the event — including the `INSERT` triggers where `OLD`
//! does not exist and the `DELETE` triggers where `NEW` does not. What is
//! available follows from [`Trigger::event`], and a template that branches on
//! the aliases branches on a constant. They are carried because they are what
//! the catalogue states, and the limit is recorded because a value that is
//! present and misleading is worse than one that is absent.
//!
//! *There is no trigger comment field, and there is no field for a hidden
//! trigger either.* `FR-PRIV-020` is the stated limit: a reader without the
//! privilege receives zero rows, which is byte-for-byte what a table with no
//! triggers returns, so `triggers` can never appear in a `restricted` marking.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// The statement a trigger fires on (`FR-CAT-050`).
///
/// All three were observed, and all six combinations of the three events and
/// the two timings appear in the fixture — which is why `FR-CAT-050` exempts
/// this field pair from the bounded claim it records for the rest. The field
/// pair is also the only place the model learns either fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[non_exhaustive]
pub enum TriggerEvent {
    /// The trigger fires on `INSERT`.
    Insert,

    /// The trigger fires on `UPDATE`.
    Update,

    /// The trigger fires on `DELETE`.
    Delete,
}

impl TriggerEvent {
    /// Reads the event from the event-manipulation field.
    #[must_use]
    pub fn from_catalogue(field: &str) -> Option<Self> {
        match field {
            "INSERT" => Some(Self::Insert),
            "UPDATE" => Some(Self::Update),
            "DELETE" => Some(Self::Delete),
            _ => None,
        }
    }

    /// The spelling the catalogue writes, which is what the document carries.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Insert => "INSERT",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
        }
    }
}

/// When a trigger fires relative to its event (`FR-CAT-050`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[non_exhaustive]
pub enum TriggerTiming {
    /// The trigger fires before the statement takes effect.
    Before,

    /// The trigger fires after it.
    After,
}

impl TriggerTiming {
    /// Reads the timing from the action-timing field.
    #[must_use]
    pub fn from_catalogue(field: &str) -> Option<Self> {
        match field {
            "BEFORE" => Some(Self::Before),
            "AFTER" => Some(Self::After),
            _ => None,
        }
    }

    /// The spelling the catalogue writes.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Before => "BEFORE",
            Self::After => "AFTER",
        }
    }
}

/// One trigger (`FR-CAT-050`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Trigger<'a> {
    /// The trigger's name.
    #[serde(borrow)]
    pub name: Cow<'a, str>,

    /// The statement it fires on.
    pub event: TriggerEvent,

    /// When it fires relative to that statement.
    pub timing: TriggerTiming,

    /// The action order, which read `1` on all six triggers of the fixture.
    pub action_order: u64,

    /// The body **as written**, with newlines and identifier case preserved —
    /// unlike a view definition, which the server rewrites.
    pub statement: Cow<'a, str>,

    /// The action orientation, carried verbatim. It read `ROW` on all six, and
    /// no second value was observed.
    pub orientation: Cow<'a, str>,

    /// The old-row alias, which read `OLD` on every row whatever the event.
    /// It states no availability; see the module documentation.
    pub old_row_alias: Cow<'a, str>,

    /// The new-row alias, which read `NEW` on every row, under the same limit.
    pub new_row_alias: Cow<'a, str>,

    /// The session SQL mode in force when the trigger was created.
    pub sql_mode: Cow<'a, str>,

    /// The definer, which read one user across the fixture.
    pub definer: Cow<'a, str>,

    /// The session character set, passed through verbatim per `FR-SRV-039`.
    pub character_set_client: Cow<'a, str>,

    /// The session collation, passed through verbatim per `FR-SRV-039`.
    ///
    /// This is one of the two fields the divergence register of `FR-SRV-036`
    /// holds: it reads `utf8mb4_general_ci` on `10.11` and
    /// `utf8mb4_uca1400_ai_ci` on the other three, from identical DDL.
    pub collation_connection: Cow<'a, str>,

    /// The schema's collation, passed through verbatim per `FR-SRV-039`.
    pub database_collation: Cow<'a, str>,
}

#[cfg(test)]
mod tests {
    use super::{Trigger, TriggerEvent, TriggerTiming};
    use std::borrow::Cow;

    fn trigger(event: TriggerEvent, timing: TriggerTiming) -> Trigger<'static> {
        Trigger {
            name: Cow::Borrowed("trg_consignment_audit"),
            event,
            timing,
            action_order: 1,
            statement: Cow::Borrowed("BEGIN\n  SET NEW.updated_at = NOW();\nEND"),
            orientation: Cow::Borrowed("ROW"),
            old_row_alias: Cow::Borrowed("OLD"),
            new_row_alias: Cow::Borrowed("NEW"),
            sql_mode: Cow::Borrowed("STRICT_TRANS_TABLES,NO_ENGINE_SUBSTITUTION"),
            definer: Cow::Borrowed("root@localhost"),
            character_set_client: Cow::Borrowed("utf8mb4"),
            collation_connection: Cow::Borrowed("utf8mb4_general_ci"),
            database_collation: Cow::Borrowed("utf8mb4_unicode_520_ci"),
        }
    }

    #[test]
    fn the_three_events_and_the_two_timings_round_trip() {
        // FR-CAT-050: all three events and both timings were observed, and all
        // six combinations appear in the fixture.
        for event in [
            TriggerEvent::Insert,
            TriggerEvent::Update,
            TriggerEvent::Delete,
        ] {
            assert_eq!(TriggerEvent::from_catalogue(event.name()), Some(event));
        }

        for timing in [TriggerTiming::Before, TriggerTiming::After] {
            assert_eq!(TriggerTiming::from_catalogue(timing.name()), Some(timing));
        }

        assert_eq!(TriggerEvent::from_catalogue("TRUNCATE"), None);
        assert_eq!(TriggerTiming::from_catalogue("INSTEAD OF"), None);
    }

    #[test]
    fn the_row_aliases_are_constants_and_the_event_is_where_availability_is_read() {
        // FR-CAT-050: both aliases hold their literal on every row, including
        // the INSERT trigger where OLD does not exist. A template that
        // branches on them branches on a constant.
        let inserting = trigger(TriggerEvent::Insert, TriggerTiming::Before);
        let deleting = trigger(TriggerEvent::Delete, TriggerTiming::After);

        assert_eq!(inserting.old_row_alias, deleting.old_row_alias);
        assert_eq!(inserting.new_row_alias, deleting.new_row_alias);
        assert_ne!(inserting.event, deleting.event);
    }

    #[test]
    fn the_statement_is_carried_as_written() {
        // FR-CAT-050: newlines and case preserved, unlike a view definition.
        let fired = trigger(TriggerEvent::Update, TriggerTiming::Before);

        assert!(fired.statement.contains('\n'));
        assert!(fired.statement.contains("BEGIN"));
    }
}
