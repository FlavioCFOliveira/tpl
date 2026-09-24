//! The caller's own command, written back with one correction.
//!
//! A `hint` that corrects a command the caller ran has to reproduce the whole
//! of it: a global flag, a `--context` document, a `--set` value and every
//! other flag given, changing only the faulty token (finding T-01 of the third
//! re-audit of rmp `#263`). A hint that wrote the command path alone and the
//! corrected token read `core.database` where the caller had named `-d shop`,
//! or the server where the caller had named a `--context` document, and could
//! exit `0` with output from the wrong source when copied verbatim.
//!
//! The argument vector is recorded once, by [`record`], before it is parsed.
//! [`restated`] walks it against the command tree, classifying each token as
//! a literal of `FR-ERR-022` — a command, an alias, a flag — or as a value, and
//! tests every value by the character set that governs it. A value its set
//! refuses is written as a placeholder, and [`Restated::replacing`] says in
//! words what each placeholder stands for, so nothing the caller gave is
//! dropped in silence and nothing refused reaches the line.
//!
//! Where nothing was recorded — a unit test that renders an [`Error`] it built
//! itself — [`restated`] answers [`None`] and the caller of this module keeps
//! the command path it already carries.
//!
//! [`Error`]: crate::error::Error

use std::ffi::OsString;
use std::sync::OnceLock;

use super::hint::{admits, admits_key, admits_template};

/// The argument vector of this process, `argv[0]` included, as it was parsed.
static RECORDED: OnceLock<Vec<OsString>> = OnceLock::new();

/// Records the argument vector the process is about to parse.
///
/// The first call wins; a second is ignored, because one process parses one
/// vector.
pub(crate) fn record(argv: &[OsString]) {
    let _ = RECORDED.set(argv.to_vec());
}

/// A correction to apply to the recorded command.
#[derive(Debug, Clone, Copy)]
pub(super) enum Edit<'a> {
    /// Write `to` in place of the value of the argument whose identifier is
    /// one of `ids` — a flag's value or a positional operand.
    Value {
        /// The clap identifiers the value may be carried under.
        ids: &'a [&'a str],
        /// What is written in its place.
        to: Replacement<'a>,
    },
    /// Make the format JSON: the value of `--format` becomes `json`, or
    /// `--format json` is written before the first `--pretty`.
    FormatJson,
    /// Append these literal words at the end of the command.
    Append(&'a [&'a str]),
}

/// What an [`Edit::Value`] writes.
#[derive(Debug, Clone, Copy)]
pub(super) enum Replacement<'a> {
    /// A value the hint has already tested and admits.
    Literal(&'a str),
    /// A placeholder, and the words that say what it stands for.
    Placeholder {
        /// The placeholder, angle brackets included.
        text: &'a str,
        /// What the caller writes in its place.
        meaning: &'a str,
    },
}

/// The recorded command with the correction applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Restated {
    /// The whole command, beginning with `tpl`.
    pub(super) command: String,
    /// Each placeholder written into it, with what it stands for.
    pub(super) placeholders: Vec<(String, String)>,
}

impl Restated {
    /// The words that say what each placeholder stands for, as a clause that
    /// follows the command: `; replace <context> with the value you gave
    /// --context`. Empty where the command carries no placeholder.
    pub(super) fn replacing(&self) -> String {
        let mut clause = String::new();
        for (index, (placeholder, meaning)) in self.placeholders.iter().enumerate() {
            clause.push_str(match index {
                0 => "; replace ",
                _ if index + 1 == self.placeholders.len() => ", and ",
                _ => ", ",
            });
            clause.push_str(placeholder);
            clause.push_str(" with ");
            clause.push_str(meaning);
        }
        clause
    }
}

/// The recorded command with `edits` applied, or [`None`] where no vector was
/// recorded, a token is not valid UTF-8 where a literal was expected, or an
/// edit found nothing to act on.
pub(super) fn restated(edits: &[Edit<'_>]) -> Option<Restated> {
    let argv = RECORDED.get()?;
    let words: Vec<Option<&str>> = argv.iter().skip(1).map(|word| word.to_str()).collect();
    restate(&crate::cli::tree(), &words, edits)
}

/// The `--tpl-dir` the caller gave, as the words to write after `tpl` in a
/// command that must run against the same project, or [`None`] where none was
/// given. A path the set of `FR-ERR-041` refuses is a placeholder, and the
/// second member says what it stands for.
pub(super) fn project_flag() -> Option<(String, Option<(String, String)>)> {
    let argv = RECORDED.get()?;
    let mut words = argv.iter().skip(1);
    while let Some(word) = words.next() {
        let text = word.to_str();
        if text == Some("--") {
            return None;
        }
        let value = match text.and_then(|text| text.strip_prefix("--tpl-dir")) {
            Some("") => words.next().and_then(|value| value.to_str()),
            Some(attached) if attached.starts_with('=') => Some(&attached[1..]),
            _ => continue,
        };
        return Some(match value {
            Some(path) if admits_template(path) => (format!("--tpl-dir {path}"), None),
            _ => (
                "--tpl-dir <tpl-dir>".to_owned(),
                Some((
                    "<tpl-dir>".to_owned(),
                    "the value you gave --tpl-dir".to_owned(),
                )),
            ),
        });
    }
    None
}

/// One word of the restated command.
#[derive(Debug)]
struct Piece {
    /// The text written, including a `--flag=` it is glued to.
    text: String,
    /// The identifier of the argument this word is the value of, if any.
    value_of: Option<String>,
    /// The flag spelling the word is glued to (`--flag=`), if any.
    glued: Option<String>,
    /// The placeholder the word carries, and what it stands for.
    placeholder: Option<(String, String)>,
}

impl Piece {
    fn literal(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            value_of: None,
            glued: None,
            placeholder: None,
        }
    }
}

/// The walk behind [`restated`], over a vector without `argv[0]`.
fn restate(tree: &clap::Command, words: &[Option<&str>], edits: &[Edit<'_>]) -> Option<Restated> {
    let mut path: Vec<&clap::Command> = vec![tree];
    let mut pieces: Vec<Piece> = Vec::with_capacity(words.len() + 2);
    let mut positional = 0_usize;
    let mut operands_only = false;
    let mut words = words.iter().copied();

    while let Some(word) = words.next() {
        let node = *path.last()?;
        let Some(text) = word else {
            // Not UTF-8: it can only be a value, and it is never written.
            pieces.push(operand(node, &mut positional, None));
            continue;
        };

        if !operands_only && text == "--" {
            operands_only = true;
            pieces.push(Piece::literal(text));
            continue;
        }

        if !operands_only && text.len() > 2 && text.starts_with("--") {
            let (name, attached) = match text[2..].split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (&text[2..], None),
            };
            let argument = declared(&path, |argument| argument.get_long() == Some(name))?;
            let flag = format!("--{name}");
            if !takes_value(argument) {
                pieces.push(Piece::literal(&flag));
            } else if let Some(value) = attached {
                let mut piece = valued(argument, Some(value));
                piece.glued = Some(flag);
                pieces.push(piece);
            } else {
                pieces.push(Piece::literal(&flag));
                pieces.push(valued(argument, words.next().flatten()));
            }
            continue;
        }

        if !operands_only && text.len() > 1 && text.starts_with('-') {
            let cluster = &text[1..];
            let mut plain = String::new();
            for (at, short) in cluster.char_indices() {
                let argument = declared(&path, |argument| argument.get_short() == Some(short))?;
                if !takes_value(argument) {
                    plain.push(short);
                    continue;
                }
                if !plain.is_empty() {
                    pieces.push(Piece::literal(&format!("-{plain}")));
                    plain.clear();
                }
                pieces.push(Piece::literal(&format!("-{short}")));
                let rest = &cluster[at + short.len_utf8()..];
                let rest = rest.strip_prefix('=').unwrap_or(rest);
                let value = if rest.is_empty() {
                    words.next().flatten()
                } else {
                    Some(rest)
                };
                pieces.push(valued(argument, value));
                break;
            }
            if !plain.is_empty() {
                pieces.push(Piece::literal(&format!("-{plain}")));
            }
            continue;
        }

        if !operands_only
            && positional == 0
            && let Some(child) = node.find_subcommand(text)
        {
            pieces.push(Piece::literal(text));
            path.push(child);
            continue;
        }

        pieces.push(operand(node, &mut positional, Some(text)));
    }

    for edit in edits {
        apply(&mut pieces, *edit)?;
    }

    let mut command = String::from("tpl");
    let mut placeholders: Vec<(String, String)> = Vec::new();
    for piece in &pieces {
        command.push(' ');
        if let Some(flag) = &piece.glued {
            command.push_str(flag);
            command.push('=');
        }
        command.push_str(&piece.text);
        if let Some(placeholder) = &piece.placeholder
            && !placeholders.contains(placeholder)
        {
            placeholders.push(placeholder.clone());
        }
    }

    Some(Restated {
        command,
        placeholders,
    })
}

/// Applies one edit, or answers [`None`] where it found nothing to act on.
fn apply(pieces: &mut Vec<Piece>, edit: Edit<'_>) -> Option<()> {
    match edit {
        Edit::Value { ids, to } => {
            let piece = pieces.iter_mut().rev().find(|piece| {
                piece
                    .value_of
                    .as_deref()
                    .is_some_and(|id| ids.contains(&id))
            })?;
            match to {
                Replacement::Literal(text) => {
                    piece.text = text.to_owned();
                    piece.placeholder = None;
                }
                Replacement::Placeholder { text, meaning } => {
                    piece.text = text.to_owned();
                    piece.placeholder = Some((text.to_owned(), meaning.to_owned()));
                }
            }
        }
        Edit::FormatJson => {
            let mut found = false;
            for piece in pieces.iter_mut() {
                if piece.value_of.as_deref() == Some("format") {
                    piece.text = "json".to_owned();
                    piece.placeholder = None;
                    found = true;
                }
            }
            if !found {
                let at = pieces.iter().position(|piece| piece.text == "--pretty")?;
                pieces.insert(at, Piece::literal("json"));
                pieces.insert(at, Piece::literal("--format"));
            }
        }
        Edit::Append(words) => pieces.extend(words.iter().map(|word| Piece::literal(word))),
    }
    Some(())
}

/// The flag of `path` — the node reached, or one of its ancestors, whose
/// global flags reach it — that `matches` selects.
fn declared<'t>(
    path: &[&'t clap::Command],
    matches: impl Fn(&clap::Arg) -> bool,
) -> Option<&'t clap::Arg> {
    path.iter()
        .rev()
        .find_map(|node| node.get_arguments().find(|argument| matches(argument)))
}

/// Whether a flag takes a value, rather than being counted or set.
fn takes_value(argument: &clap::Arg) -> bool {
    argument.get_action().takes_values()
}

/// The next positional operand of `node`.
fn operand(node: &clap::Command, positional: &mut usize, text: Option<&str>) -> Piece {
    let positionals: Vec<&clap::Arg> = node.get_positionals().collect();
    let argument = positionals
        .get(*positional)
        .or_else(|| positionals.last())
        .copied();
    *positional += 1;
    match argument {
        Some(argument) => valued(argument, text),
        None => placeholder(None, "<value>".to_owned(), "the value you gave".to_owned()),
    }
}

/// The value `text` of `argument`, tested by the set that governs it.
fn valued(argument: &clap::Arg, text: Option<&str>) -> Piece {
    let id = argument.get_id().as_str();
    let name = argument.get_long().map_or_else(
        || {
            argument
                .get_value_names()
                .and_then(|names| names.first())
                .map_or_else(|| id.to_owned(), |name| name.to_lowercase())
        },
        str::to_owned,
    );
    let meaning = if argument.is_positional() {
        format!("the {name} you gave")
    } else {
        format!("the value you gave --{name}")
    };
    let refused = || placeholder(Some(id), format!("<{name}>"), meaning.clone());

    let Some(text) = text else {
        return refused();
    };

    let admitted = match id {
        // BR-ERR-003 bars a DSN from every message, and a value of `cfg set`
        // may be a password; neither is written back whatever it holds.
        "dsn" | "value" => false,
        // FR-ERR-041 governs a template name and a path of the project.
        "template" | "tpl_dir" | "context" => admits_template(text),
        // A key of FR-CONF-002 is a literal, tested segment by segment.
        "key" => admits_key(text),
        // FR-RND-012 confines the key; the value is a value like any other.
        "set" => {
            return match text.split_once('=') {
                Some((key, value)) if admits(key) && admits(value) => Piece {
                    text: text.to_owned(),
                    value_of: Some(id.to_owned()),
                    glued: None,
                    placeholder: None,
                },
                Some((key, _)) if admits(key) => Piece {
                    text: format!("{key}=<{key}>"),
                    value_of: Some(id.to_owned()),
                    glued: None,
                    placeholder: Some((
                        format!("<{key}>"),
                        format!("the value you gave --set {key}"),
                    )),
                },
                _ => refused(),
            };
        }
        _ => admits(text),
    };

    if admitted {
        Piece {
            text: text.to_owned(),
            value_of: Some(id.to_owned()),
            glued: None,
            placeholder: None,
        }
    } else {
        refused()
    }
}

fn placeholder(id: Option<&str>, text: String, meaning: String) -> Piece {
    Piece {
        text: text.clone(),
        value_of: id.map(str::to_owned),
        glued: None,
        placeholder: Some((text, meaning)),
    }
}

#[cfg(test)]
mod tests {
    use super::{Edit, Replacement, restate};

    fn restated(words: &[&str], edits: &[Edit<'_>]) -> (String, String) {
        let words: Vec<Option<&str>> = words.iter().copied().map(Some).collect();
        let restated = restate(&crate::cli::tree(), &words, edits).expect("restates");
        let replacing = restated.replacing();
        (restated.command, replacing)
    }

    const FUNCTION_F: Edit<'static> = Edit::Value {
        ids: &["routine", "name"],
        to: Replacement::Literal("function:f"),
    };

    #[test]
    fn t_01_keeps_every_flag_and_changes_the_faulty_token() {
        let (command, replacing) = restated(
            &[
                "-d",
                "shop",
                "render",
                "rust/x",
                "--routine",
                "FUNCTION:f",
                "--set",
                "a=b",
            ],
            &[FUNCTION_F],
        );
        assert_eq!(
            command,
            "tpl -d shop render rust/x --routine function:f --set a=b"
        );
        assert_eq!(replacing, "");
    }

    #[test]
    fn t_01_keeps_the_context_document_and_the_project() {
        let (command, _) = restated(
            &[
                "--tpl-dir",
                ".tpl",
                "render",
                "example",
                "--context",
                "rt.json",
                "--routine=r",
            ],
            &[FUNCTION_F],
        );
        assert_eq!(
            command,
            "tpl --tpl-dir .tpl render example --context rt.json --routine=function:f"
        );
    }

    #[test]
    fn t_01_positional_routine_is_the_one_replaced() {
        let (command, _) = restated(
            &["-d", "r", "schema", "routine", "Function:x", "--direct"],
            &[FUNCTION_F],
        );
        assert_eq!(command, "tpl -d r schema routine function:f --direct");
    }

    #[test]
    fn t_01_refused_values_become_named_placeholders() {
        let (command, replacing) = restated(
            &[
                "-vv",
                "-dshop",
                "schema",
                "tables",
                "--pattern",
                "ord%",
                "--pretty",
            ],
            &[Edit::FormatJson],
        );
        assert_eq!(
            command,
            "tpl -vv -d shop schema tables --pattern <pattern> --format json --pretty"
        );
        assert_eq!(
            replacing,
            "; replace <pattern> with the value you gave --pattern"
        );
    }

    #[test]
    fn t_01_format_text_becomes_json() {
        let (command, _) = restated(
            &["schema", "table", "orders", "--format", "text", "--pretty"],
            &[Edit::FormatJson],
        );
        assert_eq!(command, "tpl schema table orders --format json --pretty");
    }

    #[test]
    fn t_01_set_values_are_named_by_key() {
        let (command, replacing) = restated(
            &[
                "render",
                "t",
                "--routine",
                "x",
                "--set",
                "title=a b",
                "--context",
                "/a b/c.json",
            ],
            &[FUNCTION_F],
        );
        assert_eq!(
            command,
            "tpl render t --routine function:f --set title=<title> --context <context>"
        );
        assert_eq!(
            replacing,
            "; replace <title> with the value you gave --set title, and <context> with the value \
             you gave --context"
        );
    }

    #[test]
    fn t_01_a_dsn_and_a_cfg_value_are_never_written_back() {
        let (command, replacing) = restated(
            &[
                "cfg",
                "database",
                "update",
                "f",
                "--dsn",
                "mysql://u:p@h/d",
                "--tls",
                "required",
            ],
            &[],
        );
        assert_eq!(
            command,
            "tpl cfg database update f --dsn <dsn> --tls required"
        );
        assert_eq!(replacing, "; replace <dsn> with the value you gave --dsn");

        let (command, _) = restated(&["cfg", "set", "database.a.password", "secret"], &[]);
        assert_eq!(command, "tpl cfg set database.a.password <value>");
    }

    #[test]
    fn t_01_an_edit_with_nothing_to_act_on_restates_nothing() {
        let words = [Some("version")];
        assert!(restate(&crate::cli::tree(), &words, &[FUNCTION_F]).is_none());
    }
}
