#!/usr/bin/env python3

"""Re-derive the Rust layer of the knowledge graph from the source tree.

One command turns every ``.rs`` file under ``src/`` and ``tests/`` into the
nodes and edges the graph holds for Rust: ``Component``, ``Type``, ``Trait``,
``Function``, ``Method``, ``Test``, ``Variant`` and ``Const``, joined by
``IMPLEMENTED_BY``, ``DECLARES``, ``MEMBER_OF``, ``IMPLEMENTS`` and
``VERIFIES``.

It emits; it never writes. Nothing here opens a connection to the graph, issues
a Cypher write, or touches ``~/.roadmaps/``. The output goes to standard output
and the ``knowledge-authority`` skill, which is the graph's only writer, decides
what to do with it.

The vocabulary — every label, every predicate, every property and what each one
means — is fixed by ``knowledge-model.md`` at the repository root. This file
speaks that vocabulary and does not extend it.

Usage:

    scripts/knowledge/rust-layer.py                 # Cypher on stdout
    scripts/knowledge/rust-layer.py --format json   # the same records as JSON
    scripts/knowledge/rust-layer.py --summary       # counts per label and predicate
    scripts/knowledge/rust-layer.py --verify        # re-read every record at file:line

``--verify`` is the check the layer is answerable for: it reopens each emitted
record at the ``file:line`` it claims and confirms that the source there
declares that symbol, with that name, that kind and that visibility. It uses a
single-line matcher that shares no code with the scan, so a record the scan
misattributed fails it. The exit code is ``0`` when every record matched and
``1`` when any did not.

Standard library only, and no configuration: the tree is the input.
"""

from __future__ import annotations

import argparse
import json
import random
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

# --------------------------------------------------------------- the tree ---

#: Directories scanned, relative to the repository root.
SCANNED = ("src", "tests")

#: The crate the `src/` tree realises. Its module paths are rooted here.
CRATE = "tpl"

#: The file that realises the binary, and the component name it carries.
#:
#: `src/lib.rs` gets no component. Its module path is the crate name, which is
#: the name this file already holds, and `Component.name` is an identity
#: property: two components cannot hold one identity, so one of them has to give
#: way and it is the crate root. Choosing differently is a change to the model,
#: and the model belongs to the `knowledge-authority` skill, so the collision is
#: recorded in README.md rather than settled here.
BINARY_FILE = "src/main.rs"
CRATE_ROOT_FILE = "src/lib.rs"

# ------------------------------------------------------------- the grammar ---

#: A visibility prefix, optional. `pub(in path)` is covered by the parenthesis.
VISIBILITY = r"(?:pub(?:\s*\([^)]*\))?\s+)?"

#: An ordinary Rust identifier.
IDENTIFIER = r"[A-Za-z_][A-Za-z0-9_]*"

ITEM_PATTERNS: dict[str, re.Pattern[str]] = {
    "mod": re.compile(rf"^{VISIBILITY}mod\s+({IDENTIFIER})\s*[;{{]"),
    "fn": re.compile(
        rf"^{VISIBILITY}(?:default\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?"
        rf'(?:extern\s+(?:""|"[^"]*")?\s*)?fn\s+({IDENTIFIER})'
    ),
    "struct": re.compile(rf"^{VISIBILITY}struct\s+({IDENTIFIER})"),
    "enum": re.compile(rf"^{VISIBILITY}enum\s+({IDENTIFIER})"),
    "union": re.compile(rf"^{VISIBILITY}union\s+({IDENTIFIER})"),
    "alias": re.compile(rf"^{VISIBILITY}type\s+({IDENTIFIER})"),
    "trait": re.compile(rf"^{VISIBILITY}(?:unsafe\s+)?trait\s+({IDENTIFIER})"),
    "const": re.compile(rf"^{VISIBILITY}const\s+([A-Z_][A-Za-z0-9_]*)\s*:"),
    "static": re.compile(rf"^{VISIBILITY}static\s+(?:mut\s+)?([A-Z_][A-Za-z0-9_]*)\s*:"),
    "impl": re.compile(r"^impl(?:\s*<)?"),
}

#: The order the patterns above are tried in. `fn` precedes `const`, so that
#: `const fn` is a function and not a constant named `fn`.
ITEM_ORDER = (
    "mod", "fn", "struct", "enum", "union", "alias", "trait",
    "const", "static", "impl",
)

#: An enum variant: an identifier at the head of a line inside an enum body.
VARIANT = re.compile(rf"^({IDENTIFIER})\s*(?:[,({{=]|$)")

#: The start of a raw string literal, with the hashes that close it.
RAW_STRING = re.compile(r"(?:b|c)?r(?P<hashes>#*)\"")

#: A char literal, as against a lifetime. `'{'` must not open a block.
CHAR_LITERAL = re.compile(r"'(?:\\.|[^'\\])'")

#: A test function name that names the requirement it verifies, per
#: `docs/spec-technical/verification.md`.
TEST_NAME = re.compile(r"^(fr|nfr|br|uc|oq|div|wl)_([a-z0-9]+)_(\d+)_")

#: The families whose identifiers carry no middle segment.
BARE_FAMILIES = frozenset({"uc", "oq", "div", "wl"})


# ------------------------------------------------------------- the records ---


@dataclass
class Node:
    """One node of the Rust layer, with the properties its label declares."""

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
    """Everything one scan of the tree produced."""

    nodes: list[Node] = field(default_factory=list)
    edges: list[Edge] = field(default_factory=list)


# --------------------------------------------------------------- the scope ---


@dataclass
class Frame:
    """One brace-delimited block, and what it means for the items inside it.

    `kind` is `module` for the file itself and for an inline `mod`, `impl` for
    an `impl` block, `enum` for an enum body, `trait` for a trait declaration,
    `fn` for a function body, and `other` for every other pair of braces — a
    struct body, a match, a closure. Only the first three admit items.
    """

    kind: str
    depth: int
    module: str = ""
    owner: str = ""
    trait_key: str = ""


# ------------------------------------------------------------ the stripping ---


class Stripper:
    """Removes comments and literals, so that a brace is a block delimiter.

    Rust block comments nest, raw strings end on a quote followed by as many
    hashes as opened them, and `'{'` is a character and not a block. Each of
    those turns a naive brace count into the wrong answer, so the scan never
    sees a raw line.
    """

    def __init__(self) -> None:
        self.block_depth = 0
        self.in_string = False
        self.in_raw = False
        self.raw_hashes = 0

    def clean(self, line: str) -> str:
        """The line with every comment and literal body removed."""
        out: list[str] = []
        index = 0
        length = len(line)

        while index < length:
            if self.block_depth > 0:
                if line.startswith("/*", index):
                    self.block_depth += 1
                    index += 2
                elif line.startswith("*/", index):
                    self.block_depth -= 1
                    index += 2
                else:
                    index += 1
                continue

            if self.in_raw:
                closing = '"' + "#" * self.raw_hashes
                if line.startswith(closing, index):
                    self.in_raw = False
                    index += len(closing)
                else:
                    index += 1
                continue

            if self.in_string:
                if line[index] == "\\":
                    index += 2
                elif line[index] == '"':
                    self.in_string = False
                    index += 1
                else:
                    index += 1
                continue

            if line.startswith("//", index):
                break
            if line.startswith("/*", index):
                self.block_depth = 1
                index += 2
                continue

            raw = RAW_STRING.match(line, index)
            if raw is not None:
                self.in_raw = True
                self.raw_hashes = len(raw.group("hashes"))
                index = raw.end()
                continue

            if line[index] == '"':
                self.in_string = True
                index += 1
                continue

            if line[index] == "'":
                char = CHAR_LITERAL.match(line, index)
                if char is not None:
                    index = char.end()
                    continue

            out.append(line[index])
            index += 1

        return "".join(out)


# ------------------------------------------------------------ the naming ---


def module_of(path: str) -> str:
    """The module path a file realises, exactly as the graph spells it.

    `src/model/table.rs` is `tpl::model::table`; `src/lib.rs` and `src/main.rs`
    are both `tpl`; a file under `tests/` keeps the shape of its own path,
    because a test crate is not a module of the library — `tests/support/
    fixture.rs` is `support/fixture`.
    """
    stem = path[: -len(".rs")]
    if stem == "src/lib" or stem == "src/main":
        return CRATE
    if stem.startswith("src/"):
        return CRATE + "::" + stem[len("src/"):].replace("/", "::")
    if stem.startswith("tests/"):
        return stem[len("tests/"):]
    raise ValueError(f"{path} is outside the scanned tree")


def component_of(path: str) -> tuple[str, str] | None:
    """The component a file realises, as `(name, kind)`, or `None`."""
    if path == CRATE_ROOT_FILE:
        return None
    if path == BINARY_FILE:
        return CRATE, "binary"
    if path.startswith("src/"):
        return module_of(path), "module"
    return module_of(path), "test-crate"


def visibility_of(line: str) -> str:
    """The declared visibility of an item, read from its own line."""
    stripped = line.lstrip()
    if stripped.startswith("pub(crate)"):
        return "pub(crate)"
    if stripped.startswith("pub(super)"):
        return "pub(super)"
    if stripped.startswith("pub(in"):
        return "pub(crate)"
    if re.match(r"pub\b", stripped):
        return "pub"
    return "private"


def requirement_of(name: str) -> str | None:
    """The requirement a test name states it verifies, or `None`."""
    match = TEST_NAME.match(name)
    if match is None:
        return None
    family, area, ordinal = match.groups()
    if family in BARE_FAMILIES:
        return f"{family.upper()}-{area.upper()}"
    return f"{family.upper()}-{area.upper()}-{ordinal}"


# ------------------------------------------------------------- the use list ---


def record_use(imports: dict[str, str], body: str) -> None:
    """Adds one `use` statement's names to a scope's import map."""
    body = " ".join(body.split())
    prefix, _, group = body.partition("{")
    prefix = prefix.strip().rstrip(":")
    if group:
        names = group.rstrip("}").split(",")
    else:
        prefix, _, last = body.rpartition("::")
        names = [last]
        prefix = prefix.strip()

    for name in names:
        name = name.strip().split(" as ")[0].strip()
        if not name or name in {"self", "*"} or not re.fullmatch(IDENTIFIER, name):
            continue
        imports[name] = prefix


def use_map(module: str, lines: list[str]) -> dict[str, dict[str, str]]:
    """Where each name a file imports comes from, per module scope.

    A key of the layer is derived from the file that writes it, so an `impl`
    for a type declared elsewhere would otherwise name the wrong module. The
    file's own `use` list is what repairs it, and this is that list reduced to
    the one question asked of it: given a bare type name, which module declares
    it?

    **The map is per scope and never per file.** An inline `mod tests` routinely
    writes `use super::{Thing, …}` for the very names the module around it
    declares, and a file-wide map would let that import decide how the
    surrounding module's own `impl` blocks resolve — which is one module too
    high, silently, for every name the test module re-imports.

    Only `crate::`, `super::`, `self::` and a module the file itself declares
    are resolved. An import from another crate names no node of this graph and
    is deliberately left unresolved.
    """
    stripper = Stripper()
    scopes: dict[str, dict[str, str]] = {module: {}}
    modules = [module]
    depths = [-1]
    depth = 0
    buffer = ""
    opening: str | None = None

    for raw in lines:
        cleaned = stripper.clean(raw).strip()
        if not cleaned:
            continue

        if buffer or re.match(r"^(?:pub(?:\([^)]*\))?\s+)?use\s", cleaned):
            buffer += " " + cleaned
            if ";" in buffer:
                statement = buffer.split(";", 1)[0]
                statement = re.sub(r"^\s*(?:pub(?:\([^)]*\))?\s+)?use\s+", "", statement)
                record_use(scopes.setdefault(modules[-1], {}), statement)
                buffer = ""
        else:
            declaration = ITEM_PATTERNS["mod"].match(cleaned)
            if declaration is not None and not cleaned.rstrip().endswith(";"):
                opening = f"{modules[-1]}::{declaration.group(1)}"

        for character in cleaned:
            if character == "{":
                if opening is not None:
                    modules.append(opening)
                    depths.append(depth)
                    scopes.setdefault(opening, {})
                    opening = None
                depth += 1
            elif character == "}":
                depth -= 1
                while len(modules) > 1 and depths[-1] >= depth:
                    modules.pop()
                    depths.pop()

    return scopes


def local_modules(lines: list[str]) -> dict[str, str]:
    """Modules the file declares with `mod name;`, and where each one lives.

    A `#[path = "…"]` attribute moves the file a `mod` declaration names, and
    the test crates use it to share one support module between several of them.
    Without this the shared module's symbols would be attributed to whichever
    crate imported them.
    """
    declared: dict[str, str] = {}
    pending_path: str | None = None

    for line in lines:
        stripped = line.strip()
        attribute = re.match(r'#\[path\s*=\s*"([^"]+)"\]', stripped)
        if attribute is not None:
            pending_path = attribute.group(1)
            continue
        declaration = re.match(rf"^(?:pub(?:\([^)]*\))?\s+)?mod\s+({IDENTIFIER})\s*;", stripped)
        if declaration is not None:
            declared[declaration.group(1)] = pending_path or ""
        if stripped and not stripped.startswith("#"):
            pending_path = None

    return declared


# ---------------------------------------------------------------- the scan ---


class Scan:
    """One pass over one file."""

    def __init__(self, path: str, lines: list[str]) -> None:
        self.path = path
        self.lines = lines
        self.module = module_of(path)
        self.imports = use_map(self.module, lines)
        self.declared_modules = local_modules(lines)
        self.nodes: list[Node] = []
        self.edges: list[Edge] = []
        self.unresolved: list[str] = []

    # -- name resolution ---------------------------------------------------

    def module_of_alias(self, alias: str) -> str | None:
        """The module an alias declared by this file resolves to."""
        if alias not in self.declared_modules:
            return None
        moved = self.declared_modules[alias]
        if moved:
            target = moved[: -len(".rs")] if moved.endswith(".rs") else moved
            if self.path.startswith("tests/"):
                return target
            return self.module + "::" + target.replace("/", "::")
        return self.module + "::" + alias

    def resolve(self, name: str, scope: str) -> str:
        """The key a bare type or trait name denotes, seen from `scope`."""
        prefix = self.imports.get(scope, {}).get(name)
        if prefix is None:
            return f"{scope}::{name}"

        head, _, tail = prefix.partition("::")
        if head == "crate":
            return CRATE + ("::" + tail if tail else "") + f"::{name}"
        if head == "self":
            return f"{scope}::{tail}::{name}" if tail else f"{scope}::{name}"
        if head == "super":
            parent = scope.rpartition("::")[0] or scope
            return f"{parent}::{tail}::{name}" if tail else f"{parent}::{name}"

        alias = self.module_of_alias(head)
        if alias is not None:
            return f"{alias}::{tail}::{name}" if tail else f"{alias}::{name}"

        self.unresolved.append(f"{self.path}: {prefix}::{name}")
        return f"{scope}::{name}"

    # -- the pass ----------------------------------------------------------

    def run(self) -> None:
        """Walks the file once, recording every declaration it passes."""
        stripper = Stripper()
        stack = [Frame(kind="module", depth=-1, module=self.module)]
        depth = 0
        pending: tuple[str, dict[str, object]] | None = None
        attributes: list[str] = []
        impl_header = ""

        for number, raw in enumerate(self.lines, start=1):
            cleaned = stripper.clean(raw).strip()

            if raw.lstrip().startswith("#["):
                attributes.append(raw.strip())
                continue

            if impl_header:
                impl_header += " " + cleaned
                if "{" in cleaned:
                    frame = self.impl_frame(impl_header, stack[-1].module)
                    pending = ("frame", {"frame": frame})
                    impl_header = ""
            elif cleaned:
                item = self.item(cleaned, number, stack, attributes)
                if item is not None:
                    kind, payload = item
                    if kind == "impl-header":
                        header = str(payload["header"])
                        if "{" in header:
                            frame = self.impl_frame(header, stack[-1].module)
                            pending = ("frame", {"frame": frame})
                        else:
                            impl_header = header
                    else:
                        pending = (kind, payload)

            if cleaned and not cleaned.startswith("#"):
                attributes = []

            depth, stack, pending = self.braces(cleaned, depth, stack, pending)

    def braces(
        self,
        cleaned: str,
        depth: int,
        stack: list[Frame],
        pending: tuple[str, dict[str, object]] | None,
    ) -> tuple[int, list[Frame], tuple[str, dict[str, object]] | None]:
        """Applies one line's braces, opening and closing frames as they fall."""
        for character in cleaned:
            if character == "{":
                if pending is not None and pending[0] == "frame":
                    frame = pending[1]["frame"]
                    assert isinstance(frame, Frame)
                    frame.depth = depth
                    stack.append(frame)
                    pending = None
                else:
                    stack.append(Frame(kind="other", depth=depth, module=stack[-1].module))
                depth += 1
            elif character == "}":
                depth -= 1
                while len(stack) > 1 and stack[-1].depth >= depth:
                    stack.pop()

        return depth, stack, pending

    def impl_frame(self, header: str, scope: str) -> Frame:
        """The frame an `impl` block opens, with the type and trait it names."""
        body = header.split("{", 1)[0]
        body = re.sub(r"^impl\s*", "", body)
        body = re.sub(r"\bwhere\b.*$", "", body)
        body = strip_generics(body).strip()

        parts = re.split(r"\bfor\b", body, maxsplit=1)
        trait_name = last_segment(parts[0]) if len(parts) == 2 else ""
        type_part = parts[-1]

        type_name = last_segment(type_part)
        owner = self.resolve(type_name, scope) if type_name else ""
        trait_key = self.resolve(trait_name, scope) if trait_name else ""

        return Frame(kind="impl", depth=0, module=scope, owner=owner, trait_key=trait_key)

    def item(
        self,
        cleaned: str,
        number: int,
        stack: list[Frame],
        attributes: list[str],
    ) -> tuple[str, dict[str, object]] | None:
        """Recognises one declaration, records it, and says what block it opens."""
        frame = stack[-1]

        if frame.kind == "enum":
            variant = VARIANT.match(cleaned)
            if variant is not None and variant.group(1)[0].isupper():
                name = variant.group(1)
                self.nodes.append(Node("Variant", f"{frame.owner}::{name}", {
                    "name": name, "owner": frame.owner,
                    "file": self.path, "line": number,
                }))
                self.edges.append(Edge("MEMBER_OF", "Variant", f"{frame.owner}::{name}",
                                       "Type", frame.owner))
            return None

        if frame.kind not in {"module", "impl"}:
            return None

        for kind in ITEM_ORDER:
            match = ITEM_PATTERNS[kind].match(cleaned)
            if match is None:
                continue
            if kind == "impl":
                if frame.kind != "module":
                    return None
                return ("impl-header", {"header": cleaned})
            name = match.group(1)
            return self.declare(kind, name, cleaned, number, frame, attributes)

        return None

    def declare(
        self,
        kind: str,
        name: str,
        cleaned: str,
        number: int,
        frame: Frame,
        attributes: list[str],
    ) -> tuple[str, dict[str, object]] | None:
        """Emits the node a recognised declaration stands for."""
        visibility = visibility_of(cleaned)
        opens = "{" in cleaned or not cleaned.rstrip().endswith(";")

        if kind == "mod":
            if frame.kind != "module" or cleaned.rstrip().endswith(";"):
                return None
            child = Frame(kind="module", depth=0, module=f"{frame.module}::{name}")
            return ("frame", {"frame": child})

        if kind == "fn":
            is_test = any(attribute.startswith("#[test]") for attribute in attributes)
            is_bench = any(attribute.startswith("#[bench]") for attribute in attributes)
            if is_test or is_bench:
                key = f"{self.path}::{name}"
                self.nodes.append(Node("Test", key, {
                    "name": name,
                    "kind": "bench" if is_bench else
                            ("integration" if self.path.startswith("tests/") else "unit"),
                    "file": self.path, "line": number,
                }))
                requirement = requirement_of(name)
                if requirement is not None:
                    self.edges.append(
                        Edge("VERIFIES", "Test", key, "Requirement", requirement))
            elif frame.kind == "impl":
                key = f"{frame.owner}::{name}"
                properties: dict[str, object] = {
                    "name": name, "owner": frame.owner,
                    "visibility": visibility, "file": self.path, "line": number,
                }
                if frame.trait_key:
                    properties["traitKey"] = frame.trait_key
                self.nodes.append(Node("Method", key, properties))
                self.edges.append(Edge("MEMBER_OF", "Method", key, "Type", frame.owner))
            else:
                key = f"{frame.module}::{name}"
                self.nodes.append(Node("Function", key, {
                    "name": name, "module": frame.module,
                    "visibility": visibility, "file": self.path, "line": number,
                }))
            return ("frame", {"frame": Frame(kind="fn", depth=0, module=frame.module)}) \
                if opens else None

        if kind in {"struct", "enum", "union", "alias"}:
            if frame.kind != "module":
                return None
            key = f"{frame.module}::{name}"
            self.nodes.append(Node("Type", key, {
                "name": name, "module": frame.module, "kind": kind,
                "visibility": visibility, "file": self.path, "line": number,
            }))
            if kind == "enum":
                return ("frame", {"frame": Frame(
                    kind="enum", depth=0, module=frame.module, owner=key)})
            return None

        if kind == "trait":
            if frame.kind != "module":
                return None
            key = f"{frame.module}::{name}"
            self.nodes.append(Node("Trait", key, {
                "name": name, "module": frame.module,
                "visibility": visibility, "file": self.path, "line": number,
            }))
            return ("frame", {"frame": Frame(kind="trait", depth=0, module=frame.module)})

        if kind in {"const", "static"}:
            owner = frame.owner if frame.kind == "impl" else ""
            key = f"{owner or frame.module}::{name}"
            properties = {
                "name": name, "module": frame.module, "kind": kind,
                "visibility": visibility, "file": self.path, "line": number,
            }
            if owner:
                properties["owner"] = owner
            self.nodes.append(Node("Const", key, properties))
            return None

        return None


def strip_generics(text: str) -> str:
    """The text with every balanced `<…>` group removed."""
    out: list[str] = []
    depth = 0
    for character in text:
        if character == "<":
            depth += 1
        elif character == ">":
            depth = max(0, depth - 1)
        elif depth == 0:
            out.append(character)
    return "".join(out)


def last_segment(text: str) -> str:
    """The bare type name a path denotes, or the empty string."""
    cleaned = text.replace("&", " ").replace("dyn ", " ").replace("mut ", " ").strip()
    cleaned = cleaned.split()[-1] if cleaned.split() else ""
    cleaned = cleaned.rpartition("::")[2]
    return cleaned if re.fullmatch(IDENTIFIER, cleaned) else ""


# ------------------------------------------------------------- the assembly ---


def scan_tree(root: Path) -> tuple[Layer, list[str]]:
    """Every node and edge the tree yields, and the names left unresolved."""
    layer = Layer()
    unresolved: list[str] = []
    types: set[str] = set()
    traits: set[str] = set()
    impls: list[tuple[str, str]] = []

    for path in sorted(files_of(root)):
        relative = path.relative_to(root).as_posix()
        lines = path.read_text(encoding="utf-8").splitlines()

        component = component_of(relative)
        if component is not None:
            name, kind = component
            layer.nodes.append(Node("Component", name, {
                "path": relative, "kind": kind,
            }))
            layer.edges.append(Edge("IMPLEMENTED_BY", "Component", name, "File", relative))

        scan = Scan(relative, lines)
        scan.run()
        unresolved.extend(scan.unresolved)

        for node in scan.nodes:
            layer.nodes.append(node)
            layer.edges.append(Edge(
                "DECLARES", "File", relative, node.label, node.key,
                {"line": node.properties["line"]},
            ))
            if node.label == "Type":
                types.add(node.key)
            elif node.label == "Trait":
                traits.add(node.key)
            if node.label == "Method" and "traitKey" in node.properties:
                impls.append((str(node.properties["owner"]), str(node.properties["traitKey"])))

        layer.edges.extend(scan.edges)

    # An edge is written only where this crate declares both endpoints. A
    # `MEMBER_OF` to a foreign type and an `IMPLEMENTS` of a foreign trait name
    # no node, so the absence of the edge is not the absence of the fact —
    # `knowledge-model.md` says so for both.
    layer.edges = [
        edge for edge in layer.edges
        if not (edge.predicate == "MEMBER_OF" and edge.target_key not in types)
    ]
    for owner, trait_key in sorted(set(impls)):
        if owner in types and trait_key in traits:
            layer.edges.append(Edge("IMPLEMENTS", "Type", owner, "Trait", trait_key))

    return layer, unresolved


def files_of(root: Path) -> list[Path]:
    """Every `.rs` file under the scanned directories."""
    found: list[Path] = []
    for directory in SCANNED:
        found.extend((root / directory).rglob("*.rs"))
    return found


def provenance(root: Path) -> tuple[str, str, bool]:
    """The commit and date every element is stamped with, and whether it is dirty."""
    def git(*arguments: str) -> str:
        return subprocess.run(
            ["git", *arguments], cwd=root, check=True,
            capture_output=True, text=True,
        ).stdout.strip()

    commit = git("rev-parse", "HEAD")
    date = git("show", "-s", "--format=%cs", "HEAD")
    dirty = bool(git("status", "--porcelain", "--", *SCANNED))

    return commit, date, dirty


# -------------------------------------------------------------- the output ---


def cypher_literal(value: object) -> str:
    """One Cypher literal: an integer bare, anything else a quoted string."""
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    text = str(value).replace("\\", "\\\\").replace("'", "\\'")
    return f"'{text}'"


IDENTITY = {
    "Component": "name",
    "File": "path",
    "Requirement": "id",
}


def identity_of(label: str) -> str:
    """The identity property of a label, per `knowledge-model.md`."""
    return IDENTITY.get(label, "key")


def emit_cypher(layer: Layer, commit: str, date: str) -> list[str]:
    """One statement per line, with `//` comments between the sections."""
    out: list[str] = [
        "// The Rust layer of the knowledge graph, re-derived from the tree by",
        "// scripts/knowledge/rust-layer.py. One statement per line; a line that",
        "// starts with // is a comment and an empty line separates the sections.",
        "//",
        "// Nodes are upserted. Every edge binds both endpoints with MATCH before it",
        "// MERGEs the relationship, which is what knowledge-model.md requires: a",
        "// pattern MERGE re-creates its nodes, and a MATCH that binds nothing is a",
        "// no-op rather than a stub. A File or Requirement this layer does not own",
        "// is therefore never created here.",
        f"// Stamped {commit} ({date}).",
        "",
        "// --- nodes ---",
    ]

    for node in layer.nodes:
        identity = identity_of(node.label)
        assignments = ", ".join(
            f"n.{name} = {cypher_literal(value)}"
            for name, value in sorted(node.properties.items())
        )
        assignments += f", n.gitCommit = {cypher_literal(commit)}"
        assignments += f", n.gitDate = {cypher_literal(date)}"
        out.append(
            f"MERGE (n:{node.label} {{{identity}: {cypher_literal(node.key)}}}) "
            f"ON CREATE SET {assignments} ON MATCH SET {assignments}"
        )

    out.append("")
    out.append("// --- edges ---")

    for edge in layer.edges:
        source = identity_of(edge.source_label)
        target = identity_of(edge.target_label)
        assignments = f"e.gitCommit = {cypher_literal(commit)}, e.gitDate = {cypher_literal(date)}"
        for name, value in sorted(edge.properties.items()):
            assignments += f", e.{name} = {cypher_literal(value)}"
        out.append(
            f"MATCH (a:{edge.source_label} {{{source}: {cypher_literal(edge.source_key)}}}) "
            f"MATCH (b:{edge.target_label} {{{target}: {cypher_literal(edge.target_key)}}}) "
            f"MERGE (a)-[e:{edge.predicate}]->(b) SET {assignments}"
        )

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
    out += [f"{predicate:<14} {count}" for predicate, count in sorted(predicates.items())]
    out += ["", f"{'total':<14} {len(layer.nodes)} nodes, {len(layer.edges)} edges"]

    return out


# ------------------------------------------------------------ the verifier ---

#: What the line a record names must contain, per label and kind. These
#: patterns share no code with the scan: the scan reaches a declaration through
#: a stateful pass over the whole file, and this reaches it through one line. A
#: record the scan put on the wrong line, or attributed to the wrong name or
#: kind, fails here.
CONFIRMATION = {
    "struct": r"\bstruct\s+{name}\b",
    "enum": r"\benum\s+{name}\b",
    "union": r"\bunion\s+{name}\b",
    "alias": r"\btype\s+{name}\b",
    "const": r"\bconst\s+{name}\b",
    "static": r"\bstatic\s+(?:mut\s+)?{name}\b",
}


def verify(
    layer: Layer,
    root: Path,
    sample: int | None = None,
    seed: int = 0,
) -> tuple[int, int, list[str]]:
    """Reopens records at their `file:line` and confirms the source there.

    With no `sample` every record is read. With one, a **deterministic** sample
    of that size is read instead: the records are sorted by identity first and
    drawn with a seeded generator, so the same `--sample` and `--seed` name the
    same records on every run and a reported result can be reproduced rather
    than merely repeated.

    Returns the number read, the number available, and one line per mismatch.
    """
    cache: dict[str, list[str]] = {}
    mismatches: list[str] = []
    checked = 0

    keys = {(node.label, node.key) for node in layer.nodes}

    population = sorted(layer.nodes, key=lambda node: (node.label, node.key))
    available = len(population)
    if sample is None:
        chosen = population
    else:
        chosen = random.Random(seed).sample(population, min(sample, available))

    for node in chosen:
        if node.label == "Component":
            path = str(node.properties["path"])
            checked += 1
            if not (root / path).is_file():
                mismatches.append(f"{node.label} {node.key}: {path} is not a file")
            continue

        path = str(node.properties["file"])
        line_number = int(node.properties["line"])
        if path not in cache:
            cache[path] = (root / path).read_text(encoding="utf-8").splitlines()
        lines = cache[path]
        checked += 1

        if not 1 <= line_number <= len(lines):
            mismatches.append(f"{node.label} {node.key}: {path}:{line_number} is past the file")
            continue

        source = lines[line_number - 1]
        name = re.escape(str(node.properties["name"]))

        if node.label in {"Function", "Method", "Test"}:
            pattern = rf"\bfn\s+{name}\b"
        elif node.label == "Trait":
            pattern = rf"\btrait\s+{name}\b"
        elif node.label == "Variant":
            pattern = rf"^\s*{name}\s*(?:[,({{=]|$)"
        else:
            template = CONFIRMATION.get(str(node.properties.get("kind")))
            if template is None:
                mismatches.append(f"{node.label} {node.key}: no confirmation for its kind")
                continue
            pattern = template.format(name=name)

        if re.search(pattern, source) is None:
            mismatches.append(
                f"{node.label} {node.key}: {path}:{line_number} does not declare it — {source.strip()!r}"
            )
            continue

        declared = node.properties.get("visibility")
        if declared is not None and visibility_of(source) != declared:
            mismatches.append(
                f"{node.label} {node.key}: {path}:{line_number} is "
                f"{visibility_of(source)}, recorded as {declared}"
            )
            continue

        owner = node.properties.get("owner")
        if owner is not None and node.label == "Variant" and ("Type", owner) not in keys:
            mismatches.append(f"{node.label} {node.key}: its enum {owner} was not declared")

    # The endpoint check reads the whole layer or none of it: a sample reports
    # exactly the records it drew, and an edge is not one of them.
    if sample is None:
        for edge in layer.edges:
            if edge.source_label in {"File", "Component"}:
                continue
            if edge.target_label in {"File", "Requirement"}:
                continue
            checked += 1
            if (edge.source_label, edge.source_key) not in keys:
                mismatches.append(
                    f"{edge.predicate}: {edge.source_key} is not a node of this layer")
            elif (edge.target_label, edge.target_key) not in keys:
                mismatches.append(
                    f"{edge.predicate}: {edge.target_key} is not a node of this layer")

    return checked, available, mismatches


# ----------------------------------------------------------------- the main ---


def repository_root() -> Path:
    """The repository this script lives in, found from the script's own path."""
    return Path(__file__).resolve().parent.parent.parent


def main() -> int:
    """Scans the tree and writes what was asked for; returns the exit code."""
    parser = argparse.ArgumentParser(
        description="Re-derive the Rust layer of the knowledge graph from the tree.",
    )
    parser.add_argument("--format", choices=("cypher", "json"), default="cypher",
                        help="the shape of the emitted layer (default: cypher)")
    parser.add_argument("--summary", action="store_true",
                        help="print counts per label and predicate instead of the layer")
    parser.add_argument("--verify", action="store_true",
                        help="re-read records at their file:line and report")
    parser.add_argument("--sample", type=int,
                        help="verify this many records, drawn deterministically, "
                             "instead of all of them")
    parser.add_argument("--seed", type=int, default=0,
                        help="the seed the sample is drawn with (default: 0)")
    parser.add_argument("--commit", help="override the commit every element is stamped with")
    parser.add_argument("--date", help="override the date every element is stamped with")
    arguments = parser.parse_args()

    root = repository_root()
    layer, unresolved = scan_tree(root)

    if arguments.verify:
        checked, available, mismatches = verify(layer, root, arguments.sample, arguments.seed)
        for mismatch in mismatches:
            print(f"MISMATCH {mismatch}")
        if arguments.sample is None:
            print(f"checked {checked} records at file:line; {len(mismatches)} mismatched")
        else:
            print(
                f"sampled {checked} of {available} records at file:line "
                f"with seed {arguments.seed}; {len(mismatches)} mismatched"
            )
        if unresolved:
            print(f"{len(unresolved)} imported names could not be resolved to a module:")
            for name in sorted(set(unresolved)):
                print(f"  {name}")
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
            f"rust-layer.py: the scanned tree has uncommitted changes; {commit} "
            "stamps a state that is not what was read",
            file=sys.stderr,
        )

    if arguments.format == "json":
        print(emit_json(layer, commit, date))
    else:
        print("\n".join(emit_cypher(layer, commit, date)))

    return 0


if __name__ == "__main__":
    sys.exit(main())
