//! Files that are not regular files where `tpl` reads one: `.tpl/.cfg`
//! (`FR-PROJ-030`) and a template (`FR-TMPL-033`), rmp `#307`.
//!
//! Every body plants a FIFO, or a symbolic link, where a regular file is
//! expected, and asserts the exit code and the lines the requirement fixes.
//! A FIFO read the way a regular file is read blocks until a writer appears,
//! and no writer ever does here, so a body that returns at all shows that the
//! FIFO was neither opened for a blocking read nor read. No body measures
//! time.

#[path = "support/sandbox.rs"]
mod sandbox;

use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::{Command, Output};

use sandbox::Sandbox;

/// A context document carrying a database with no member.
const CONTEXT: &str = r#"{"schema_version":1,"source":"server","data":{"database":{"name":"shop","charset":"utf8mb4","collation":"utf8mb4_general_ci","server":{"version":"11.4.13-MariaDB","series":"11.4","standing":"supported"},"tables":[],"views":[],"routines":[]}}}"#;

/// Creates a FIFO at `path`, with the system's own `mkfifo`.
fn fifo(path: &Path) {
    let made = Command::new("mkfifo")
        .arg(path)
        .status()
        .expect("mkfifo runs");

    assert!(made.success(), "mkfifo {} failed", path.display());
}

/// What the run wrote to stderr.
fn stderr(printed: &Output) -> String {
    String::from_utf8_lossy(&printed.stderr).into_owned()
}

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

/// Asserts `printed` is a refusal with `code`, nothing on stdout and the four
/// labelled lines, and returns stderr.
fn refused(printed: &Output, code: i32, spelled: &str) -> String {
    let written = stderr(printed);

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

/// A sandbox whose `.tpl` holds no `.cfg` yet, for a body to plant one.
fn bare_project() -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.directory(".tpl");

    sandbox
}

// ------------------------------------------------------ FR-PROJ-030, .cfg ---

#[test]
fn fr_proj_030_a_cfg_that_is_a_fifo_is_78_and_is_not_read() {
    let sandbox = bare_project();
    fifo(&sandbox.path(".tpl/.cfg"));
    let canonical = std::fs::canonicalize(sandbox.path(".tpl"))
        .expect("the project exists")
        .join(".cfg");

    for arguments in [
        &["cfg", "list"][..],
        &["template", "list"][..],
        &["cfg", "set", "core.query_timeout", "60"][..],
    ] {
        let printed = sandbox.run(arguments);
        let written = refused(&printed, 78, &arguments.join(" "));

        assert_eq!(line(&written, "error: "), ".tpl/.cfg is not a regular file");
        assert_eq!(
            line(&written, "cause: "),
            format!(
                "{} is a FIFO; tpl reads its configuration only from a regular file",
                canonical.display()
            )
        );
        let hint = line(&written, "hint:  ");
        assert!(hint.starts_with("replace "), "{hint}");
        assert!(
            hint.ends_with("with a regular file holding the configuration"),
            "{hint}"
        );
        assert!(!hint.contains("rm "), "{hint}");
    }
}

#[test]
fn fr_proj_030_a_cfg_that_is_a_symbolic_link_is_78_whatever_it_points_at() {
    // A link to a regular file the two trust checks would pass, and a link to
    // a device, which must not be read either.
    for target in ["regular", "device"] {
        let sandbox = bare_project();
        let away = Sandbox::new();
        let pointed = if target == "device" {
            Path::new("/dev/null").to_owned()
        } else {
            let file = away.write("elsewhere.cfg", "[core]\n");
            std::fs::set_permissions(&file, std::os::unix::fs::PermissionsExt::from_mode(0o600))
                .expect("the sandbox is writable");
            file
        };
        symlink(&pointed, sandbox.path(".tpl/.cfg")).expect("the sandbox is writable");

        let printed = sandbox.run(&["cfg", "list"]);
        let written = refused(&printed, 78, target);

        assert!(
            line(&written, "cause: ").contains(
                " is a symbolic link; tpl reads its configuration only from a regular file"
            ),
            "{target}: {written}"
        );
    }
}

// ------------------------------------------------------ FR-TMPL-033 ---

/// A project with a valid template `main`, and a FIFO at `header.jinja`.
fn with_fifo_template() -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write(".tpl/templates/main.jinja", "main\n");
    sandbox.write(
        ".tpl/templates/includer.jinja",
        "{% include 'header.jinja' %}\n",
    );
    sandbox.write("context.json", CONTEXT);
    fifo(&sandbox.path(".tpl/templates/header.jinja"));

    sandbox
}

#[test]
fn fr_tmpl_033_a_fifo_template_is_not_listed_and_not_checked() {
    let sandbox = with_fifo_template();

    let listed = sandbox.run(&["template", "list"]);
    assert_eq!(listed.status.code(), Some(0), "{}", stderr(&listed));
    let names = String::from_utf8_lossy(&listed.stdout);
    assert!(!names.contains("header"), "{names}");
    assert!(names.contains("main"), "{names}");

    let checked = sandbox.run(&["template", "check"]);
    assert_eq!(checked.status.code(), Some(0), "{}", stderr(&checked));
    assert!(
        !String::from_utf8_lossy(&checked.stdout).contains("header"),
        "{}",
        String::from_utf8_lossy(&checked.stdout)
    );
}

#[test]
fn fr_tmpl_033_a_fifo_template_named_on_the_command_line_is_66_naming_its_kind() {
    let sandbox = with_fifo_template();

    for arguments in [
        &["render", "header", "--context", "context.json"][..],
        &["template", "show", "header"][..],
        &["template", "check", "header"][..],
        &["template", "path", "header"][..],
    ] {
        let printed = sandbox.run(arguments);
        let written = refused(&printed, 66, &arguments.join(" "));

        assert_eq!(
            line(&written, "error: "),
            "template 'header' does not exist",
            "{arguments:?}"
        );
        assert_eq!(
            line(&written, "cause: "),
            ".tpl/templates/header.jinja exists and is a FIFO; a template is a regular file",
            "{arguments:?}"
        );
    }
}

#[test]
fn fr_tmpl_033_an_include_of_a_fifo_is_65_naming_its_kind() {
    let sandbox = with_fifo_template();

    let printed = sandbox.run(&["render", "includer", "--context", "context.json"]);
    let written = refused(&printed, 65, "render includer");

    assert!(
        line(&written, "cause: ").contains(
            "names 'header.jinja', which exists under the template folder and is a FIFO; a \
             template is a regular file"
        ),
        "{written}"
    );
}

#[test]
fn fr_tmpl_024_a_template_linked_to_a_device_is_refused_and_not_read() {
    // A symbolic link under the root is refused by FR-TMPL-024 before any
    // read, whatever it points at: here `/dev/null`, which a read would take
    // for an empty template.
    let sandbox = Sandbox::new();
    sandbox.project("[core]\n");
    sandbox.write("context.json", CONTEXT);
    sandbox.directory(".tpl/templates");
    symlink("/dev/null", sandbox.path(".tpl/templates/device.jinja"))
        .expect("the sandbox is writable");

    let printed = sandbox.run(&["render", "device", "--context", "context.json"]);
    refused(&printed, 65, "render device");

    let listed = sandbox.run(&["template", "list"]);
    assert_eq!(listed.status.code(), Some(0));
    assert!(!String::from_utf8_lossy(&listed.stdout).contains("device"));
}

#[test]
fn fr_proj_030_a_cfg_that_is_a_socket_is_78_naming_the_kind() {
    // A socket refuses the open itself (EOPNOTSUPP or ENXIO) before its type
    // can be read from a descriptor; it is still named for what it is.
    let sandbox = bare_project();
    let path = sandbox.path(".tpl/.cfg");
    let _listening = std::os::unix::net::UnixListener::bind(&path).expect("the socket binds");

    let printed = sandbox.run(&["cfg", "list"]);
    let written = refused(&printed, 78, "cfg list over a socket");

    assert_eq!(line(&written, "error: "), ".tpl/.cfg is not a regular file");
    assert!(
        line(&written, "cause: ")
            .contains(" is a socket; tpl reads its configuration only from a regular file"),
        "{written}"
    );
}
