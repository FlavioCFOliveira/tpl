//! `tpl::run_from` drives the whole invocation in process, and answers the
//! exit code the binary answers for the same argument vector (rmp `#257`).

#[path = "support/sandbox.rs"]
mod sandbox;

use sandbox::Sandbox;

/// The exit code `main.rs` derives from what the library returned.
fn code(result: Result<(), tpl::Error>) -> i32 {
    result.map_or_else(|error| i32::from(error.exit_code()), |()| 0)
}

#[test]
fn run_from_answers_the_exit_code_the_binary_answers() {
    // Neither vector needs a project: `version` is exempt from discovery per
    // FR-PROJ-025, and an unknown flag is refused at parsing, step 1 of
    // FR-ERR-006.
    let sandbox = Sandbox::new();

    for (argv, expected) in [(&["tpl", "version"][..], 0), (&["tpl", "--nope"][..], 64)] {
        let binary = sandbox.run(&argv[1..]);

        assert_eq!(binary.status.code(), Some(expected), "{argv:?}");
        assert_eq!(
            code(tpl::run_from(argv.iter().copied())),
            expected,
            "{argv:?}"
        );
    }
}

#[test]
fn run_from_takes_owned_strings_as_well_as_borrowed_ones() {
    let argv: Vec<String> = vec!["tpl".to_owned(), "--nope".to_owned()];

    assert_eq!(code(tpl::run_from(argv)), 64);
}
