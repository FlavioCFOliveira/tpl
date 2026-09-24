//! `FR-CFG-048`: the write that would leave a database entry in a combination
//! `FR-CONF-007` refuses.
//!
//! Three subcommands write a `[database.<name>]` block — `tpl cfg set`,
//! `tpl cfg database add` and `tpl cfg database update` — and all three reach
//! [`refuse`] before they touch the file, so the rule is applied once and the
//! three cannot drift apart.
//!
//! **The rule is applied to the entry as it would stand after the write**: the
//! fields `.tpl/.cfg` already carries, with the fields the invocation names
//! added or changed. Nothing the invocation did not name is removed, replaced
//! or rewritten to make the entry coherent — `FR-CFG-020` and `BR-CFG-001` both
//! forbid that, and `BR-CONF-004` makes it the guess the reader is not allowed
//! to make either.
//!
//! **The refusal is `64` and not `78`.** The file as it stands is valid and
//! stays untouched; what is refused is the invocation, which the caller wrote
//! and can rewrite. The same combination *found in the file* is
//! [`Error::ConflictingEntryKeys`] and `78`, and the predicate that decides
//! both is [`Combination::refused`] — one rule, two faults.

use std::path::Path;

use crate::error::{EntryRepair, Error};
use crate::project::config::entry::Combination;
use crate::project::config::keys::{EntryKey, Key};
use crate::project::config::{Configuration, dsn};

/// Refuses a write that would leave the entry `name` incoherent
/// (`FR-CFG-048`).
///
/// `command` is the command path below `tpl` that writes it, `written` the keys
/// the invocation names, and `dsn` the `dsn` value it writes, where it writes
/// one. Nothing here reads the file again: `configuration` is the document step
/// 3 of `FR-ERR-006` has already validated.
///
/// # Errors
///
/// Returns [`Error::IncoherentEntryWrite`] naming both members of the refused
/// pair, with the repair whose runnable command makes the write legal.
pub(super) fn refuse(
    configuration: &Configuration,
    command: &'static str,
    name: &str,
    written: &[EntryKey],
    dsn: Option<&str>,
) -> Result<(), Error> {
    let carried = configuration.entry(name);

    let mut combination = carried.map(Combination::of).unwrap_or_default();
    for field in written {
        combination.declare(*field, true);
    }

    // The DSN in force after the write: the one the invocation writes, or the
    // one the file carries where it writes none. Both have already been through
    // the grammar — the file's by the reader, the invocation's by `FR-CFG-031`
    // before this is reached — so a value that does not parse here cannot
    // occur, and is read as carrying no password rather than as a second
    // refusal of a value the caller has already been told about.
    let inherited = carried.and_then(|entry| entry.dsn.as_ref().map(|dsn| dsn.value.as_str()));
    let effective = if written.contains(&EntryKey::Dsn) {
        dsn
    } else {
        inherited
    };
    combination = combination.with_dsn_password(effective.is_some_and(|value| {
        dsn::parse(value, "", Path::new("")).is_ok_and(|parsed| parsed.password().is_some())
    }));

    let Some(pair) = combination.refused() else {
        return Ok(());
    };

    // The pair is reported in the order `FR-CFG-048` states it: the key the
    // invocation writes, then the key it cannot stand beside. The file the
    // invocation met was coherent, so at least one member is always a key this
    // invocation writes.
    let (first, second) = pair;
    let (writes, conflicting) = if written.contains(&first) {
        (first, second)
    } else {
        (second, first)
    };

    Err(Error::IncoherentEntryWrite {
        entry: name.to_owned(),
        written: qualified(name, writes),
        conflicting: qualified(name, conflicting),
        repair: repair(command, name, combination, written, conflicting),
    })
}

/// Which runnable command makes the write legal.
fn repair(
    command: &'static str,
    entry: &str,
    combination: Combination,
    written: &[EntryKey],
    conflicting: EntryKey,
) -> EntryRepair {
    if written.contains(&conflicting) {
        return EntryRepair::Restate(command.to_owned());
    }

    // Whether removing the one key resolves the refusal is asked of the rule
    // rather than counted: an entry carrying a DSN beside three discrete fields
    // is still refused once one of the three is gone, and a hint naming that
    // one would not be the command `FR-CFG-048` asks for.
    let mut without = combination;
    without.declare(conflicting, false);

    if without.refused().is_none() {
        return EntryRepair::Unset;
    }

    // Every key the entry carries that the write cannot stand beside, found by
    // asking the rule again after each removal. A pair whose two members are
    // both written cannot occur here, because the file the invocation met was
    // coherent and the pair written in full is the `Restate` above.
    let mut unset = vec![conflicting];
    while let Some((first, second)) = without.refused() {
        let carried = if written.contains(&first) {
            second
        } else {
            first
        };
        if written.contains(&carried) || unset.contains(&carried) {
            break;
        }
        without.declare(carried, false);
        unset.push(carried);
    }

    EntryRepair::Rewrite {
        unset: unset
            .into_iter()
            .map(|field| qualified(entry, field))
            .collect(),
        command,
    }
}

/// One key of one entry, fully qualified as `FR-CONF-002` spells it.
fn qualified(entry: &str, field: EntryKey) -> String {
    Key::Entry {
        entry: entry.to_owned(),
        field,
    }
    .to_string()
}
