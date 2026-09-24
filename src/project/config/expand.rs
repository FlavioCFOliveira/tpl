//! `${VAR}` expansion: the one mechanism by which `tpl` reads the environment.
//!
//! `BR-CONF-003` states what it is for — a secret need not be written to disk —
//! and `FR-CONF-030` states what it is not: not a layer of the precedence of
//! `FR-CONF-029`, because it supplies the value of a key that is already in the
//! file.
//!
//! Five rules govern one scan, and the scan is the whole implementation:
//!
//! | Rule | Requirement |
//! |---|---|
//! | `$$` is a literal `$` | `FR-CONF-020` |
//! | `${VAR}` is the value of `VAR` | `FR-CONF-015` |
//! | An undefined variable is `78`, never an empty substitution | `FR-CONF-022`, `BR-CONF-002` |
//! | A `${` with no closing brace is `78` | `FR-CONF-021` |
//! | The result of an expansion is not re-expanded | `FR-CONF-019` |
//!
//! The last is a property of the scan rather than a check after it: the output
//! is appended to and never read back, so an expanded value carrying `${…}` or
//! `$$` reaches the caller exactly as the environment holds it.
//!
//! **The environment is a parameter.** [`expand`] takes the lookup rather than
//! calling [`std::env::var`] itself, for two reasons that point the same way.
//! `FR-SEC-007` treats the environment as untrusted, and a function that reads
//! it directly cannot be exercised against a hostile value without the process
//! carrying that value; and in edition 2024 setting a variable is `unsafe`,
//! which `#![forbid(unsafe_code)]` denies this crate outright. [`environment`]
//! is the one caller that reads the real environment.

use std::borrow::Cow;
use std::path::Path;

use crate::error::{Error, ReferenceFault};

/// The value of an environment variable, or [`None`] where it is not defined.
///
/// This is the only read of the process environment in the crate, which is what
/// `FR-CLI-021` and `BR-CONF-003` between them allow: one mechanism, reached
/// from one place.
#[allow(
    dead_code,
    reason = "the commands that resolve an entry are a later sprint; BR-CONF-003 makes this the \
              one read of the process environment in the crate, so it is written where the rule \
              is and a test drives the resolution with a table instead"
)]
pub(crate) fn environment(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Expands `${VAR}` in `raw`, once.
///
/// `key` is the fully qualified key whose value this is, and `file` the file
/// that declares it; both appear in the two conditions this can report, per
/// `FR-ERR-034`.
///
/// A value carrying no `$` is returned borrowed and nothing is allocated, which
/// is the ordinary case for every field of every entry.
///
/// # Errors
///
/// Returns [`Error::UnclosedExpansion`] for a `${` with no closing brace
/// (`FR-CONF-021`), [`Error::InvalidReferenceName`] for a reference whose name
/// is not a variable name (`FR-CONF-049`), and [`Error::UndefinedVariable`]
/// for a reference to a variable the lookup does not define (`FR-CONF-022`).
pub(crate) fn expand<'a, L>(
    raw: &'a str,
    lookup: &L,
    key: &str,
    file: &Path,
) -> Result<Cow<'a, str>, Error>
where
    L: Fn(&str) -> Option<String>,
{
    if !raw.contains('$') {
        return Ok(Cow::Borrowed(raw));
    }

    let mut expanded = String::with_capacity(raw.len());
    let mut rest = raw;

    while let Some(at) = rest.find('$') {
        expanded.push_str(&rest[..at]);
        let after = &rest[at + 1..];

        // FR-CONF-020: `$$` is a literal `$`, and the second `$` is consumed
        // rather than left to open an expansion of its own.
        if let Some(tail) = after.strip_prefix('$') {
            expanded.push('$');
            rest = tail;
            continue;
        }

        let Some(opened) = after.strip_prefix('{') else {
            // A `$` that opens nothing is a `$`. `FR-CONF-020` gives the
            // doubled form for a literal one and the corpus states no
            // condition for the single form, so it is carried through rather
            // than refused under a requirement that does not reach it.
            expanded.push('$');
            rest = after;
            continue;
        };

        let Some(end) = opened.find('}') else {
            return Err(Error::UnclosedExpansion {
                key: key.to_owned(),
                file: file.to_owned(),
            });
        };

        let name = &opened[..end];
        // FR-CONF-049: a name no shell can define is a fault in the file, met
        // where the field is expanded, as FR-CONF-021 is.
        if !is_variable_name(name) {
            return Err(Error::InvalidReferenceName {
                key: key.to_owned(),
                file: file.to_owned(),
                name: name.to_owned(),
            });
        }
        let Some(value) = lookup(name) else {
            return Err(Error::UndefinedVariable {
                name: name.to_owned(),
                key: key.to_owned(),
                file: file.to_owned(),
            });
        };

        // FR-CONF-019: the value is appended and never re-read, so a single
        // pass is a property of the loop rather than a check made after it.
        expanded.push_str(&value);
        rest = &opened[end + 1..];
    }

    expanded.push_str(rest);

    Ok(Cow::Owned(expanded))
}

/// Whether `name` is `[A-Za-z_][A-Za-z0-9_]*`, the portable form of a shell
/// variable name (`FR-CONF-049`).
pub(crate) fn is_variable_name(name: &str) -> bool {
    let mut bytes = name.bytes();

    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// The first fault of a reference in `raw`, read by the grammar [`expand`]
/// applies, or [`None`] where every reference is well formed (`FR-CONF-021`,
/// `FR-CONF-049`).
///
/// It is the check a value written on the command line is held to, so that a
/// value the next expansion would refuse is refused where the caller wrote it.
pub(crate) fn reference_fault(raw: &str) -> Option<ReferenceFault> {
    let mut rest = raw;

    while let Some(at) = rest.find('$') {
        let after = &rest[at + 1..];

        // FR-CONF-020: `$$` is a literal `$` and opens nothing.
        if let Some(tail) = after.strip_prefix('$') {
            rest = tail;
            continue;
        }
        let Some(opened) = after.strip_prefix('{') else {
            rest = after;
            continue;
        };
        let Some(end) = opened.find('}') else {
            return Some(ReferenceFault::Unclosed);
        };

        let name = &opened[..end];
        if !is_variable_name(name) {
            return Some(ReferenceFault::Name(name.to_owned()));
        }
        rest = &opened[end + 1..];
    }

    None
}

/// Whether `raw` is exactly one `${VAR}` reference and nothing else.
///
/// `FR-CFG-021` prints `${VAR}` in any field unexpanded, which keeps a password
/// living in an environment variable off stdout. A value that is **partly** a
/// reference is not that case: printing `secret${SUFFIX}` as written would
/// disclose the literal half, so the redaction of a credential-bearing key
/// applies to it.
pub(crate) fn is_whole_reference(raw: &str) -> bool {
    let Some(inner) = raw
        .strip_prefix("${")
        .and_then(|rest| rest.strip_suffix('}'))
    else {
        return false;
    };

    !inner.is_empty() && !inner.contains('}') && !inner.contains('$')
}

#[cfg(test)]
mod tests {
    use super::{expand, is_variable_name, is_whole_reference, reference_fault};
    use crate::error::Error;
    use crate::error::ReferenceFault;
    use std::path::PathBuf;

    /// The file every condition below names.
    fn file() -> PathBuf {
        PathBuf::from("/work/.tpl/.cfg")
    }

    /// A lookup over a fixed table, standing in for the process environment.
    fn table(name: &str) -> Option<String> {
        match name {
            "SHOP_DB_PASSWORD" => Some("hunter2".to_owned()),
            "NESTED" => Some("${INNER}".to_owned()),
            "DOUBLED" => Some("$$".to_owned()),
            "EMPTY" => Some(String::new()),
            "INNER" => Some("reached".to_owned()),
            _ => None,
        }
    }

    /// Expands against [`table`], for a test that expects success.
    fn expanded(raw: &str) -> String {
        expand(raw, &table, "database.shop.password", &file())
            .expect("the value expands")
            .into_owned()
    }

    #[test]
    fn fr_conf_015_a_reference_is_replaced_by_the_value_of_the_variable() {
        // FR-CONF-015: expansion supplies the value of a key already in the
        // file.
        assert_eq!(expanded("${SHOP_DB_PASSWORD}"), "hunter2");
        assert_eq!(expanded("pre-${SHOP_DB_PASSWORD}-post"), "pre-hunter2-post");
    }

    #[test]
    fn a_value_with_no_dollar_is_returned_untouched_and_unallocated() {
        let raw = "db.example.com";
        let value = expand(raw, &table, "database.shop.host", &file()).expect("nothing to expand");

        assert_eq!(value, "db.example.com");
        assert!(matches!(value, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn fr_conf_020_a_doubled_dollar_is_a_literal_dollar() {
        // FR-CONF-020: `$$` denotes a literal `$`.
        assert_eq!(expanded("$$"), "$");
        assert_eq!(expanded("a$$b"), "a$b");
        assert_eq!(expanded("$${SHOP_DB_PASSWORD}"), "${SHOP_DB_PASSWORD}");
        assert_eq!(expanded("$$$$"), "$$");
    }

    #[test]
    fn fr_conf_019_expansion_never_runs_twice_over_its_own_output() {
        // FR-CONF-019: the system SHALL NOT re-expand the result of an
        // expansion. A value that is itself a reference is carried through as
        // text, and a value that is `$$` stays two characters.
        assert_eq!(expanded("${NESTED}"), "${INNER}");
        assert_eq!(expanded("${DOUBLED}"), "$$");
    }

    #[test]
    fn fr_conf_021_an_unclosed_expansion_is_refused() {
        // FR-CONF-021: `${VAR` with no closing brace is 78.
        let refused = expand(
            "${SHOP_DB_PASSWORD",
            &table,
            "database.shop.password",
            &file(),
        )
        .expect_err("an unclosed expansion is refused");

        assert!(matches!(
            refused,
            Error::UnclosedExpansion { ref key, .. } if key == "database.shop.password"
        ));
        assert_eq!(refused.exit_code(), 78);
    }

    #[test]
    fn fr_conf_021_an_unclosed_expansion_after_a_closed_one_is_refused_too() {
        // The scan reaches it at the second `$`, which is the whole of why the
        // rule is a property of the loop.
        let refused = expand(
            "${SHOP_DB_PASSWORD}${OPEN",
            &table,
            "database.shop.password",
            &file(),
        )
        .expect_err("the second reference is unclosed");

        assert!(matches!(refused, Error::UnclosedExpansion { .. }));
    }

    #[test]
    fn fr_conf_022_an_undefined_variable_is_refused_and_is_never_an_empty_substitution() {
        // FR-CONF-022, BR-CONF-002.
        let refused = expand("${ABSENT}", &table, "database.shop.host", &file())
            .expect_err("an undefined variable is refused");

        match refused {
            Error::UndefinedVariable {
                ref name, ref key, ..
            } => {
                assert_eq!(name, "ABSENT");
                assert_eq!(key, "database.shop.host");
            }
            other => panic!("expected an undefined variable, got {other:?}"),
        }
        assert_eq!(refused.exit_code(), 78);
    }

    #[test]
    fn br_conf_002_a_variable_defined_as_the_empty_string_is_defined() {
        // BR-CONF-002 forbids a silent empty substitution for an *undefined*
        // variable. A variable the environment defines as empty is defined, and
        // expands to what it holds.
        assert_eq!(expanded("${EMPTY}"), "");
    }

    #[test]
    fn fr_conf_015_a_lone_dollar_that_opens_nothing_is_carried_through() {
        assert_eq!(expanded("a$b"), "a$b");
        assert_eq!(expanded("trailing$"), "trailing$");
    }

    #[test]
    fn fr_cfg_021_a_whole_reference_is_told_apart_from_a_value_that_merely_contains_one() {
        // FR-CFG-021: `${VAR}` in any field prints as written. A value that is
        // only partly a reference is not that case.
        assert!(is_whole_reference("${SHOP_DB_PASSWORD}"));
        assert!(!is_whole_reference("secret${SUFFIX}"));
        assert!(!is_whole_reference("${A}${B}"));
        assert!(!is_whole_reference("${}"));
        assert!(!is_whole_reference("hunter2"));
    }

    #[test]
    fn fr_conf_049_a_reference_whose_name_no_shell_can_define_is_refused_where_it_expands() {
        for raw in ["${1X}", "a${X-Y}b", "${}"] {
            let condition = expand(raw, &table, "database.shop.user", &file())
                .expect_err("the name is not a variable name");
            assert!(
                matches!(condition, Error::InvalidReferenceName { .. }),
                "{raw}: {condition:?}"
            );
            assert_eq!(condition.exit_code(), 78);
        }
    }

    #[test]
    fn fr_conf_049_the_variable_name_rule_is_the_portable_shell_form() {
        for valid in ["X", "_", "_1", "SHOP_DB_PASSWORD", "a9"] {
            assert!(is_variable_name(valid), "{valid}");
        }
        for invalid in ["", "1X", "X-Y", "X.Y", "X Y", "\u{e1}"] {
            assert!(!is_variable_name(invalid), "{invalid}");
        }
    }

    #[test]
    fn fr_conf_049_the_write_check_reads_the_grammar_the_expansion_reads() {
        assert_eq!(reference_fault("db.example.com"), None);
        assert_eq!(reference_fault("${SHOP_HOST}"), None);
        assert_eq!(
            reference_fault("$${1X}"),
            None,
            "a doubled dollar opens nothing"
        );
        assert_eq!(reference_fault("a$b"), None);
        assert_eq!(reference_fault("${SHOP"), Some(ReferenceFault::Unclosed));
        assert_eq!(
            reference_fault("${OK}-${1X}"),
            Some(ReferenceFault::Name("1X".to_owned()))
        );
    }
}
