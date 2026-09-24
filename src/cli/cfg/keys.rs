//! The dotted-key arm: `tpl cfg get`, `set`, `unset` and `list`.
//!
//! The four ask the same question of the same key space in four ways, and
//! `FR-ERR-035` separates the three codes a key can produce:
//!
//! | Condition | Code | Requirement |
//! |---|---|---|
//! | A key no space admits, supplied to `set` | `64` | `FR-CFG-009` |
//! | A key the file does not carry, supplied to `get` or `unset` | `66` | `FR-CFG-007`, `FR-CFG-012` |
//! | A key the file carries and the space refuses | `78` | `FR-CONF-034` |
//!
//! The third is reached before either of the other two, and not by these
//! commands: step 3 of `FR-ERR-006` reads and validates the file for every
//! command that requires a project, so a key that reaches `set` is met in a
//! file that carries only keys the space admits. `tpl cfg set` given a key
//! outside the space, against a file that already carries one, is therefore
//! `78` and not `64` — which `FR-ERR-007` states and which is easy to
//! implement backwards.
//!
//! **`get` does not redact and the other two printers do.** `BR-CFG-002` makes
//! it the one deliberate exception, because it is a directed read of a named
//! key: whoever types the key knows what they are asking for, and redacting
//! here would leave no way to feed a password to another command. `FR-CFG-013`
//! and `FR-CFG-021` redact in `list`, on the printing path, per
//! [`redact`].
//!
//! **`list` resolves nothing.** `FR-CFG-014` forbids it to expand `${VAR}`, to
//! run `password_command`, or to apply a default, so it prints the document
//! [`config`](crate::project::config) read and never the
//! [`settings`](crate::project::settings) that would be resolved from it.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::Write;

use serde::Serialize;

use super::super::local::Format;
use super::{Supplied, coherence, form, project};
use crate::error::Error;
use crate::output::{self, Document, Source};
use crate::project::config::Configuration;
use crate::project::config::entry::Written;
use crate::project::config::keys::{CoreKey, EntryKey, Key, Target};
use crate::project::config::redact;
use crate::project::edit;

/// The command path `FR-CFG-048` refuses an invocation of, where the key it
/// writes cannot stand beside a key the entry already carries.
const SET: &str = "cfg set";

/// The `data` of `tpl cfg get` (`FR-CFG-036`).
///
/// Two keys, in this order, with the value **as written in the file**,
/// unexpanded and unredacted per `FR-CFG-006`.
#[derive(Debug, Serialize)]
struct Read<'a> {
    /// The key that was read.
    key: &'a str,
    /// Its value, as written.
    value: Written<'a>,
}

/// The `data` of `tpl cfg list` (`FR-CFG-037`).
///
/// The key space of `FR-CONF-002` mirrored as nested objects: `core`, and
/// `database` keyed by entry name. A key absent from the file is **absent from
/// the document** rather than emitted as its default, which `FR-CFG-014`
/// requires and which `OD-18` implements by building the document from the keys
/// the file carries rather than by omitting `None`s.
#[derive(Debug, Serialize)]
struct Listing<'a> {
    /// The `[core]` section.
    core: BTreeMap<&'a str, Written<'a>>,
    /// The `[database.<name>]` blocks, by entry name.
    database: BTreeMap<&'a str, BTreeMap<&'a str, Printed<'a>>>,
}

/// One value of a `[database.<name>]` block, as `FR-CFG-021` prints it.
///
/// It is [`Written`] with the string half widened to a [`Cow`], because the
/// redaction of a DSN composes a new string while every other value is the
/// file's own. The enum is untagged, so it serialises as the value itself and
/// carries no map, per `FR-OUT-013`.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Printed<'a> {
    /// A string, redacted or the file's own.
    Text(Cow<'a, str>),
    /// A TOML integer.
    Number(u64),
    /// A TOML array of strings.
    List(&'a [String]),
}

/// `tpl cfg get <key>` (`FR-CFG-006`, `FR-CFG-007`, `FR-CFG-036`).
///
/// # Errors
///
/// Returns what opening the project returns, and
/// key — with a nearest-match suggestion over the whole key space.
/// key — with a nearest-match suggestion over the keys that do exist.
pub(crate) fn get<W: Write>(out: &mut W, supplied: &Supplied<'_>, key: &str) -> Result<(), Error> {
    let configuration = project(supplied)?.configuration()?;

    let Some(parsed) = Key::parse(key) else {
        // FR-CFG-007: a block is refused for its form, before the question of
        // presence, whether or not the file carries it.
        return Err(match Target::parse(key) {
            Some(Target::Core | Target::Databases) => Error::BlockKeyGiven {
                key: key.to_owned(),
                entry: None,
            },
            Some(Target::Entry(name)) => Error::BlockKeyGiven {
                key: key.to_owned(),
                entry: configuration.entry(&name).is_some().then_some(name),
            },
            Some(Target::Key(_)) | None => configuration.key_not_found(key),
        });
    };
    let Some(value) = configuration.written(&parsed) else {
        return Err(configuration.key_not_found(key));
    };

    match supplied.format() {
        Format::Json => output::emit_to(
            out,
            &Document::new(
                Source::Project,
                Read {
                    key: &parsed.to_string(),
                    value,
                },
            ),
            form(supplied),
        ),
        // FR-CFG-006 prints the value as written, and the worked example feeds
        // it straight into another command, so it is emitted byte for byte with
        // one terminating newline and nothing else.
        Format::Text => output::emit_verbatim(out, &format!("{}\n", value.line())),
    }
}

/// `tpl cfg set <key> <value>` (`FR-CFG-008`, `FR-CFG-009`, `FR-CFG-010`).
///
/// It writes no result: `FR-OUT-023` leaves stdout empty for a command that
/// produces none.
///
/// # Errors
///
/// Returns what opening the project returns,
/// [`Error::UnknownConfigurationKey`] where the key is outside the enumerated
/// space of `FR-CONF-002`, [`Error::MalformedValue`] where the value does not
/// conform to the type that space declares for it,
/// [`Error::IncoherentEntryWrite`] where the key cannot stand beside a key the
/// entry already carries (`FR-CFG-048`), and [`Error::ProjectFileUnwritable`]
/// where the rewrite failed.
pub(crate) fn set(supplied: &Supplied<'_>, key: &str, value: &str) -> Result<(), Error> {
    let project = project(supplied)?;
    let configuration = project.configuration()?;

    let Some(parsed) = Key::parse(key) else {
        return Err(Error::UnknownConfigurationKey {
            key: key.to_owned(),
            nearest: configuration.nearest_key_in_space(key),
        });
    };

    let item = edit::assign(&parsed, value)?;

    // FR-CFG-048: one key of one entry is still a write to that entry, and
    // `tpl cfg set database.<name>.dsn` beside a host the file carries is the
    // invocation the twenty-second edition was written for. The value has
    // already been validated, and the file is still untouched.
    if let Key::Entry { entry, field } = &parsed {
        let dsn = (*field == EntryKey::Dsn).then_some(value);

        coherence::refuse(&configuration, SET, entry, &[*field], dsn)?;
    }

    let mut editor = project.editor()?;
    editor.set(&parsed, item);

    editor.save()
}

/// `tpl cfg unset <key>` (`FR-CFG-011`, `FR-CFG-012`).
///
/// # Errors
///
/// Returns what opening the project returns,
/// [`Error::ConfigurationKeyNotFound`] where the key or block is absent, and
/// [`Error::ProjectFileUnwritable`] where the rewrite failed.
pub(crate) fn unset(supplied: &Supplied<'_>, key: &str) -> Result<(), Error> {
    let project = project(supplied)?;
    let configuration = project.configuration()?;

    let Some(target) = Target::parse(key) else {
        return Err(configuration.key_not_found(key));
    };

    let mut editor = project.editor()?;
    if !editor.remove(&target) {
        return Err(configuration.key_not_found(key));
    }

    // FR-CFG-023: a deletion that takes the entry `core.database` names with it
    // clears the reference too, silently and in the same rewrite, so that the
    // next invocation without -d reports "no database entry selected" rather
    // than looking for an entry the file no longer carries. A leaf does not
    // engage the rule — the entry survives and the reference still resolves.
    if removes_selected_entry(&configuration, &target) {
        editor.remove(&Target::Key(Key::Core(CoreKey::Database)));
    }

    editor.save()
}

/// Whether `target` deletes the entry `core.database` names (`FR-CFG-023`).
///
/// Two targets reach that state: the block of the entry itself, and the whole
/// `[database]` table, which takes every block with it. The twenty-second
/// edition states the obligation over the state rather than over one command,
/// and both commands produce it.
fn removes_selected_entry(configuration: &Configuration, target: &Target) -> bool {
    let Some(selected) = configuration.core().database.as_deref() else {
        return false;
    };

    match target {
        Target::Entry(name) => name == selected,
        Target::Databases => configuration.entry(selected).is_some(),
        Target::Core | Target::Key(_) => false,
    }
}

/// `tpl cfg list` (`FR-CFG-013`, `FR-CFG-014`, `FR-CFG-037`).
///
/// # Errors
///
/// Returns what opening the project returns, and the write conditions of
/// [`output`].
pub(crate) fn list<W: Write>(out: &mut W, supplied: &Supplied<'_>) -> Result<(), Error> {
    let configuration = project(supplied)?.configuration()?;

    match supplied.format() {
        Format::Json => output::emit_to(
            out,
            &Document::new(Source::Project, listing(&configuration)),
            form(supplied),
        ),
        // FR-CFG-013: the contents of .tpl/.cfg, literally, with passwords
        // redacted. Every byte outside a redacted value — the comments, the key
        // order, the spacing — is the file's own.
        Format::Text => output::emit_verbatim(out, &configuration.printed()),
    }
}

/// The document of `FR-CFG-037`, built from the keys the file carries.
fn listing(configuration: &Configuration) -> Listing<'_> {
    let mut core = BTreeMap::new();
    for key in CoreKey::ALL {
        if let Some(value) = configuration.core().written(key) {
            core.insert(key.leaf(), value);
        }
    }

    let mut database = BTreeMap::new();
    for (name, entry) in configuration.entries() {
        let mut block = BTreeMap::new();
        for field in EntryKey::ALL {
            let Some(value) = entry.written(field) else {
                continue;
            };
            block.insert(field.leaf(), printed(field, value));
        }
        database.insert(name, block);
    }

    Listing { core, database }
}

/// `value` as `FR-CFG-021` prints it, for a document that carries it as JSON.
///
/// A key the rule reaches is always printed as a string, because `***` is one
/// and so is a DSN with `***` in it; a value the rule leaves alone keeps the
/// shape the file wrote it in, so a port stays a number.
fn printed(field: EntryKey, value: Written<'_>) -> Printed<'_> {
    match field {
        EntryKey::Password | EntryKey::Dsn => Printed::Text(redact::value(field, value)),
        _ => match value {
            Written::Text(text) => Printed::Text(Cow::Borrowed(text)),
            Written::Number(number) => Printed::Number(number),
            Written::List(members) => Printed::List(members),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::Harness;
    use crate::error::Error;

    #[test]
    fn fr_cfg_006_get_prints_the_value_as_written_and_does_not_redact_it() {
        // FR-CFG-006, BR-CFG-002: the one deliberate exception to redaction.
        let harness = Harness::new("[database.reporting]\nhost = \"a\"\npassword = \"hunter2\"\n");

        assert_eq!(harness.get("database.reporting.password"), "hunter2\n");
    }

    #[test]
    fn fr_cfg_006_get_does_not_expand_a_reference() {
        // FR-CFG-006: without expanding ${VAR}.
        let harness = Harness::new("[database.shop]\nhost = \"${SHOP_DB_HOST}\"\n");

        assert_eq!(harness.get("database.shop.host"), "${SHOP_DB_HOST}\n");
    }

    #[test]
    fn fr_cfg_036_get_answers_with_the_value_in_the_shape_the_file_wrote_it() {
        // FR-CFG-036: the value as written, so a port is a JSON number and a
        // password_command a JSON array.
        let harness = Harness::new(
            "[database.shop]\nport = 3307\npassword_command = [\"pass\", \"db/shop\"]\n",
        );

        assert_eq!(
            harness.get_json("database.shop.port"),
            "{\"schema_version\":1,\"source\":\"project\",\"data\":{\"key\":\"database.shop.port\",\"value\":3307}}\n"
        );
        assert_eq!(
            harness.get_json("database.shop.password_command"),
            "{\"schema_version\":1,\"source\":\"project\",\"data\":{\"key\":\"database.shop.password_command\",\"value\":[\"pass\",\"db/shop\"]}}\n"
        );
    }

    #[test]
    fn fr_cfg_007_a_key_absent_from_the_file_is_a_named_object_that_does_not_exist() {
        // FR-CFG-007: 66, with a nearest-match suggestion over the key space,
        // each candidate paired with whether the file sets it.
        let harness = Harness::new("[core]\ndatabase = \"shop\"\n");

        let condition = harness.get_refused("core.databse");

        match condition {
            Error::ConfigurationKeyNotFound { ref nearest, .. } => {
                assert_eq!(nearest, &[("core.database".to_owned(), true)]);
            }
            other => panic!("expected a missing key, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 66);
    }

    #[test]
    fn fr_cfg_007_a_key_in_the_space_the_file_does_not_set_is_also_sixty_six() {
        // FR-CFG-007: exiting 0 with empty output would be indistinguishable
        // from a key whose value is empty.
        let harness = Harness::new("[core]\ndatabase = \"shop\"\n");

        assert_eq!(harness.get_refused("core.query_timeout").exit_code(), 66);
    }

    #[test]
    fn fr_cfg_007_the_suggestion_covers_the_key_space_and_says_what_is_unset() {
        // FR-CFG-007: a slip in a key the file does not set is suggested too,
        // and the hint says the candidate is unset, so that the caller is not
        // sent to a command that exits 66 again (BR-ERR-004).
        let harness =
            Harness::new("[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"h\"\n");

        let refused = harness.get_refused("core.conect_timeout");
        match &refused {
            Error::ConfigurationKeyNotFound { nearest, .. } => {
                assert_eq!(nearest, &[("core.connect_timeout".to_owned(), false)]);
            }
            other => panic!("expected a missing key, got {other:?}"),
        }
        let rendered = crate::diagnostics::rendered(&refused);
        assert!(
            rendered.contains(
                "hint:  did you mean 'core.connect_timeout'? .tpl/.cfg does not set it, so its \
                 default applies; list every key, its type and its default with: tpl help cfg set"
            ),
            "{rendered}"
        );

        // The <name> segment is bound to the entries the file declares, and a
        // candidate the file sets is not said to be unset.
        let refused = harness.get_refused("database.shop.hots");
        let rendered = crate::diagnostics::rendered(&refused);
        assert!(
            rendered.contains("hint:  did you mean 'database.shop.host'? list every key"),
            "{rendered}"
        );

        // FR-CFG-012: unset draws the same population and says the same.
        let refused = harness.unset("database.shop.pasword").expect_err("absent");
        let rendered = crate::diagnostics::rendered(&refused);
        assert!(
            rendered.contains(
                "hint:  did you mean 'database.shop.password'? .tpl/.cfg does not set it; list \
                 every key"
            ),
            "{rendered}"
        );
    }

    #[test]
    fn fr_cfg_008_set_writes_the_value_under_the_key() {
        // FR-CFG-008.
        let harness = Harness::new("[core]\n");

        harness
            .set("core.database", "shop")
            .expect("the key is written");

        assert_eq!(harness.written(), "[core]\ndatabase = \"shop\"\n");
    }

    #[test]
    fn fr_out_023_set_writes_nothing_to_stdout() {
        // FR-OUT-023: a command that writes no result leaves stdout empty.
        let harness = Harness::new("[core]\n");

        assert_eq!(harness.set_output("core.database", "shop"), "");
    }

    #[test]
    fn fr_cfg_009_set_refuses_a_key_outside_the_enumerated_space() {
        // FR-CFG-009: 64, with a nearest-match suggestion over the known keys.
        let harness = Harness::new("[core]\n");

        let condition = harness
            .set("core.databse", "shop")
            .expect_err("the key is outside the space");

        match condition {
            Error::UnknownConfigurationKey { ref nearest, .. } => {
                assert_eq!(nearest, &["core.database".to_owned()]);
            }
            other => panic!("expected an unknown key, got {other:?}"),
        }
        assert_eq!(condition.exit_code(), 64);
    }

    #[test]
    fn fr_cfg_010_set_refuses_a_value_that_does_not_conform_to_the_declared_type() {
        // FR-CFG-010: 64.
        let harness = Harness::new("[core]\n");

        let condition = harness
            .set("core.query_timeout", "soon")
            .expect_err("the value is not a positive integer");

        assert!(matches!(condition, Error::MalformedValue { .. }));
        assert_eq!(condition.exit_code(), 64);
    }

    #[test]
    fn fr_err_007_set_against_a_file_carrying_an_unknown_key_is_seventy_eight_and_not_sixty_four() {
        // FR-ERR-007, and the consequence its own text says is easy to
        // implement backwards: the file is validated at step 3, before the
        // command resolves a key of its own at step 4.
        let harness = Harness::new("[core]\ndatabse = \"shop\"\n");

        let condition = harness
            .set("core.databse", "shop")
            .expect_err("the file carries an unknown key");

        assert!(matches!(
            condition,
            Error::ConfigurationKeyOutsideSpace { .. }
        ));
        assert_eq!(condition.exit_code(), 78);
    }

    #[test]
    fn fr_cfg_011_unset_deletes_a_leaf_and_a_whole_block() {
        // FR-CFG-011.
        let harness = Harness::new(
            "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"a\"\nuser = \"b\"\n",
        );

        harness
            .unset("database.shop.user")
            .expect("the leaf is deleted");
        assert_eq!(
            harness.written(),
            "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"a\"\n"
        );

        // The block takes `core.database` with it, per FR-CFG-023, which
        // `fr_cfg_023_unset_of_the_selected_block_clears_the_reference` states
        // on its own.
        harness
            .unset("database.shop")
            .expect("the block is deleted");
        assert_eq!(harness.written(), "[core]\n");
    }

    #[test]
    fn fr_cfg_012_unset_refuses_a_key_the_file_does_not_carry() {
        // FR-CFG-012: 66.
        let harness = Harness::new("[core]\ndatabase = \"shop\"\n");

        assert_eq!(
            harness
                .unset("core.query_timeout")
                .expect_err("the key is absent")
                .exit_code(),
            66
        );
        assert_eq!(
            harness
                .unset("nonsense")
                .expect_err("the key names nothing")
                .exit_code(),
            66
        );
    }

    #[test]
    fn fr_cfg_013_list_prints_the_file_literally_with_passwords_redacted() {
        // FR-CFG-013, FR-CFG-021, FR-SEC-003.
        let original = concat!(
            "# a note the author wrote\n",
            "[core]\n",
            "database = \"shop\"\n",
            "\n",
            "[database.shop]\n",
            "user = \"alice\"\n",
            "password = \"hunter2\"\n",
        );
        let harness = Harness::new(original);

        let printed = harness.list();

        assert!(!printed.contains("hunter2"), "{printed}");
        assert!(printed.contains("# a note the author wrote"), "{printed}");
        assert!(printed.contains("password = \"***\""), "{printed}");
        assert!(printed.contains("user = \"alice\""), "{printed}");
    }

    #[test]
    fn fr_cfg_021_list_prints_a_reference_exactly_as_written() {
        // FR-CFG-021, third row: a password living in an environment variable
        // never reaches stdout through this command.
        let harness = Harness::new("[database.shop]\npassword = \"${SHOP_DB_PASSWORD}\"\n");

        assert!(harness.list().contains("${SHOP_DB_PASSWORD}"));
    }

    #[test]
    fn fr_cfg_037_list_mirrors_the_key_space_and_omits_what_the_file_does_not_set() {
        // FR-CFG-037: nested objects, with an absent key absent rather than
        // emitted as its default.
        let harness = Harness::new(
            "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"db\"\nport = 3307\npassword = \"hunter2\"\n",
        );

        assert_eq!(
            harness.list_json(),
            concat!(
                "{\"schema_version\":1,\"source\":\"project\",\"data\":{",
                "\"core\":{\"database\":\"shop\"},",
                "\"database\":{\"shop\":{\"host\":\"db\",\"password\":\"***\",\"port\":3307}}",
                "}}\n"
            )
        );
    }

    #[test]
    fn fr_out_033_list_of_a_project_with_nothing_set_is_an_empty_document_and_a_success() {
        // FR-OUT-033, FR-PROJ-018: an empty listing is the ordinary first state
        // of a project.
        let harness = Harness::new("");

        assert_eq!(harness.list(), "");
        assert_eq!(
            harness.list_json(),
            "{\"schema_version\":1,\"source\":\"project\",\"data\":{\"core\":{},\"database\":{}}}\n"
        );
    }

    #[test]
    fn fr_cfg_021_list_redacts_the_password_inside_a_dsn_and_leaves_the_rest_visible() {
        // FR-CFG-021, second row.
        let harness = Harness::new(
            "[database.shop]\ndsn = \"mysql://alice:hunter2@db.example.com:3306/shop\"\n",
        );

        let document = harness.list_json();

        assert!(!document.contains("hunter2"), "{document}");
        assert!(
            document.contains("mysql://alice:***@db.example.com:3306/shop"),
            "{document}"
        );
    }

    #[test]
    fn fr_cfg_014_list_does_not_resolve_the_configuration() {
        // FR-CFG-014: no expansion, no child process, no defaults. A file whose
        // only password source is a command that does not exist still lists.
        let harness = Harness::new(
            "[database.shop]\nhost = \"${ABSENT}\"\npassword_command = [\"/nonexistent/helper\"]\n",
        );

        let printed = harness.list();

        assert!(printed.contains("${ABSENT}"), "{printed}");
        assert!(!harness.list_json().contains("connect_timeout"));
    }

    #[test]
    fn fr_cfg_013_the_text_of_a_listing_is_the_bytes_the_file_holds() {
        // The literal print is a copy, not a re-serialisation: a document with
        // unusual spacing comes back with it.
        let original = "[core]\ndatabase    =    \"shop\"\n";
        let harness = Harness::new(original);

        assert_eq!(harness.list(), original);
    }

    #[test]
    fn fr_cfg_010_set_refuses_a_dsn_outside_the_three_requirements_with_the_same_code_as_the_flag()
    {
        // FR-CFG-010 with FR-CFG-031: the declared type of database.<name>.dsn
        // is those three requirements, so both write paths admit the same set
        // and refuse with the same code.
        let harness = Harness::new("[core]\n");

        for value in [
            "postgres://db.example.com/shop",
            "mysql://db.example.com/shop?charset=utf8",
            "mysql://db.example.com",
        ] {
            let condition = harness
                .set("database.shop.dsn", value)
                .expect_err("the value is outside what the file admits");

            assert_eq!(condition.exit_code(), 64, "{value}");
            assert_eq!(harness.written(), "[core]\n", "{value}");
        }

        harness
            .set(
                "database.shop.dsn",
                "mysql://alice:${SHOP_DB_PASSWORD}@db.example.com:3306/shop",
            )
            .expect("a reference is opaque text within the field it occupies");
        assert!(
            harness.written().contains("${SHOP_DB_PASSWORD}"),
            "{}",
            harness.written()
        );
    }

    #[test]
    fn fr_cfg_048_set_refuses_a_key_that_cannot_stand_beside_one_the_entry_carries() {
        // FR-CFG-048: the entry as it would stand after the write is a
        // combination FR-CONF-007 refuses, so nothing is written and the
        // invocation is 64 — the file it met is valid.
        let file = "[database.shop]\ndsn = \"mysql://alice@db.example.com/shop\"\n";
        let harness = Harness::new(file);

        let condition = harness
            .set("database.shop.host", "10.0.1.5")
            .expect_err("the entry states its connection as a URL");

        assert_eq!(condition.exit_code(), 64);
        assert_eq!(harness.written(), file);

        match condition {
            Error::IncoherentEntryWrite {
                ref entry,
                ref written,
                ref conflicting,
                ..
            } => {
                assert_eq!(entry, "shop");
                assert_eq!(written, "database.shop.host");
                assert_eq!(conflicting, "database.shop.dsn");
            }
            other => panic!("expected an incoherent write, got {other:?}"),
        }
    }

    #[test]
    fn fr_cfg_048_set_refuses_the_second_password_source_of_an_entry() {
        // FR-CFG-048 over the fifth row of FR-CONF-007.
        let file = "[database.shop]\nhost = \"db\"\npassword = \"hunter2\"\n";
        let harness = Harness::new(file);

        let condition = harness
            .set("database.shop.password_command", "pass db/shop")
            .expect_err("two password sources in one entry");

        assert_eq!(condition.exit_code(), 64);
        assert_eq!(harness.written(), file);
    }

    #[test]
    fn fr_cfg_048_set_writes_a_key_the_entry_can_hold_beside_what_it_carries() {
        // FR-CFG-048 refuses only what FR-CONF-007 refuses: the fourth row is
        // admitted, and so is every key outside the table.
        let harness = Harness::new("[database.shop]\nhost = \"db\"\n");

        harness
            .set("database.shop.password_command", "pass db/shop")
            .expect("a password command composes with the discrete fields");
        harness
            .set("database.shop.tls", "required")
            .expect("tls takes no part in the table");

        assert!(
            harness.written().contains("password_command"),
            "{}",
            harness.written()
        );
    }

    #[test]
    fn fr_cfg_023_unset_of_the_selected_block_clears_the_reference() {
        // FR-CFG-023: the same rewrite, silently, with no change to the exit
        // code and nothing written to either stream.
        let harness = Harness::new(
            "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"db\"\nuser = \"alice\"\n",
        );

        harness
            .unset("database.shop")
            .expect("the block is deleted");

        assert_eq!(harness.written(), "[core]\n");
    }

    #[test]
    fn fr_cfg_023_unset_of_a_leaf_of_the_selected_entry_leaves_the_reference_alone() {
        // FR-CFG-023: a deletion that leaves the entry in place does not engage
        // the rule — the entry still exists and core.database still resolves.
        let harness = Harness::new(
            "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"db\"\nuser = \"alice\"\n",
        );

        harness
            .unset("database.shop.user")
            .expect("the leaf is deleted");

        assert_eq!(
            harness.written(),
            "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"db\"\n"
        );
    }

    #[test]
    fn fr_cfg_023_unset_of_a_block_the_reference_does_not_name_leaves_it_alone() {
        // FR-CFG-023 fires on the entry core.database names, and on no other.
        let harness = Harness::new(
            "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"db\"\n\n\
             [database.reporting]\nhost = \"other\"\n",
        );

        harness
            .unset("database.reporting")
            .expect("the block is deleted");

        assert_eq!(
            harness.written(),
            "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"db\"\n"
        );
    }

    #[test]
    fn fr_cfg_023_unset_of_every_block_clears_a_reference_to_one_of_them() {
        // FR-CFG-023 is stated over the state and not over one command, and
        // `tpl cfg unset database` reaches the same state as the block of the
        // entry the reference names.
        let harness =
            Harness::new("[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"db\"\n");

        harness.unset("database").expect("the table is deleted");

        assert_eq!(harness.written(), "[core]\n");
    }
}
