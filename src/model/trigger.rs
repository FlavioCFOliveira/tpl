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

catalogued! {
    /// The statement a trigger fires on (`FR-CAT-050`).
    ///
    /// All three were observed, and all six combinations of the three events
    /// and the two timings appear in the fixture — which is why `FR-CAT-050`
    /// exempts this field pair from the bounded claim it records for the rest.
    /// The field pair is also the only place the model learns either fact.
    ///
    /// *The set is closed by that observation and not by the catalogue, and
    /// this is the one field of the five whose declaration is **not** identical
    /// across the window*: the event column is `varchar(6)` on `10.11`, `11.4`
    /// and `11.8` and `varchar(20)` on `12.3`, which is difference 9 of
    /// `FR-SRV-038`. The declared width behind a recorded set has therefore
    /// already grown inside the window, and a fourth event is carried verbatim
    /// under `FR-CAT-055`.
    TriggerEvent from "EVENT_MANIPULATION" {
        /// The trigger fires on `INSERT`.
        Insert = "INSERT",
        /// The trigger fires on `UPDATE`.
        Update = "UPDATE",
        /// The trigger fires on `DELETE`.
        Delete = "DELETE",
    }
}

catalogued! {
    /// When a trigger fires relative to its event (`FR-CAT-050`).
    ///
    /// The field is declared `varchar(6)` and `NOT NULL` on all four series,
    /// never an `ENUM`, so a third timing is carried verbatim under
    /// `FR-CAT-055`.
    TriggerTiming from "ACTION_TIMING" {
        /// The trigger fires before the statement takes effect.
        Before = "BEFORE",
        /// The trigger fires after it.
        After = "AFTER",
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
    pub event: TriggerEvent<'a>,

    /// When it fires relative to that statement.
    pub timing: TriggerTiming<'a>,

    /// The action order, which read `1` on all six triggers of the fixture.
    pub action_order: u64,

    /// The body **as written**, with newlines and identifier case preserved —
    /// unlike a view definition, which the server rewrites — or [`None`] where
    /// the catalogue returned SQL `NULL` (`FR-CAT-056`).
    ///
    /// The field is declared nullable, `longtext`, on all four series. It was
    /// populated on all six triggers of the fixture, and `FR-PRIV-020` is why
    /// an absent one is not a privilege marking: a reader without the privilege
    /// receives no trigger row at all rather than a row with an empty body.
    pub statement: Option<Cow<'a, str>>,

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

    /// The definer, or [`None`] where the catalogue returned SQL `NULL`
    /// (`FR-CAT-056`).
    ///
    /// The field is declared nullable, `varchar(384)`, on all four series. It
    /// read one user on all six triggers of the fixture.
    pub definer: Option<Cow<'a, str>>,

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

    fn trigger(event: TriggerEvent<'static>, timing: TriggerTiming<'static>) -> Trigger<'static> {
        Trigger {
            name: Cow::Borrowed("trg_consignment_audit"),
            event,
            timing,
            action_order: 1,
            statement: Some(Cow::Borrowed("BEGIN\n  SET NEW.updated_at = NOW();\nEND")),
            orientation: Cow::Borrowed("ROW"),
            old_row_alias: Cow::Borrowed("OLD"),
            new_row_alias: Cow::Borrowed("NEW"),
            sql_mode: Cow::Borrowed("STRICT_TRANS_TABLES,NO_ENGINE_SUBSTITUTION"),
            definer: Some(Cow::Borrowed("root@localhost")),
            character_set_client: Cow::Borrowed("utf8mb4"),
            collation_connection: Cow::Borrowed("utf8mb4_general_ci"),
            database_collation: Cow::Borrowed("utf8mb4_unicode_520_ci"),
        }
    }

    #[test]
    fn fr_cat_050_the_three_events_and_the_two_timings_round_trip() {
        // FR-CAT-050: all three events and both timings were observed, and all
        // six combinations appear in the fixture.
        for event in [
            TriggerEvent::Insert,
            TriggerEvent::Update,
            TriggerEvent::Delete,
        ] {
            assert_eq!(TriggerEvent::from_catalogue(event.name()), event);
        }

        for timing in [TriggerTiming::Before, TriggerTiming::After] {
            assert_eq!(TriggerTiming::from_catalogue(timing.name()), timing);
        }
    }

    #[test]
    fn fr_cat_055_an_event_or_a_timing_outside_the_recorded_set_is_carried_verbatim() {
        // FR-CAT-055: neither field is an `ENUM` on any series, and the event
        // column is the one of the five whose declared width already grew
        // inside the window — `varchar(6)` on three series and `varchar(20)`
        // on 12.3, difference 9 of FR-SRV-038. A fourth event is carried, not
        // refused.
        let truncating = TriggerEvent::from_catalogue("TRUNCATE");
        let instead = TriggerTiming::from_catalogue("INSTEAD OF");

        assert_eq!(
            truncating,
            TriggerEvent::Unrecorded(Cow::Borrowed("TRUNCATE"))
        );
        assert_eq!(truncating.name(), "TRUNCATE");
        assert_eq!(truncating.recorded(), None);

        assert_eq!(
            instead,
            TriggerTiming::Unrecorded(Cow::Borrowed("INSTEAD OF"))
        );
        assert_eq!(instead.name(), "INSTEAD OF");
        assert_eq!(instead.recorded(), None);
    }

    #[test]
    fn fr_cat_056_the_definer_and_the_statement_are_absent_rather_than_empty() {
        // FR-CAT-056: both fields are declared nullable on all four series,
        // and the model carries `null` rather than the empty string — which is
        // what lets a consumer tell an absent body from an empty one.
        let absent = Trigger {
            statement: None,
            definer: None,
            ..trigger(TriggerEvent::Insert, TriggerTiming::Before)
        };

        assert_eq!(absent.statement, None);
        assert_eq!(absent.definer, None);

        let written = serde_json::to_string(&absent).expect("a trigger serialises");

        assert!(written.contains(r#""statement":null"#), "{written}");
        assert!(written.contains(r#""definer":null"#), "{written}");
    }

    #[test]
    fn fr_cat_050_the_row_aliases_are_constants_and_the_event_is_where_availability_is_read() {
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
    fn fr_cat_050_the_statement_is_carried_as_written() {
        // FR-CAT-050: newlines and case preserved, unlike a view definition.
        let fired = trigger(TriggerEvent::Update, TriggerTiming::Before);

        let statement = fired
            .statement
            .as_deref()
            .expect("the fixture states a body");

        assert!(statement.contains('\n'));
        assert!(statement.contains("BEGIN"));
    }
}
