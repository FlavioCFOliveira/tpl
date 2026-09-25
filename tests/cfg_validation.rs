//! The five configuration checks of the fifty-ninth edition, asserted on the
//! bytes the process emits: entry names on `tpl cfg database add`
//! (`FR-CONF-048`, rmp `#264`), the default entry must exist (`FR-CFG-054`,
//! `#271`), `-d` where it has no effect (`FR-GLOB-007`, `#266`), blocks as
//! suggestion candidates (`FR-CFG-012`, `#277`), and line continuation in a
//! `password_command` (`FR-CONF-025`, `#278`).
//!
//! No body needs a server: every command here reads or writes `.tpl/.cfg`
//! alone.

#[path = "support/sandbox.rs"]
mod sandbox;

use std::process::Output;

use sandbox::Sandbox;

/// A project declaring two entries, `shop` the default.
const TWO: &str = "[core]\ndatabase = \"shop\"\n\n[database.shop]\nhost = \"h\"\n\n\
                   [database.crm]\nhost = \"h\"\n";

/// The labelled line of a diagnostic, without its label.
fn line(written: &str, label: &str) -> String {
    written
        .lines()
        .find(|line| line.starts_with(label))
        .unwrap_or_else(|| panic!("no {label:?} line in {written:?}"))
        .trim_start_matches(label)
        .trim_start()
        .to_owned()
}

/// Asserts `printed` is a refusal with `code` in the four lines of
/// `FR-ERR-008` and nothing on stdout, and returns stderr.
fn refused(printed: &Output, code: i32, spelled: &str) -> String {
    let written = String::from_utf8_lossy(&printed.stderr).into_owned();

    assert_eq!(printed.status.code(), Some(code), "{spelled}: {written}");
    assert!(printed.stdout.is_empty(), "{spelled} wrote to stdout");
    let labels: Vec<&str> = written
        .lines()
        .map(|line| line.split_once(':').map_or(line, |(label, _)| label))
        .collect();
    assert_eq!(
        labels,
        ["error", "cause", "hint", "exit"],
        "{spelled}: {written}"
    );

    written
}

// ------------------------------------------------------ FR-CONF-048, #264 ---

#[test]
fn fr_conf_048_add_refuses_a_name_outside_the_rule_with_64_and_writes_nothing() {
    let long = "a".repeat(65);
    for (name, fault) in [
        (
            "bad-name",
            "holds a character other than a letter, a digit or an underscore",
        ),
        (
            "a b",
            "holds a character other than a letter, a digit or an underscore",
        ),
        (
            "${DB}",
            "holds a character other than a letter, a digit or an underscore",
        ),
        ("", "is empty"),
        (long.as_str(), "is longer than 64 characters"),
    ] {
        let sandbox = Sandbox::new();
        sandbox.project(TWO);
        let before = sandbox.configuration();

        let printed = sandbox.run(&["cfg", "database", "add", name, "--host", "h"]);
        let written = refused(&printed, 64, &format!("add {name:?}"));
        let cause = line(&written, "cause: ");

        // The rule is stated, and a value outside it — which is outside the
        // set of FR-ERR-022 by the same rule — is described, not reproduced.
        assert!(cause.contains(fault), "{name:?}: {cause}");
        assert!(
            cause.contains("an entry name is 1 to 64 letters, digits or underscores"),
            "{name:?}: {cause}"
        );
        if !name.is_empty() {
            assert!(!cause.contains(name), "{name:?} was reproduced: {cause}");
        }
        assert!(
            line(&written, "hint:  ").contains("tpl cfg database add <name>"),
            "{name:?}: {written}"
        );
        assert_eq!(
            sandbox.configuration(),
            before,
            "{name:?}: the file was written"
        );
    }
}

// ------------------------------------------------------ FR-CFG-054, #271 ---

#[test]
fn fr_cfg_054_setting_the_default_to_an_undeclared_entry_is_66_with_a_suggestion() {
    let sandbox = Sandbox::new();
    sandbox.project(TWO);
    let before = sandbox.configuration();

    let printed = sandbox.run(&["cfg", "set", "core.database", "shpo"]);
    let written = refused(&printed, 66, "set core.database shpo");

    assert_eq!(
        line(&written, "error: "),
        "database entry 'shpo' does not exist"
    );
    assert_eq!(
        line(&written, "cause: "),
        "core.database names an entry, and .tpl/.cfg declares no entry 'shpo'"
    );
    assert_eq!(
        line(&written, "hint:  "),
        "did you mean 'shop'? set it with: tpl cfg set core.database shop"
    );
    assert_eq!(sandbox.configuration(), before, "the file was written");
}

#[test]
fn fr_cfg_054_the_comparison_is_byte_for_byte_and_the_hint_falls_back_in_order() {
    // Another ASCII case is another name. With no candidate admitted the
    // entries are listed; with no entry at all the hint adds one.
    for (configuration, value, hint) in [
        (TWO, "SHOP", None),
        (
            TWO,
            "zzzzzz",
            Some("list the entries with: tpl cfg database list"),
        ),
        (
            "[core]\n",
            "shop",
            Some("add the entry first with: tpl cfg database add <name>"),
        ),
    ] {
        let sandbox = Sandbox::new();
        sandbox.project(configuration);
        let before = sandbox.configuration();

        let printed = sandbox.run(&["cfg", "set", "core.database", value]);
        let written = refused(&printed, 66, &format!("set core.database {value}"));

        if let Some(hint) = hint {
            assert_eq!(line(&written, "hint:  "), hint, "{value}");
        }
        assert_eq!(
            sandbox.configuration(),
            before,
            "{value}: the file was written"
        );
    }
}

#[test]
fn fr_cfg_054_a_malformed_value_is_still_64_and_a_declared_one_is_written() {
    let sandbox = Sandbox::new();
    sandbox.project(TWO);
    let before = sandbox.configuration();

    let printed = sandbox.run(&["cfg", "set", "core.database", "bad-x"]);
    refused(&printed, 64, "set core.database bad-x");
    assert_eq!(sandbox.configuration(), before);

    let printed = sandbox.run(&["cfg", "set", "core.database", "crm"]);
    assert_eq!(
        printed.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&printed.stderr)
    );
    let file = String::from_utf8(sandbox.configuration()).expect("the file is UTF-8");
    assert!(file.contains("database = \"crm\""), "{file}");
}

#[test]
fn fr_help_011_the_exit_codes_of_cfg_set_carry_66() {
    let sandbox = Sandbox::new();
    let printed = sandbox.run(&["help", "cfg", "set"]);
    let help = String::from_utf8(printed.stdout).expect("help is UTF-8");
    let codes = help
        .split("\nEXIT CODES\n")
        .nth(1)
        .expect("the help carries EXIT CODES");

    assert!(
        codes
            .lines()
            .any(|line| line.starts_with("  66  EX_NOINPUT")),
        "{codes}"
    );
}

// ------------------------------------------------------ FR-GLOB-007, #266 ---

#[test]
fn fr_glob_007_d_on_a_command_that_uses_no_entry_changes_nothing_and_says_nothing() {
    // Each pair runs in a sandbox of its own, so `init` creates the same
    // project in both; the bytes and the code are compared whole.
    for arguments in [
        &["template", "list"][..],
        &["init"][..],
        &["help"][..],
        &["help", "template", "list"][..],
    ] {
        let run = |with_flag: bool| {
            let sandbox = Sandbox::new();
            if arguments[0] != "init" {
                sandbox.project(TWO);
            }
            let mut vector: Vec<&str> = Vec::new();
            if with_flag {
                vector.extend(["-d", "shop"]);
            }
            vector.extend_from_slice(arguments);
            let printed = sandbox.run(&vector);
            let created = sandbox.path(".tpl/.cfg").exists();

            (printed, created)
        };

        let (without, created_without) = run(false);
        let (with, created_with) = run(true);

        assert_eq!(with.status.code(), without.status.code(), "{arguments:?}");
        assert_eq!(with.stdout, without.stdout, "{arguments:?}");
        assert_eq!(
            String::from_utf8_lossy(&with.stderr),
            String::from_utf8_lossy(&without.stderr),
            "{arguments:?}: -d was reported"
        );
        assert_eq!(created_with, created_without, "{arguments:?}");
    }
}

#[test]
fn fr_cfg_051_d_on_database_add_still_warns() {
    let sandbox = Sandbox::new();
    sandbox.project(TWO);

    let printed = sandbox.run(&["-d", "shop", "cfg", "database", "add", "n2", "--host", "h"]);

    assert_eq!(printed.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&printed.stderr)
            .starts_with("warning: -d/--database has no effect on tpl cfg database add"),
        "{}",
        String::from_utf8_lossy(&printed.stderr)
    );
}

// ------------------------------------------------------ FR-CFG-012, #277 ---

#[test]
fn fr_cfg_012_an_absent_entry_block_suggests_a_declared_one_and_shows_it() {
    let sandbox = Sandbox::new();
    sandbox.project(TWO);
    let before = sandbox.configuration();

    let printed = sandbox.run(&["cfg", "unset", "database.shpo"]);
    let written = refused(&printed, 66, "unset database.shpo");
    let hint = line(&written, "hint:  ");

    assert_eq!(
        hint,
        "did you mean 'database.shop'? show it with: tpl cfg database show shop"
    );
    assert!(!hint.contains("cfg unset"), "{hint}");
    assert_eq!(sandbox.configuration(), before);
}

#[test]
fn fr_cfg_012_with_no_candidate_the_hint_lists_the_entries_or_the_file() {
    for (configuration, key, hint) in [
        (
            TWO,
            "database.zzzzzz",
            "list the entries with: tpl cfg database list",
        ),
        (
            "[database.shop]\nhost = \"h\"\n",
            "core",
            "show every key and its value with: tpl cfg list",
        ),
        (
            "[core]\n",
            "database",
            "show every key and its value with: tpl cfg list",
        ),
    ] {
        let sandbox = Sandbox::new();
        sandbox.project(configuration);
        let before = sandbox.configuration();

        let printed = sandbox.run(&["cfg", "unset", key]);
        let written = refused(&printed, 66, &format!("unset {key}"));
        let given = line(&written, "hint:  ");

        assert_eq!(given, hint, "{key}");
        assert!(!given.contains("cfg unset"), "{key}: {given}");
        assert_eq!(sandbox.configuration(), before, "{key}");
    }
}

// ------------------------------------------------------ FR-CONF-025, #278 ---

#[test]
fn fr_conf_025_a_line_continuation_is_removed_outside_single_quotes() {
    for (supplied, stored) in [
        ("get a\\\nb", ["get", "ab"]),
        ("get \"a\\\nb\"", ["get", "ab"]),
        ("get 'a\\\nb'", ["get", "a\\\nb"]),
    ] {
        let sandbox = Sandbox::new();
        sandbox.project(TWO);

        let printed = sandbox.run(&["cfg", "set", "database.shop.password_command", supplied]);

        assert_eq!(
            printed.status.code(),
            Some(0),
            "{supplied:?}: {}",
            String::from_utf8_lossy(&printed.stderr)
        );
        let file = String::from_utf8(sandbox.configuration()).expect("the file is UTF-8");
        let parsed: toml::Table = toml::from_str(&file).expect("the file is TOML");
        let written: Vec<&str> = parsed["database"]["shop"]["password_command"]
            .as_array()
            .expect("an array is stored")
            .iter()
            .map(|word| word.as_str().expect("each word is a string"))
            .collect();

        assert_eq!(written, stored, "{supplied:?}: {file}");
    }
}
