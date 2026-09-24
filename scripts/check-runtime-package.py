#!/usr/bin/env python3
"""Check the references agents can follow in a staged release archive."""

import argparse
from pathlib import Path
import re
import tempfile
import unittest
from urllib.parse import unquote, urlsplit


LINK = re.compile(r"\[[^\]\n]*\]\(<?([^\s)>]+)>?\)")
CODE_DOCUMENT = re.compile(r"`((?:[A-Za-z0-9_.-]+/)*[A-Za-z0-9_.-]+\.md)`")


def check_skill(skill):
    """A copied skill must not require sibling or repository resources."""
    skill = skill.resolve()
    errors = []
    if not (skill / "SKILL.md").is_file():
        errors.append(f"missing skill entrypoint: {skill.name}")
    for resource in sorted(skill.rglob("*")):
        if not resource.resolve().is_relative_to(skill):
            errors.append(f"resource escapes skill: {skill.name}/{resource.relative_to(skill)}")
            continue
        if not resource.is_file() or resource.suffix != ".md":
            continue
        for number, line in enumerate(resource.read_text(encoding="utf-8").splitlines(), 1):
            for raw in LINK.findall(line) + CODE_DOCUMENT.findall(line):
                link = urlsplit(raw)
                if link.scheme or link.netloc or not link.path:
                    continue
                target = (resource.parent / unquote(link.path)).resolve()
                label = f"{skill.name}/{resource.relative_to(skill)}:{number}"
                if not target.is_relative_to(skill):
                    errors.append(f"{label}: reference escapes standalone skill: {raw}")
                elif not target.exists():
                    errors.append(f"{label}: missing standalone reference: {raw}")
    return errors


def check_package(package, expected_skills):
    package = package.resolve()
    errors = []
    for path in package.rglob("*"):
        if path.is_symlink():
            errors.append(f"symlink in archive: {path.relative_to(package)}")
    guides = [package / name for name in ("AGENTS.md", "CLAUDE.md")]
    for guide in guides:
        if not guide.is_file():
            errors.append(f"missing guide: {guide.name}")
    if all(p.is_file() for p in guides) and guides[0].read_bytes() != guides[1].read_bytes():
        errors.append("AGENTS.md and CLAUDE.md differ")

    trees = []
    documents = [p for p in guides if p.is_file()]
    for prefix in (".agents/skills", ".claude/skills"):
        root = package / prefix
        names = {p.name for p in root.iterdir() if p.is_dir()} if root.is_dir() else set()
        if names != set(expected_skills):
            errors.append(f"{prefix}: expected {sorted(expected_skills)}, found {sorted(names)}")
        tree = {}
        for name in names:
            errors.extend(check_skill(root / name))
            if not (root / name / "SKILL.md").is_file():
                errors.append(f"missing entrypoint: {prefix}/{name}/SKILL.md")
        for path in sorted(root.rglob("*")):
            if not path.resolve().is_relative_to(package):
                errors.append(f"resource escapes archive: {path.relative_to(package)}")
                continue
            if path.is_file():
                tree[path.relative_to(root).as_posix()] = path.read_bytes()
                if path.suffix == ".md":
                    documents.append(path)
        for shared_name in ("desktop-session.md", "mcp-connection.md"):
            copies = [content for name, content in tree.items()
                      if name.endswith(f"/references/{shared_name}")]
            if copies and any(content != copies[0] for content in copies[1:]):
                errors.append(f"{prefix}: shared guidance differs: {shared_name}")
        trees.append(tree)
    if trees[0] != trees[1]:
        errors.append("Codex and Claude runtime resources differ")

    # Traverse referenced Markdown, including the packaged getting-started docs.
    # Links to the online repository are external and are never fetched.
    visited = set()
    while documents:
        document = documents.pop()
        if document in visited:
            continue
        visited.add(document)
        for number, line in enumerate(document.read_text(encoding="utf-8").splitlines(), 1):
            references = LINK.findall(line)
            # Skills must also resolve document paths written as inline code,
            # the form used by the missing reference that motivated this check.
            if document.is_relative_to(package / ".agents/skills") or document.is_relative_to(
                package / ".claude/skills"
            ):
                references += CODE_DOCUMENT.findall(line)
            for raw in references:
                link = urlsplit(raw)
                if link.scheme or link.netloc or not link.path:
                    continue
                target = (document.parent / unquote(link.path)).resolve()
                label = f"{document.relative_to(package)}:{number}"
                if not target.is_relative_to(package):
                    errors.append(f"{label}: link escapes archive: {raw}")
                elif not target.exists():
                    errors.append(f"{label}: missing reference: {raw}")
                elif target.is_file() and target.suffix == ".md":
                    documents.append(target)
    return errors


class PackageReferenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for guide in ("AGENTS.md", "CLAUDE.md"):
            (self.root / guide).write_text("[start](.agents/skills/one/SKILL.md)\n")
        self.write_skill("one", "# First workflow\n")
        self.write_skill("two", "[details](references/detail.md)\n")
        self.write_detail("# Details\n")

    def write_skill(self, name, text):
        for prefix in (".agents", ".claude"):
            path = self.root / prefix / "skills" / name / "SKILL.md"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)

    def write_detail(self, text):
        for prefix in (".agents", ".claude"):
            path = self.root / prefix / "skills/two/references/detail.md"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)

    def check(self):
        return check_package(self.root, ["one", "two"])

    def test_self_contained_skills_are_complete(self):
        self.assertEqual(self.check(), [])

    def test_archive_rejects_links_even_with_internal_targets(self):
        link = self.root / "guide-link.md"
        try:
            link.symlink_to("AGENTS.md")
        except OSError as error:
            self.skipTest(f"symlinks unavailable: {error}")
        self.assertIn("symlink in archive: guide-link.md", self.check())

    def test_sibling_reference_fails_even_when_packaged(self):
        self.write_skill("one", "[other](../two/references/detail.md)\n")
        self.assertEqual(sum("escapes standalone skill" in e for e in self.check()), 2)

    def test_repository_reference_fails_even_when_packaged(self):
        self.write_skill("one", "Read `../../../AGENTS.md`.\n")
        self.assertEqual(sum("escapes standalone skill" in e for e in self.check()), 2)

    def test_transitive_sibling_reference_fails(self):
        self.write_detail("[other](../../one/SKILL.md)\n")
        self.assertEqual(sum("escapes standalone skill" in e for e in self.check()), 2)

    def test_missing_reference_fails_in_both_roots(self):
        for prefix in (".agents", ".claude"):
            (self.root / prefix / "skills/two/references/detail.md").unlink()
        errors = self.check()
        self.assertEqual(sum("missing reference" in e for e in errors), 2)

    def test_transitive_reference_is_checked(self):
        self.write_detail("[missing](absent.md)\n")
        self.assertTrue(any("absent.md" in e for e in self.check()))

    def test_inline_code_document_cannot_hide_missing_reference(self):
        self.write_skill("one", "Read `docs/observation-workflows.md` when needed.\n")
        self.assertEqual(sum("missing reference" in e for e in self.check()), 2)

    def test_reference_outside_package_fails(self):
        self.write_skill("one", "[outside](../../../../outside.md)\n")
        self.assertTrue(any("escapes archive" in e for e in self.check()))

    def test_extra_development_skill_fails(self):
        self.write_skill("release-prep", "# Contributor only\n")
        self.assertTrue(any("release-prep" in e for e in self.check()))

    def test_shared_guidance_cannot_drift_between_independent_copies(self):
        for prefix in (".agents", ".claude"):
            for name in ("one", "two"):
                path = self.root / prefix / "skills" / name / "references/mcp-connection.md"
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(f"Different connection guidance: {name}\n")
        self.assertEqual(sum("shared guidance differs" in e for e in self.check()), 2)

    def test_copies_and_guides_must_match(self):
        (self.root / ".claude/skills/one/SKILL.md").write_text("# Drift\n")
        (self.root / "CLAUDE.md").write_text("# Drift\n")
        self.assertEqual(sum("differ" in e for e in self.check()), 2)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("package", nargs="?", type=Path)
    parser.add_argument("--skills", nargs="+")
    parser.add_argument("--skill", type=Path, help="validate an isolated skill directory")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(PackageReferenceTests)
        return 0 if unittest.TextTestRunner().run(suite).wasSuccessful() else 1
    if args.skill:
        errors = check_skill(args.skill)
        for error in errors:
            print(error)
        if not errors:
            print(f"Standalone skill references passed: {args.skill.name}")
        return int(bool(errors))
    if args.package is None or not args.skills:
        parser.error("package and --skills are required unless --self-test is used")
    errors = check_package(args.package, args.skills)
    for error in errors:
        print(error)
    if not errors:
        print(f"Runtime package references and parity passed ({len(args.skills)} skills per root).")
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
