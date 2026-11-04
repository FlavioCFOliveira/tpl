#!/usr/bin/env python3

"""Re-derive the document layer of the knowledge graph from the tree.

One command turns the tracked files the model covers into the nodes and edges
the graph holds for documents: ``File``, ``Directory``, ``Requirement``,
``Decision``, ``Baseline`` and ``Crate``, joined by ``CONTAINS``, ``DEFINES``,
``CITES``, ``REFERENCES``, ``TARGETS``, ``RELATED_TO``, ``DECLARES`` and
``DEPENDS_ON``.

It emits; it never writes. Nothing here opens a connection to the graph, issues
a Cypher write, or touches ``~/.roadmaps/``. The output goes to standard output
and the ``knowledge-authority`` skill, which is the graph's only writer, decides
what to do with it.

The vocabulary — every label, every predicate, every property and what each one
means — is fixed by ``knowledge-model.md`` at the repository root. This file
speaks that vocabulary and does not extend it.

Usage:

    scripts/knowledge/doc-layer.py                 # Cypher on stdout
    scripts/knowledge/doc-layer.py --format json   # the same records as JSON
    scripts/knowledge/doc-layer.py --summary       # counts per label and predicate
    scripts/knowledge/doc-layer.py --verify        # re-derive and re-read every record

``--verify`` is the check the layer is answerable for. It re-derives the whole
layer with a second implementation that shares no code with the scan, compares
the two record by record, and reopens each record at the ``file:line`` it
claims. The exit code is ``0`` when everything matched and ``1`` when anything
did not.

Standard library only, and no configuration: the tree is the input.
"""

from __future__ import annotations

import argparse
import json
import posixpath
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

# ------------------------------------------------------------ the coverage ---

#: The tracked files the model covers, and the `File.kind` each one takes.
#: `knowledge-model.md` closes the set of kinds; this table says where each
#: kind lives. A path takes the kind of the first entry it matches in full, and
#: a path no entry matches is not covered. The first element of each entry is
#: the pathspec the dirty-tree check reads.
COVERAGE: tuple[tuple[str, re.Pattern[str], str], ...] = (
    ("specification", re.compile(r"specification/.+\.md"), "spec"),
    ("docs/spec-technical", re.compile(r"docs/spec-technical/.+\.md"), "techspec"),
    ("docs/adr", re.compile(r"docs/adr/.+\.md"), "adr"),
    ("CLAUDE.md", re.compile(r"CLAUDE\.md"), "coordination"),
    ("README.md", re.compile(r"README\.md"), "coordination"),
    ("BENCHMARKS.md", re.compile(r"BENCHMARKS\.md"), "benchmark"),
    ("benches", re.compile(r"benches/.+"), "benchmark"),
    ("scripts/mariadb", re.compile(r"scripts/mariadb/.+"), "fixture"),
    ("scripts/sqlite", re.compile(r"scripts/sqlite/.+"), "fixture"),
    ("src", re.compile(r"src/.+\.rs"), "source"),
    ("tests", re.compile(r"tests/.+\.rs"), "test"),
    ("templates", re.compile(r"templates/.+"), "template"),
)

#: Suffixes whose contents are never read. Key and certificate material holds
#: no identifier, and `knowledge-model.md` names the private key of the fixture
#: as the file the no-content rule exists for: the scan gives such a file a
#: node and opens nothing else of it.
UNREAD_SUFFIXES = (".pem",)

#: Where each family of definition is read from.
SPECIFICATION = "specification/"
REGISTER_DIRECTORY = "docs/spec-technical/"
RECORD_DIRECTORY = "docs/adr/"
BENCHMARKS = "BENCHMARKS.md"
MANIFEST = "Cargo.toml"

#: The document that states why each shipped crate earns its place, and the
#: column it states it in. `knowledge-model.md` takes `Crate.justification`
#: from a document that states one and never from the crate's purpose, which
#: is the table's *What it does here* column and is not read.
JUSTIFICATION_FILE = "docs/spec-technical/technology-stack.md"
JUSTIFICATION_COLUMN = "Chosen because"

# ------------------------------------------------------------- the grammar ---

#: Every identifier the two corpora issue. `FR`, `NFR` and `BR` carry a middle
#: segment; the rest are bare. A placeholder such as `FR-X`, `OD-nn` or
#: `ADR-NNN` has no digits where the number goes and is not an identifier.
IDENTIFIER = re.compile(
    r"\b(?:(?:FR|NFR|BR)-[A-Z][A-Z0-9]*-\d+|(?:UC|OQ|DIV|WL|OD|ADR)-\d+)\b"
)

#: The families whose identifiers carry a middle segment, and the families
#: that are decisions rather than requirements.
AREA_FAMILIES = frozenset({"FR", "NFR", "BR"})
DECISION_FAMILIES = frozenset({"OD", "ADR"})

#: The families whose source supplies a `title`, per `knowledge-model.md`.
TITLED_FAMILIES = frozenset({"UC", "OQ", "DIV"})

#: A requirement bullet: `FR`, `NFR` and `BR` close the bold with a colon,
#: `WL` with an em dash.
BULLET = re.compile(
    r"^- \*\*((?:FR|NFR|BR)-[A-Z][A-Z0-9]*-\d+|WL-\d+)\*\*(?::|\s+—)"
)

#: The `##` headings that open a whole-section definition block.
USE_CASE = re.compile(r"^## (UC-\d+) — (.+?)\s*$")
DIVERGENCE = re.compile(r"^## (DIV-\d+)\s*$")
OPEN_DECISION = re.compile(r"^## (OD-\d+) — (.+?)\s*$")

#: Markdown structure.
HEADING = re.compile(r"^(#{1,6})(?:\s|$)")
FENCE = re.compile(r"^\s*(`{3,}|~{3,})")
FRONT_KEY = re.compile(r"^([A-Za-z][A-Za-z0-9_-]*):\s*(.*?)\s*$")
CODE_SPAN = re.compile(r"`([^`]+)`")

#: The structured field a divergence record states its target in, and the code
#: span that opens its value.
TARGET_FIELD = re.compile(r"^\*\*Target\*\*:\s*(.*)$")
LEADING_SPAN = re.compile(r"`([^`]+)`")

#: A baseline entry of `BENCHMARKS.md`, and the clause naming its targets.
BASELINE = re.compile(r"^## (\d{4}-\d{2}-\d{2}) — (.+?)\s*$")
TARGET_OF_RECORD = re.compile(r"Target of record:\s*(.*?)\.(?=\s|\*|$)")

#: The header cells that identify the three tables this scan reads.
QUESTION_TABLE = ("Id", "Question")
DIVERGENCE_INDEX = ("Id", "Target", "Kind", "Status", "Subject")

#: The ceiling on one emitted statement.
STATEMENT_LIMIT = 1_000_000


class ScanError(Exception):
    """The tree holds something the model's rules cannot place."""


# ------------------------------------------------------------- the records ---


@dataclass
class Node:
    """One node of the document layer, with the properties its label declares."""

    label: str
    key: str
    properties: dict[str, object]


@dataclass
class Edge:
    """One edge, named by its endpoints' labels and identity properties."""

    predicate: str
    source_label: str
    source_key: str
    target_label: str
    target_key: str
    properties: dict[str, object] = field(default_factory=dict)


@dataclass
class Layer:
    """Everything one scan of the tree produced, and what it could not place."""

    nodes: list[Node] = field(default_factory=list)
    edges: list[Edge] = field(default_factory=list)
    #: `path:line ID` for an occurrence that names no defined identifier.
    dangling: list[str] = field(default_factory=list)
    #: `path:line OWNER -> ID` for a decision named inside a requirement's
    #: block: the model defines no `(Requirement)-[:CITES]->(Decision)`.
    unplaced: list[str] = field(default_factory=list)


@dataclass
class Definition:
    """The single site that defines one identifier."""

    ident: str
    path: str
    line: int
    title: str | None


@dataclass
class Document:
    """One covered file, read once."""

    path: str
    kind: str
    lines: list[str]
    front: dict[str, tuple[str, int]]
    front_end: int
    owner: list[str | None] = field(default_factory=list)


# ------------------------------------------------------------- the naming ---


def label_of(ident: str) -> str:
    """The label an identifier's node carries."""
    family = ident.split("-", 1)[0]
    return "Decision" if family in DECISION_FAMILIES else "Requirement"


def identifier_properties(ident: str, title: str | None) -> dict[str, object]:
    """`family`, `area`, `ordinal` and `title`, each only where the model has it."""
    parts = ident.split("-")
    family = parts[0]
    properties: dict[str, object] = {"family": family, "ordinal": int(parts[-1])}
    if family in AREA_FAMILIES:
        properties["area"] = parts[1]
    if title is not None:
        properties["title"] = title
    return properties


def kind_of(path: str) -> str | None:
    """The `File.kind` of a tracked path, or `None` where the model covers none."""
    for _, pattern, kind in COVERAGE:
        if pattern.fullmatch(path):
            return kind
    return None


def slug_of(subject: str) -> str:
    """The slug of a baseline subject: lower case, every other run a hyphen."""
    return re.sub(r"[^a-z0-9]+", "-", subject.replace("`", "").lower()).strip("-")


# -------------------------------------------------------------- the reading ---


def tracked_files(root: Path) -> list[str]:
    """Every path git tracks, repository-relative, sorted."""
    listing = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=root,
        check=True,
        capture_output=True,
    ).stdout.decode("utf-8")
    return sorted(path for path in listing.split("\0") if path)


def front_matter(lines: list[str]) -> tuple[dict[str, tuple[str, int]], int]:
    """A Markdown file's front-matter keys, each with its value and line.

    Returns the keys and the number of the closing `---` line, or an empty map
    and `0` where the file opens with no front matter.
    """
    if not lines or lines[0] != "---":
        return {}, 0
    keys: dict[str, tuple[str, int]] = {}
    for index in range(1, len(lines)):
        if lines[index] == "---":
            return keys, index + 1
        match = FRONT_KEY.match(lines[index])
        if match is not None:
            keys[match.group(1)] = (match.group(2), index + 1)
    raise ScanError("front matter opened on line 1 and never closed")


def fenced_lines(lines: list[str]) -> list[bool]:
    """Whether each line sits inside a fenced code block.

    The opening fence is not inside — it is read as an ordinary line, so a fence
    written at column 0 ends a bullet — and the closing fence is.
    """
    fenced = [False] * len(lines)
    marker = ""
    for index, line in enumerate(lines):
        match = FENCE.match(line)
        if not marker:
            if match is not None:
                marker = match.group(1)
            continue
        fenced[index] = True
        stripped = line.strip()
        if (
            match is not None
            and stripped == match.group(1)
            and stripped[0] == marker[0]
            and len(stripped) >= len(marker)
        ):
            marker = ""
    return fenced


def cells(row: str) -> list[str]:
    """The cells of one Markdown table row, honouring code spans and `\\|`."""
    out: list[str] = []
    current: list[str] = []
    in_code = False
    index = 0
    body = row.strip()
    while index < len(body):
        character = body[index]
        if character == "\\" and index + 1 < len(body):
            current.append(body[index : index + 2])
            index += 2
            continue
        if character == "`":
            in_code = not in_code
        if character == "|" and not in_code:
            out.append("".join(current).strip())
            current = []
        else:
            current.append(character)
        index += 1
    out.append("".join(current).strip())
    if out and out[0] == "":
        out = out[1:]
    if out and out[-1] == "":
        out = out[:-1]
    return out


def is_separator(row: list[str]) -> bool:
    """Whether a table row is the `|---|` line under the header."""
    return bool(row) and all(re.fullmatch(r":?-{3,}:?", cell) for cell in row)


def read_document(root: Path, path: str, kind: str) -> Document:
    """One covered file, its lines and its front matter."""
    if path.endswith(UNREAD_SUFFIXES):
        return Document(path, kind, [], {}, 0)
    try:
        text = (root / path).read_text(encoding="utf-8")
    except UnicodeDecodeError as error:
        raise ScanError(f"{path}: not UTF-8 ({error})") from error
    lines = text.splitlines()
    front: dict[str, tuple[str, int]] = {}
    front_end = 0
    if path.endswith(".md"):
        try:
            front, front_end = front_matter(lines)
        except ScanError as error:
            raise ScanError(f"{path}: {error}") from error
    return Document(path, kind, lines, front, front_end)


# ----------------------------------------------------- the definition blocks ---


class Blocks:
    """Every definition site, and the owner of every line that sits in a block.

    A block is the bullet and its indented continuation for `FR`, `NFR`, `BR`
    and `WL`; the whole `##` section for `UC`, `DIV` and `OD`; the single table
    row for `OQ`; and, for an `ADR`, the whole file below its front matter.
    """

    def __init__(self) -> None:
        self.definitions: dict[str, Definition] = {}
        self.divergence_titles: dict[str, tuple[str, str, int]] = {}
        self.targets: dict[str, tuple[str, int]] = {}

    def define(self, ident: str, path: str, line: int, title: str | None) -> None:
        """Records one definition site; a second site for one identifier fails."""
        if ident in self.definitions:
            first = self.definitions[ident]
            raise ScanError(
                f"{ident} is defined twice: {first.path}:{first.line} "
                f"and {path}:{line}"
            )
        self.definitions[ident] = Definition(ident, path, line, title)

    def read(self, document: Document) -> None:
        """Assigns every line of one document its owner, recording definitions."""
        document.owner = [None] * len(document.lines)
        path = document.path
        if path.startswith(SPECIFICATION):
            self.structured(document, (USE_CASE, DIVERGENCE), specification=True)
        elif path.startswith(REGISTER_DIRECTORY):
            self.structured(document, (OPEN_DECISION,), specification=False)
        elif path.startswith(RECORD_DIRECTORY):
            self.record(document)

    def record(self, document: Document) -> None:
        """An architecture decision record: one identifier owns the whole body."""
        if "id" not in document.front:
            return
        ident, line = document.front["id"]
        if not re.fullmatch(r"ADR-\d+", ident):
            raise ScanError(f"{document.path}:{line}: front-matter id {ident!r}")
        if "title" not in document.front:
            raise ScanError(f"{document.path}: {ident} has no front-matter title")
        self.define(ident, document.path, line, document.front["title"][0])
        for index in range(document.front_end, len(document.lines)):
            document.owner[index] = ident

    def structured(
        self,
        document: Document,
        sections: tuple[re.Pattern[str], ...],
        specification: bool,
    ) -> None:
        """A specification module or the decision register, read line by line.

        `sections` are the headings that open a whole-section block. The bullet
        and table-row definitions, and the divergence index, are read only in
        the functional specification, which is the only corpus that issues them.
        """
        path = document.path
        fenced = fenced_lines(document.lines)
        section: str | None = None
        bullet: str | None = None
        header: list[str] | None = None

        for index, line in enumerate(document.lines):
            number = index + 1
            if index < document.front_end:
                continue
            if fenced[index]:
                document.owner[index] = bullet or section
                continue

            if line.strip() and not line[0].isspace():
                bullet = None

            heading = HEADING.match(line)
            if heading is not None and len(heading.group(1)) <= 2:
                section = None
                for pattern in sections:
                    match = pattern.match(line)
                    if match is not None:
                        section = match.group(1)
                        title = match.group(2) if pattern.groups == 2 else None
                        self.define(section, path, number, title)

            row_owner: str | None = None
            if line.startswith("|"):
                row = cells(line)
                if header is None:
                    header = row
                elif specification and not is_separator(row):
                    row_owner = self.row(header, row, path, number)
            else:
                header = None

            if specification:
                match = BULLET.match(line)
                if match is not None:
                    if section is not None:
                        raise ScanError(
                            f"{path}:{number}: {match.group(1)} is defined "
                            f"inside the block of {section}"
                        )
                    bullet = match.group(1)
                    self.define(bullet, path, number, None)

            if row_owner is not None and (bullet or section) is not None:
                raise ScanError(
                    f"{path}:{number}: {row_owner} is defined inside the block "
                    f"of {bullet or section}"
                )
            owner = row_owner or bullet or section
            document.owner[index] = owner

            if owner is not None and owner.startswith("DIV-"):
                target = TARGET_FIELD.match(line)
                if target is not None and owner not in self.targets:
                    self.targets[owner] = (target.group(1), number)

    def row(
        self, header: list[str], row: list[str], path: str, number: int
    ) -> str | None:
        """Reads one table row: an open question's definition, or a title."""
        if tuple(header[: len(QUESTION_TABLE)]) == QUESTION_TABLE:
            if row and re.fullmatch(r"OQ-\d+", row[0]):
                if len(row) < 2:
                    raise ScanError(f"{path}:{number}: {row[0]} has no question")
                self.define(row[0], path, number, row[1])
                return row[0]
        if tuple(header) == DIVERGENCE_INDEX and row:
            match = re.search(r"DIV-\d+", row[0])
            if match is not None:
                ident = match.group(0)
                if ident in self.divergence_titles:
                    raise ScanError(f"{path}:{number}: {ident} is indexed twice")
                subject = row[DIVERGENCE_INDEX.index("Subject")]
                self.divergence_titles[ident] = (subject, path, number)
        return None


# ------------------------------------------------------------ the assembly ---


def scan_tree(root: Path) -> Layer:
    """Every node and edge the tree yields, and what could not be placed."""
    layer = Layer()

    covered: list[tuple[str, str]] = []
    for path in tracked_files(root):
        kind = kind_of(path)
        if kind is None:
            continue
        if not (root / path).is_file():
            raise ScanError(f"{path} is tracked but is not a file in the tree")
        covered.append((path, kind))
    files = {path for path, _ in covered}

    documents = [read_document(root, path, kind) for path, kind in covered]

    directories(layer, covered)

    for document in documents:
        properties: dict[str, object] = {
            "name": posixpath.basename(document.path),
            "kind": document.kind,
        }
        for key in ("title", "status"):
            if key in document.front:
                properties[key] = document.front[key][0]
        layer.nodes.append(Node("File", document.path, properties))

    blocks = Blocks()
    for document in documents:
        blocks.read(document)

    for ident, definition in sorted(blocks.definitions.items()):
        title = definition.title
        if ident.startswith("DIV-"):
            title = blocks.divergence_titles.get(ident, (None, "", 0))[0]
        if label_of(ident) == "Requirement" and ident.split("-")[0] not in (
            TITLED_FAMILIES
        ):
            title = None
        layer.nodes.append(
            Node(label_of(ident), ident, identifier_properties(ident, title))
        )
        layer.edges.append(
            Edge(
                "DEFINES",
                "File",
                definition.path,
                label_of(ident),
                ident,
                {"line": definition.line},
            )
        )

    occurrences(layer, documents, blocks)
    targets(layer, blocks, covered)
    related(layer, documents, files)
    baselines(layer, root, files)
    crates(layer, root)

    order = {label: rank for rank, label in enumerate(NODE_ORDER)}
    layer.nodes.sort(key=lambda node: (order[node.label], node.key))
    order = {predicate: rank for rank, predicate in enumerate(EDGE_ORDER)}
    layer.edges.sort(
        key=lambda edge: (
            order[edge.predicate],
            edge.source_label,
            edge.source_key,
            edge.target_label,
            edge.target_key,
        )
    )
    return layer


#: The order nodes and edges are emitted in: every node before any edge.
NODE_ORDER = ("Directory", "File", "Requirement", "Decision", "Baseline", "Crate")
EDGE_ORDER = (
    "CONTAINS",
    "DEFINES",
    "CITES",
    "REFERENCES",
    "TARGETS",
    "RELATED_TO",
    "DECLARES",
    "DEPENDS_ON",
)


def directories(layer: Layer, covered: list[tuple[str, str]]) -> None:
    """Every directory above a covered file, and the `CONTAINS` tree."""
    seen: set[str] = set()
    for path, _ in covered:
        parent = posixpath.dirname(path) or "."
        layer.edges.append(Edge("CONTAINS", "Directory", parent, "File", path))
        child = parent
        while child not in seen:
            seen.add(child)
            if child == ".":
                break
            above = posixpath.dirname(child) or "."
            layer.edges.append(
                Edge("CONTAINS", "Directory", above, "Directory", child)
            )
            child = above
    for path in sorted(seen):
        name = "." if path == "." else posixpath.basename(path)
        layer.nodes.append(Node("Directory", path, {"name": name}))


def occurrences(layer: Layer, documents: list[Document], blocks: Blocks) -> None:
    """`CITES` and `REFERENCES`: every non-definition occurrence, partitioned.

    An occurrence inside a definition block is a citation from the block's
    owner; one outside every block is a reference from the file. The
    identifier's own definition and an occurrence of a block's owner inside
    that block produce nothing. Both edges keep the first occurrence of their
    endpoint pair.
    """
    sites = {(d.path, d.line, d.ident) for d in blocks.definitions.values()}
    cites: dict[tuple[str, str], tuple[int, str]] = {}
    references: dict[tuple[str, str], int] = {}

    for document in documents:
        for index, line in enumerate(document.lines):
            number = index + 1
            owner = document.owner[index] if document.owner else None
            for match in IDENTIFIER.finditer(line):
                ident = match.group(0)
                if (document.path, number, ident) in sites or ident == owner:
                    continue
                if ident not in blocks.definitions:
                    layer.dangling.append(f"{document.path}:{number} {ident}")
                    continue
                if owner is None:
                    references.setdefault((document.path, ident), number)
                elif label_of(owner) == "Requirement" and label_of(ident) == "Decision":
                    layer.unplaced.append(
                        f"{document.path}:{number} {owner} -> {ident}"
                    )
                else:
                    cites.setdefault((owner, ident), (number, document.path))

    for (owner, ident), (number, path) in cites.items():
        layer.edges.append(
            Edge(
                "CITES",
                label_of(owner),
                owner,
                label_of(ident),
                ident,
                {"firstLine": number, "path": path},
            )
        )
    for (path, ident), number in references.items():
        layer.edges.append(
            Edge(
                "REFERENCES",
                "File",
                path,
                label_of(ident),
                ident,
                {"firstLine": number},
            )
        )


def targets(layer: Layer, blocks: Blocks, covered: list[tuple[str, str]]) -> None:
    """`TARGETS`: the file each divergence record names in its target field.

    The field opens with the file, as a code span, and goes on to name the
    section of it; only the opening span is the target. A field reading *both*
    names the two coordination documents, which are the pair every divergence
    record is raised against.
    """
    pair = sorted(path for path, kind in covered if kind == "coordination")
    if len(pair) != 2:
        raise ScanError(f"expected two coordination documents, found {pair}")
    for ident in sorted(i for i in blocks.definitions if i.startswith("DIV-")):
        definition = blocks.definitions[ident]
        if ident not in blocks.targets:
            raise ScanError(
                f"{definition.path}:{definition.line}: {ident} has no target"
            )
        value, number = blocks.targets[ident]
        opening = LEADING_SPAN.match(value)
        if value.lower().startswith("both"):
            named = pair
        elif opening is not None and opening.group(1) in pair:
            named = [opening.group(1)]
        else:
            raise ScanError(f"{definition.path}:{number}: {ident} targets {value!r}")
        for path in named:
            layer.edges.append(Edge("TARGETS", "Requirement", ident, "File", path))


def related(layer: Layer, documents: list[Document], files: set[str]) -> None:
    """`RELATED_TO`: the files a front-matter `related:` list declares adjacent."""
    for document in documents:
        if "related" not in document.front:
            continue
        value, number = document.front["related"]
        listed = [item.strip() for item in value.strip().strip("[]").split(",")]
        for item in listed:
            if not item:
                continue
            target = posixpath.normpath(
                posixpath.join(posixpath.dirname(document.path), item)
            )
            if target not in files:
                raise ScanError(
                    f"{document.path}:{number}: related {item!r} is not a covered file"
                )
            layer.edges.append(
                Edge("RELATED_TO", "File", document.path, "File", target)
            )


def baselines(layer: Layer, root: Path, files: set[str]) -> None:
    """`Baseline` and `DECLARES`: one per dated entry of `BENCHMARKS.md`."""
    if BENCHMARKS not in files:
        return
    lines = (root / BENCHMARKS).read_text(encoding="utf-8").splitlines()
    fenced = fenced_lines(lines)
    entries: list[tuple[int, str, str]] = []
    ends: list[int] = []
    for index, line in enumerate(lines):
        if fenced[index]:
            continue
        heading = HEADING.match(line)
        if heading is None or len(heading.group(1)) > 2:
            continue
        if entries and len(ends) < len(entries):
            ends.append(index)
        match = BASELINE.match(line)
        if match is not None:
            entries.append((index, match.group(1), match.group(2)))
    while len(ends) < len(entries):
        ends.append(len(lines))

    seen: set[str] = set()
    for (index, date, subject), end in zip(entries, ends, strict=True):
        body = " ".join(line.strip() for line in lines[index + 1 : end])
        clause = TARGET_OF_RECORD.search(body)
        if clause is None:
            raise ScanError(f"{BENCHMARKS}:{index + 1}: the entry names no target")
        named: list[str] = []
        for span in CODE_SPAN.findall(clause.group(1)):
            if span not in named:
                named.append(span)
        if not named:
            raise ScanError(f"{BENCHMARKS}:{index + 1}: the target clause is empty")
        key = f"{date}#{slug_of(subject)}"
        if key in seen:
            raise ScanError(f"{BENCHMARKS}:{index + 1}: baseline key {key} repeats")
        seen.add(key)
        layer.nodes.append(
            Node(
                "Baseline",
                key,
                {"date": date, "subject": subject, "target": "; ".join(named)},
            )
        )
        layer.edges.append(Edge("DECLARES", "File", BENCHMARKS, "Baseline", key))


def crates(layer: Layer, root: Path) -> None:
    """`Crate` and `DEPENDS_ON`, from the `[dependencies]` table alone.

    The edge runs from the component named after the package, which the Rust
    layer emits and this one only binds.
    """
    manifest = tomllib.loads((root / MANIFEST).read_text(encoding="utf-8"))
    package = manifest["package"]["name"]
    justifications = justification_table(root)

    for name, declared in sorted(manifest.get("dependencies", {}).items()):
        if isinstance(declared, str):
            declared = {"version": declared}
        if "path" in declared or "git" in declared or "version" not in declared:
            raise ScanError(f"{MANIFEST}: {name} is not a crates.io dependency")
        properties: dict[str, object] = {
            "version": declared["version"],
            "source": "crates.io",
        }
        if declared.get("features"):
            properties["features"] = ", ".join(declared["features"])
        if name in justifications:
            properties["justification"] = justifications[name]
        layer.nodes.append(Node("Crate", name, properties))
        layer.edges.append(
            Edge(
                "DEPENDS_ON",
                "Component",
                package,
                "Crate",
                name,
                {"optional": bool(declared.get("optional", False))},
            )
        )


def justification_table(root: Path) -> dict[str, str]:
    """Each crate's *Chosen because* cell, read from the technology stack."""
    path = root / JUSTIFICATION_FILE
    if not path.is_file():
        return {}
    found: dict[str, str] = {}
    header: list[str] | None = None
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.startswith("|"):
            header = None
            continue
        row = cells(line)
        if header is None:
            header = row
            continue
        if is_separator(row) or not header or header[0] != "Crate":
            continue
        if JUSTIFICATION_COLUMN not in header:
            continue
        text = row[header.index(JUSTIFICATION_COLUMN)]
        for name in CODE_SPAN.findall(row[0]):
            found[name] = text
    return found


# -------------------------------------------------------------- the output ---


IDENTITY = {
    "Directory": "path",
    "File": "path",
    "Requirement": "id",
    "Decision": "id",
    "Baseline": "key",
    "Crate": "name",
    "Component": "name",
}


def cypher_literal(value: object) -> str:
    """One Cypher literal: a boolean or integer bare, anything else a string."""
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    text = str(value)
    if "\n" in text or "\r" in text:
        raise ScanError(f"a value spans lines and cannot be emitted: {text!r}")
    text = text.replace("\\", "\\\\").replace("'", "\\'")
    return f"'{text}'"


def emit_cypher(layer: Layer, commit: str, date: str) -> list[str]:
    """One statement per line, with `//` comments between the sections."""
    stamp = f"gitCommit = {cypher_literal(commit)}, "
    stamp_date = f"gitDate = {cypher_literal(date)}"
    out: list[str] = [
        "// The document layer of the knowledge graph, re-derived from the tree by",
        "// scripts/knowledge/doc-layer.py. One statement per line; a line that",
        "// starts with // is a comment and an empty line separates the sections.",
        "//",
        "// Nodes are upserted on their identity alone. Every edge binds both",
        "// endpoints with MATCH, MERGEs the relationship alone, and re-binds it by",
        "// its type before it SETs a property, so a MATCH that binds nothing is a",
        "// no-op rather than a stub, and a property never lands on an edge of",
        "// another type. A Component this layer does not own is never created.",
        "//",
        "// DEFINES and REFERENCES join the same (File, identifier) pair wherever a",
        "// file names an identifier it also defines. knowledge-model.md records",
        "// that a MERGE between a pair already joined by another type matches that",
        "// edge and creates nothing: write such edges by diffing --format json",
        "// against the live set.",
        f"// Stamped {commit} ({date}).",
    ]

    current = ""
    for node in layer.nodes:
        if node.label != current:
            current = node.label
            out += ["", f"// --- nodes: {current} ---"]
        identity = IDENTITY[node.label]
        assignments = ", ".join(
            f"n.{name} = {cypher_literal(value)}"
            for name, value in sorted(node.properties.items())
        )
        assignments += f", n.{stamp}n.{stamp_date}"
        out.append(
            f"MERGE (n:{node.label} {{{identity}: {cypher_literal(node.key)}}}) "
            f"ON CREATE SET {assignments} ON MATCH SET {assignments}"
        )

    current = ""
    for edge in layer.edges:
        if edge.predicate != current:
            current = edge.predicate
            out += ["", f"// --- edges: {current} ---"]
        source = IDENTITY[edge.source_label]
        target = IDENTITY[edge.target_label]
        assignments = f"e.{stamp}e.{stamp_date}"
        for name, value in sorted(edge.properties.items()):
            assignments += f", e.{name} = {cypher_literal(value)}"
        out.append(
            f"MATCH (a:{edge.source_label} {{{source}: "
            f"{cypher_literal(edge.source_key)}}}) "
            f"MATCH (b:{edge.target_label} {{{target}: "
            f"{cypher_literal(edge.target_key)}}}) "
            f"MERGE (a)-[:{edge.predicate}]->(b) WITH DISTINCT a, b "
            f"MATCH (a)-[e:{edge.predicate}]->(b) SET {assignments}"
        )

    for statement in out:
        if len(statement.encode("utf-8")) >= STATEMENT_LIMIT:
            raise ScanError(f"a statement exceeds {STATEMENT_LIMIT} bytes")
    return out


def emit_json(layer: Layer, commit: str, date: str) -> str:
    """The same records, for a reader that wants to diff rather than load."""
    return json.dumps(
        {
            "gitCommit": commit,
            "gitDate": date,
            "nodes": [
                {"label": node.label, "key": node.key, "properties": node.properties}
                for node in layer.nodes
            ],
            "edges": [
                {
                    "predicate": edge.predicate,
                    "source": {"label": edge.source_label, "key": edge.source_key},
                    "target": {"label": edge.target_label, "key": edge.target_key},
                    "properties": edge.properties,
                }
                for edge in layer.edges
            ],
        },
        indent=2,
        sort_keys=True,
    )


def summary(layer: Layer) -> list[str]:
    """Counts per label and per predicate."""
    labels: dict[str, int] = {}
    predicates: dict[str, int] = {}
    for node in layer.nodes:
        labels[node.label] = labels.get(node.label, 0) + 1
    for edge in layer.edges:
        predicates[edge.predicate] = predicates.get(edge.predicate, 0) + 1

    out = ["LABEL          NODES"]
    out += [f"{label:<14} {count}" for label, count in sorted(labels.items())]
    out += ["", "PREDICATE      EDGES"]
    out += [f"{name:<14} {count}" for name, count in sorted(predicates.items())]
    out += ["", f"{'total':<14} {len(layer.nodes)} nodes, {len(layer.edges)} edges"]
    return out


def provenance(root: Path) -> tuple[str, str, bool]:
    """The commit and date every element is stamped with, and whether it is dirty."""

    def git(*arguments: str) -> str:
        return subprocess.run(
            ["git", *arguments],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

    commit = git("rev-parse", "HEAD")
    date = git("show", "-s", "--format=%cs", "HEAD")
    pathspecs = [spec for spec, _, _ in COVERAGE] + [MANIFEST]
    dirty = bool(git("status", "--porcelain", "--", *pathspecs))
    return commit, date, dirty


# ------------------------------------------------------------ the verifier ---
#
# Everything below derives the layer a second time and shares no code with the
# scan above: no pattern, no helper and no table of it is read here. The scan
# walks each file once with a state machine; the verifier finds every
# definition site by its own patterns, walks each block forward from its site,
# derives every record again, and reopens each emitted citation and reference
# at the line it names. A record the scan put on the wrong line, gave the wrong
# property, attributed to the wrong owner, or never emitted, fails here — once,
# under the record's own name, however many checks it fails.


V_ID = re.compile(
    r"(?<![A-Za-z0-9_])(?:FR-[A-Z][A-Z0-9]*-[0-9]+|NFR-[A-Z][A-Z0-9]*-[0-9]+"
    r"|BR-[A-Z][A-Z0-9]*-[0-9]+|UC-[0-9]+|OQ-[0-9]+|DIV-[0-9]+|WL-[0-9]+"
    r"|OD-[0-9]+|ADR-[0-9]+)(?![A-Za-z0-9_])"
)

V_DECISIONS = ("OD-", "ADR-")
V_ROOT_DOCUMENTS = ("CLAUDE.md", "README.md")

Key = tuple[str, ...]


class Verifier:
    """A second derivation of the layer, compared record by record."""

    def __init__(self, layer: Layer, root: Path) -> None:
        self.layer = layer
        self.root = root
        self.records: set[str] = set()
        self.findings: dict[str, list[str]] = {}
        self.cache: dict[str, list[str]] = {}
        self.nodes = {(node.label, node.key): node for node in layer.nodes}
        self.edges: dict[str, list[Edge]] = {}
        for edge in layer.edges:
            self.edges.setdefault(edge.predicate, []).append(edge)

    # -- reading -----------------------------------------------------------

    def text(self, path: str) -> list[str]:
        """The file's lines, read once."""
        if path not in self.cache:
            raw = (self.root / path).read_bytes().decode("utf-8").split("\n")
            if raw and raw[-1] == "":
                raw.pop()
            self.cache[path] = [line.rstrip("\r") for line in raw]
        return self.cache[path]

    def line(self, path: str, number: object) -> str | None:
        """One line, 1-based, or `None` past either end."""
        lines = self.text(path)
        if isinstance(number, int) and 1 <= number <= len(lines):
            return lines[number - 1]
        return None

    def front(self, path: str) -> tuple[dict[str, tuple[str, int]], int]:
        """Front matter as `key -> (value, line)`, and its closing line."""
        if not path.endswith(".md"):
            return {}, 0
        lines = self.text(path)
        if not lines or lines[0].strip() != "---":
            return {}, 0
        found: dict[str, tuple[str, int]] = {}
        for number in range(2, len(lines) + 1):
            raw = lines[number - 1]
            if raw.strip() == "---":
                return found, number
            key, colon, value = raw.partition(":")
            if colon and key and " " not in key:
                found[key] = (value.strip(), number)
        return found, len(lines)

    def code_lines(self, path: str) -> set[int]:
        """Line numbers inside a fence, closing fence included, opening not."""
        inside: set[int] = set()
        opener: tuple[str, int] | None = None
        for number, raw in enumerate(self.text(path), start=1):
            bare = raw.strip()
            fence = bare[:3] if bare[:3] in ("```", "~~~") else None
            if opener is None:
                if fence is not None:
                    opener = (fence[0], len(bare) - len(bare.lstrip(fence[0])))
                continue
            inside.add(number)
            if set(bare) == {opener[0]} and len(bare) >= max(3, opener[1]):
                opener = None
        return inside

    # -- what a file defines -------------------------------------------------

    def sites(self, path: str) -> list[tuple[str, int, str]]:
        """Every `(id, line, shape)` the file defines, by the verifier's patterns."""
        found: list[tuple[str, int, str]] = []
        if path.startswith("docs/adr/"):
            front, _ = self.front(path)
            if "id" in front:
                found.append((front["id"][0], front["id"][1], "record"))
            return found
        in_spec = path.startswith("specification/")
        in_register = path.startswith("docs/spec-technical/")
        if not (in_spec or in_register):
            return found
        code = self.code_lines(path)
        in_questions = False
        for number, raw in enumerate(self.text(path), start=1):
            if number in code:
                continue
            if in_register and raw.startswith("## OD-") and " — " in raw:
                found.append((raw[3:].split(" ", 1)[0], number, "section"))
            if not in_spec:
                continue
            if raw.startswith("- **"):
                ident, close, rest = raw[4:].partition("**")
                shaped = rest.startswith(":") or rest.startswith(" —")
                family = ident.split("-", 1)[0]
                if close and shaped and V_ID.fullmatch(ident):
                    if family in ("FR", "NFR", "BR", "WL"):
                        found.append((ident, number, "bullet"))
            if raw.startswith("## UC-") and " — " in raw:
                found.append((raw[3:].split(" ", 1)[0], number, "section"))
            if raw.startswith("## DIV-") and V_ID.fullmatch(raw[3:].strip()):
                found.append((raw[3:].strip(), number, "section"))
            if raw.replace(" ", "").startswith("|Id|Question|"):
                in_questions = True
            elif not raw.startswith("|"):
                in_questions = False
            elif in_questions and raw.startswith("| OQ-"):
                found.append((raw[2:].split(" ", 1)[0], number, "row"))
        return found

    def spans(self, path: str) -> list[tuple[str, int, int]]:
        """Every block in the file as `(owner, first line, last line)`."""
        lines = self.text(path)
        code = self.code_lines(path)
        out: list[tuple[str, int, int]] = []
        for ident, start, shape in self.sites(path):
            end = len(lines)
            if shape == "row":
                end = start
            elif shape == "record":
                start = self.front(path)[1] + 1
            elif shape == "section":
                for number in range(start + 1, len(lines) + 1):
                    raw = lines[number - 1]
                    top = raw in ("#", "##") or raw.startswith(("# ", "## "))
                    if top and number not in code:
                        end = number - 1
                        break
            else:
                for number in range(start + 1, len(lines) + 1):
                    raw = lines[number - 1]
                    if number in code or raw.strip() == "":
                        continue
                    if raw[0] not in " \t":
                        end = number - 1
                        break
            out.append((ident, start, end))
        return out

    # -- the bookkeeping -----------------------------------------------------

    def fail(self, record: str, reason: str) -> None:
        """Files one reason against one record."""
        self.records.add(record)
        self.findings.setdefault(record, []).append(reason)

    def emitted(self, what: str, pairs: list[tuple[Key, object]]) -> dict[Key, object]:
        """The layer's records of one kind, keyed; an identity emitted twice fails."""
        out: dict[Key, object] = {}
        for key, value in pairs:
            if key in out:
                self.fail(f"{what} {' -> '.join(key)}", "emitted twice")
            out[key] = value
        return out

    def compare(
        self, what: str, derived: dict[Key, object], emitted: dict[Key, object]
    ) -> None:
        """Both directions of one comparison, one record per identity."""
        for key in sorted(set(derived) | set(emitted)):
            record = f"{what} {' -> '.join(key)}"
            self.records.add(record)
            if key not in emitted:
                self.fail(record, f"derived {derived[key]!r}, not emitted")
            elif key not in derived:
                self.fail(record, f"emitted {emitted[key]!r}, not derived")
            elif emitted[key] != derived[key]:
                self.fail(record, f"emitted {emitted[key]!r}, derived {derived[key]!r}")

    def run(self) -> tuple[int, list[str], list[str]]:
        """Every check; returns the records read, the mismatches and the notes."""
        files = self.check_files()
        self.check_directories(files)
        definitions = self.check_definitions(files)
        notes = self.check_occurrences(files, definitions)
        self.check_targets(definitions)
        self.check_related(files)
        self.check_baselines(files)
        self.check_crates()
        self.check_endpoints()
        mismatches = [
            f"{record}: {'; '.join(reasons)}"
            for record, reasons in sorted(self.findings.items())
        ]
        return len(self.records), mismatches, notes

    # -- the checks ----------------------------------------------------------

    def check_files(self) -> dict[str, str]:
        """The covered set, classified again, against the `File` nodes."""
        listing = subprocess.run(
            ["git", "ls-files", "-z"], cwd=self.root, check=True, capture_output=True
        ).stdout.decode("utf-8")
        kinds: dict[str, str] = {}
        for path in listing.split("\0"):
            kind = None
            if path in V_ROOT_DOCUMENTS:
                kind = "coordination"
            elif path == "BENCHMARKS.md" or path.startswith("benches/"):
                kind = "benchmark"
            elif path.endswith(".md") and path.startswith("specification/"):
                kind = "spec"
            elif path.endswith(".md") and path.startswith("docs/spec-technical/"):
                kind = "techspec"
            elif path.endswith(".md") and path.startswith("docs/adr/"):
                kind = "adr"
            elif path.startswith(("scripts/mariadb/", "scripts/sqlite/")):
                kind = "fixture"
            elif path.endswith(".rs") and path.startswith("src/"):
                kind = "source"
            elif path.endswith(".rs") and path.startswith("tests/"):
                kind = "test"
            elif path.startswith("templates/"):
                kind = "template"
            if kind is not None:
                kinds[path] = kind

        derived: dict[Key, object] = {}
        for path, kind in kinds.items():
            props: dict[str, object] = {"name": path.split("/")[-1], "kind": kind}
            front = {} if path.endswith(".pem") else self.front(path)[0]
            for name in ("title", "status"):
                if name in front:
                    props[name] = front[name][0]
            derived[(path,)] = props
        emitted = self.emitted(
            "File",
            [((n.key,), n.properties) for n in self.layer.nodes if n.label == "File"],
        )
        self.compare("File", derived, emitted)
        return kinds

    def check_directories(self, files: dict[str, str]) -> None:
        """Every directory above a covered file, and the tree that joins them."""
        derived: dict[Key, object] = {(".",): {"name": "."}}
        contains: dict[Key, object] = {}
        for path in files:
            parts = path.split("/")
            contains[("/".join(parts[:-1]) or ".", path)] = ("Directory", "File")
            for depth in range(1, len(parts)):
                child = "/".join(parts[:depth])
                above = "/".join(parts[: depth - 1]) or "."
                derived[(child,)] = {"name": parts[depth - 1]}
                contains[(above, child)] = ("Directory", "Directory")
        for (path,) in derived:
            if not (self.root / path).is_dir():
                self.fail(f"Directory {path}", "not a directory in the tree")
        emitted = self.emitted(
            "Directory",
            [
                ((n.key,), n.properties)
                for n in self.layer.nodes
                if n.label == "Directory"
            ],
        )
        self.compare("Directory", derived, emitted)
        emitted = self.emitted(
            "CONTAINS",
            [
                ((e.source_key, e.target_key), (e.source_label, e.target_label))
                for e in self.edges.get("CONTAINS", [])
            ],
        )
        self.compare("CONTAINS", contains, emitted)

    def check_definitions(self, files: dict[str, str]) -> dict[str, tuple[str, int]]:
        """Every definition site found again, and every identifier node rebuilt."""
        found: dict[str, tuple[str, int]] = {}
        for path in sorted(files):
            if not path.endswith(".md"):
                continue
            for ident, number, _ in self.sites(path):
                if ident in found:
                    self.fail(f"DEFINES {ident}", f"two sites, {found[ident]}")
                found[ident] = (path, number)

        defines = {(path, i): {"line": n} for i, (path, n) in found.items()}
        emitted = self.emitted(
            "DEFINES",
            [
                ((e.source_key, e.target_key), e.properties)
                for e in self.edges.get("DEFINES", [])
            ],
        )
        self.compare("DEFINES", defines, emitted)

        derived: dict[Key, object] = {}
        for ident, (path, number) in found.items():
            source = self.line(path, number) or ""
            family, _, rest = ident.partition("-")
            label = "Decision" if ident.startswith(V_DECISIONS) else "Requirement"
            props: dict[str, object] = {"family": family}
            digits = rest
            if family in ("FR", "NFR", "BR"):
                props["area"], _, digits = rest.partition("-")
            props["ordinal"] = int(digits)
            if family in ("UC", "OD"):
                props["title"] = source.split(" — ", 1)[1].strip()
            elif family == "OQ":
                props["title"] = source.split(" | ")[1].strip()
            elif family == "ADR":
                title = self.front(path)[0].get("title")
                if title is not None:
                    props["title"] = title[0]
            elif family == "DIV":
                for raw in self.text(path):
                    if raw.startswith(f"| [{ident}]("):
                        cell = raw.rstrip().rstrip("|").rsplit(" | ", 1)[1]
                        props["title"] = cell.strip()
                        break
            derived[(ident,)] = (label, props)
        emitted = self.emitted(
            "identifier",
            [
                ((n.key,), (n.label, n.properties))
                for n in self.layer.nodes
                if n.label in ("Requirement", "Decision")
            ],
        )
        self.compare("identifier", derived, emitted)
        for node in self.layer.nodes:
            if isinstance(node.properties.get("ordinal"), bool):
                self.fail(f"identifier {node.key}", "ordinal is a boolean")
        return found

    def check_occurrences(
        self, files: dict[str, str], definitions: dict[str, tuple[str, int]]
    ) -> list[str]:
        """`CITES` and `REFERENCES`, derived again, and each reopened at its line."""
        cites: dict[Key, object] = {}
        references: dict[Key, object] = {}
        dangling = 0
        unplaced = 0
        site_lines = {(p, n, i) for i, (p, n) in definitions.items()}

        for path in sorted(files):
            if path.endswith(".pem"):
                continue
            owners: dict[int, str] = {}
            for ident, first, last in self.spans(path):
                for number in range(first, last + 1):
                    owners[number] = ident
            for number, raw in enumerate(self.text(path), start=1):
                owner = owners.get(number)
                for ident in V_ID.findall(raw):
                    if (path, number, ident) in site_lines or owner == ident:
                        continue
                    if ident not in definitions:
                        dangling += 1
                    elif owner is None:
                        references.setdefault((path, ident), number)
                    elif ident.startswith(V_DECISIONS) and not owner.startswith(
                        V_DECISIONS
                    ):
                        unplaced += 1
                    else:
                        cites.setdefault((owner, ident), (number, path))

        emitted = self.emitted(
            "CITES",
            [
                (
                    (e.source_key, e.target_key),
                    (e.properties.get("firstLine"), e.properties.get("path")),
                )
                for e in self.edges.get("CITES", [])
            ],
        )
        self.compare("CITES", cites, emitted)
        emitted = self.emitted(
            "REFERENCES",
            [
                ((e.source_key, e.target_key), e.properties.get("firstLine"))
                for e in self.edges.get("REFERENCES", [])
            ],
        )
        self.compare("REFERENCES", references, emitted)

        for edge in self.edges.get("CITES", []) + self.edges.get("REFERENCES", []):
            path = str(edge.properties.get("path", edge.source_key))
            number = edge.properties.get("firstLine")
            source = self.line(path, number) if (self.root / path).is_file() else None
            if source is None or edge.target_key not in V_ID.findall(source):
                self.fail(
                    f"{edge.predicate} {edge.source_key} -> {edge.target_key}",
                    f"{path}:{number} does not name {edge.target_key}",
                )

        if len(self.layer.dangling) != dangling:
            self.fail("dangling", f"{len(self.layer.dangling)} reported, {dangling}")
        if len(self.layer.unplaced) != unplaced:
            self.fail("unplaced", f"{len(self.layer.unplaced)} reported, {unplaced}")
        return [
            f"{dangling} occurrences name an identifier nothing defines",
            f"{unplaced} occurrences inside a requirement's block name a decision, "
            "for which the model has no predicate",
        ]

    def check_targets(self, definitions: dict[str, tuple[str, int]]) -> None:
        """The target field of every divergence record, read again."""
        derived: dict[Key, object] = {}
        for ident, (path, number) in definitions.items():
            if not ident.startswith("DIV-"):
                continue
            named: list[str] = []
            for raw in self.text(path)[number:]:
                if raw.startswith(("# ", "## ")):
                    break
                if raw.startswith("**Target**: "):
                    value = raw[len("**Target**: ") :]
                    if value.startswith("both"):
                        named = list(V_ROOT_DOCUMENTS)
                    named += [d for d in V_ROOT_DOCUMENTS if value.startswith(f"`{d}`")]
                    break
            if not named:
                self.fail(f"TARGETS {ident}", "the record names no target")
            for document in named:
                derived[(ident, document)] = ()
        emitted = self.emitted(
            "TARGETS",
            [((e.source_key, e.target_key), ()) for e in self.edges.get("TARGETS", [])],
        )
        self.compare("TARGETS", derived, emitted)

    def check_related(self, files: dict[str, str]) -> None:
        """Every front-matter `related:` entry, resolved again."""
        derived: dict[Key, object] = {}
        for path in files:
            front = {} if path.endswith(".pem") else self.front(path)[0]
            if "related" not in front:
                continue
            for item in front["related"][0].strip("[] ").split(","):
                item = item.strip()
                if not item:
                    continue
                parts: list[str] = path.split("/")[:-1]
                for part in item.split("/"):
                    if part == "..":
                        parts = parts[:-1]
                    elif part not in ("", "."):
                        parts.append(part)
                derived[(path, "/".join(parts))] = ()
        emitted = self.emitted(
            "RELATED_TO",
            [
                ((e.source_key, e.target_key), ())
                for e in self.edges.get("RELATED_TO", [])
            ],
        )
        self.compare("RELATED_TO", derived, emitted)

    def check_baselines(self, files: dict[str, str]) -> None:
        """Every dated entry of `BENCHMARKS.md`, read again."""
        derived: dict[Key, object] = {}
        if "BENCHMARKS.md" in files:
            lines = self.text("BENCHMARKS.md")
            code = self.code_lines("BENCHMARKS.md")
            for number, raw in enumerate(lines, start=1):
                if number in code or not raw.startswith("## 20"):
                    continue
                date, dash, subject = raw[3:].partition(" — ")
                if not dash:
                    continue
                words = re.findall(r"[a-z0-9]+", subject.replace("`", "").lower())
                window: list[str] = []
                for later in lines[number : number + 12]:
                    if later.startswith("## "):
                        break
                    window.append(later.strip())
                joined = " ".join(window)
                clause = joined[joined.find("Target of record:") :]
                cut = re.search(r"\.(\s|\*|$)", clause)
                clause = clause[: cut.start()] if cut else clause
                spans: list[str] = []
                for piece in clause.split("`")[1::2]:
                    if piece not in spans:
                        spans.append(piece)
                derived[(date + "#" + "-".join(words),)] = {
                    "date": date,
                    "subject": subject.strip(),
                    "target": "; ".join(spans),
                }
        emitted = self.emitted(
            "Baseline",
            [
                ((n.key,), n.properties)
                for n in self.layer.nodes
                if n.label == "Baseline"
            ],
        )
        self.compare("Baseline", derived, emitted)
        declares = {("BENCHMARKS.md", key): () for (key,) in derived}
        emitted = self.emitted(
            "DECLARES",
            [
                ((e.source_key, e.target_key), ())
                for e in self.edges.get("DECLARES", [])
            ],
        )
        self.compare("DECLARES", declares, emitted)

    def check_crates(self) -> None:
        """The `[dependencies]` table, read again line by line, and each crate's
        justification cell, read again from the technology stack."""
        section = ""
        package = ""
        statements: dict[str, str] = {}
        current = ""
        depth = 0
        for raw in self.text("Cargo.toml"):
            bare = "" if raw.lstrip().startswith("#") else raw.split(" #", 1)[0].strip()
            if depth == 0 and bare.startswith("[") and bare.endswith("]"):
                section = bare.strip("[]")
                continue
            if section == "package" and bare.startswith("name"):
                package = bare.split("=", 1)[1].strip().strip('"')
            if section != "dependencies" or not bare:
                continue
            if depth == 0:
                current = bare.split("=", 1)[0].strip()
                statements[current] = ""
            statements[current] += " " + bare
            depth += bare.count("{") + bare.count("[")
            depth -= bare.count("}") + bare.count("]")

        derived: dict[Key, object] = {}
        for name, statement in statements.items():
            version = re.search(r'version\s*=\s*"([^"]*)"', statement)
            first = re.findall(r'"([^"]*)"', statement)
            props: dict[str, object] = {
                "version": version.group(1) if version else first[0],
                "source": "crates.io",
            }
            features = re.search(r"\bfeatures\s*=\s*\[([^\]]*)\]", statement)
            listed = re.findall(r'"([^"]*)"', features.group(1)) if features else []
            if listed:
                props["features"] = ", ".join(listed)
            derived[(name,)] = props

        chosen = -1
        for raw in self.text("docs/spec-technical/technology-stack.md"):
            if not raw.startswith("|"):
                chosen = -1
                continue
            parts = [part.strip() for part in raw.strip().strip("|").split(" | ")]
            if parts[0] == "Crate":
                chosen = -1
                if "Chosen because" in parts:
                    chosen = parts.index("Chosen because")
                continue
            if chosen < 0 or len(parts) <= chosen:
                continue
            for name in re.findall(r"`([^`]+)`", parts[0]):
                props = derived.get((name,))
                if isinstance(props, dict):
                    props["justification"] = parts[chosen]

        emitted = self.emitted(
            "Crate",
            [((n.key,), n.properties) for n in self.layer.nodes if n.label == "Crate"],
        )
        self.compare("Crate", derived, emitted)
        depends = {
            (package, name): ("Component", "optional = true" in statement)
            for name, statement in statements.items()
        }
        emitted = self.emitted(
            "DEPENDS_ON",
            [
                (
                    (e.source_key, e.target_key),
                    (e.source_label, e.properties.get("optional")),
                )
                for e in self.edges.get("DEPENDS_ON", [])
            ],
        )
        self.compare("DEPENDS_ON", depends, emitted)
        if not (self.root / "src" / "main.rs").is_file():
            self.fail(f"Component {package}", "no binary realises it")

    def check_endpoints(self) -> None:
        """Every endpoint is a node of this layer, the binary component aside."""
        for edge in self.layer.edges:
            record = f"{edge.predicate} {edge.source_key} -> {edge.target_key}"
            foreign = edge.source_label == "Component"
            if not foreign and (edge.source_label, edge.source_key) not in self.nodes:
                self.fail(record, f"{edge.source_key} is not a node of this layer")
            if (edge.target_label, edge.target_key) not in self.nodes:
                self.fail(record, f"{edge.target_key} is not a node of this layer")
        defines: dict[str, int] = {}
        for edge in self.edges.get("DEFINES", []):
            defines[edge.target_key] = defines.get(edge.target_key, 0) + 1
        for label, key in self.nodes:
            if label in ("Requirement", "Decision") and defines.get(key) != 1:
                self.fail(f"identifier {key}", f"{defines.get(key, 0)} DEFINES edges")


def verify(layer: Layer, root: Path) -> tuple[int, list[str], list[str]]:
    """Re-derives the layer independently; returns the count, mismatches, notes."""
    return Verifier(layer, root).run()


# ----------------------------------------------------------------- the main ---


def repository_root() -> Path:
    """The repository this script lives in, found from the script's own path."""
    return Path(__file__).resolve().parent.parent.parent


def main() -> int:
    """Scans the tree and writes what was asked for; returns the exit code."""
    parser = argparse.ArgumentParser(
        description="Re-derive the document layer of the knowledge graph.",
    )
    parser.add_argument(
        "--format",
        choices=("cypher", "json"),
        default="cypher",
        help="the shape of the emitted layer (default: cypher)",
    )
    parser.add_argument(
        "--summary",
        action="store_true",
        help="print counts per label and predicate instead of the layer",
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="re-derive the layer independently and re-read every record",
    )
    parser.add_argument(
        "--commit", help="override the commit every element is stamped with"
    )
    parser.add_argument(
        "--date", help="override the date every element is stamped with"
    )
    arguments = parser.parse_args()

    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")

    root = repository_root()
    try:
        layer = scan_tree(root)
    except ScanError as error:
        print(f"doc-layer.py: {error}", file=sys.stderr)
        return 1

    if arguments.verify:
        checked, mismatches, notes = verify(layer, root)
        for mismatch in mismatches:
            print(f"MISMATCH {mismatch}")
        print(f"checked {checked} records at file:line; {len(mismatches)} mismatched")
        for note in notes:
            print(note)
        return 1 if mismatches else 0

    if arguments.summary:
        print("\n".join(summary(layer)))
        return 0

    commit, date, dirty = provenance(root)
    if arguments.commit:
        commit = arguments.commit
    if arguments.date:
        date = arguments.date
    if dirty:
        print(
            f"doc-layer.py: the scanned tree has uncommitted changes; {commit} "
            "stamps a state that is not what was read",
            file=sys.stderr,
        )

    try:
        if arguments.format == "json":
            print(emit_json(layer, commit, date))
        else:
            print("\n".join(emit_cypher(layer, commit, date)))
    except ScanError as error:
        print(f"doc-layer.py: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
