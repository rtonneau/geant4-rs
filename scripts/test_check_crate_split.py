"""Mutation tests for the exact crate-ownership snapshot comparator."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import check_crate_split as checker


class SnapshotTests(unittest.TestCase):
    def setUp(self):
        self.revision = checker.command(["git", "rev-parse", "HEAD"])
        self.baseline = {
            "schema": checker.SCHEMA,
            "metadata": {"revision": self.revision, "package": "g4-sim"},
            "inputs": {"Cargo.lock": "abc", "data/MANIFEST.toml": "def"},
            "toolchain": {"rustc": "same compiler"},
            "runs": [{"energy_ev": "0x4059000000000000", "species": ["H3Op", "°OH"],
                      "species_csv_header": "species,x_nm,y_nm,z_nm,time_ps"}],
        }
        self.candidate = copy.deepcopy(self.baseline)

    def test_equal_pair(self):
        checker.compare(self.baseline, self.candidate)

    def test_checkout_validated_revision_change(self):
        self.baseline["metadata"]["revision"] = checker.command(["git", "rev-parse", "HEAD~1"])
        checker.compare(self.baseline, self.candidate)

    def test_one_bit_change(self):
        self.candidate["runs"][0]["energy_ev"] = "0x4059000000000001"
        with self.assertRaisesRegex(ValueError, "energy_ev"):
            checker.compare(self.baseline, self.candidate)

    def test_species_order_change(self):
        self.candidate["runs"][0]["species"].reverse()
        with self.assertRaisesRegex(ValueError, "species"):
            checker.compare(self.baseline, self.candidate)

    def test_header_change(self):
        self.candidate["runs"][0]["species_csv_header"] += ",extra"
        with self.assertRaisesRegex(ValueError, "header"):
            checker.compare(self.baseline, self.candidate)

    def test_input_changes(self):
        for name in self.baseline["inputs"]:
            candidate = copy.deepcopy(self.baseline)
            candidate["inputs"][name] += "changed"
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, "inputs"):
                checker.compare(self.baseline, candidate)

    def test_toolchain_change(self):
        self.candidate["toolchain"]["rustc"] = "other compiler"
        with self.assertRaisesRegex(ValueError, "toolchain"):
            checker.compare(self.baseline, self.candidate)

    def test_unvalidated_revision_rejected(self):
        self.candidate["metadata"]["revision"] = "0" * 40
        with self.assertRaises((ValueError, checker.subprocess.CalledProcessError)):
            checker.compare(self.baseline, self.candidate)

    def test_unlisted_metadata_is_not_ignored(self):
        self.candidate["metadata"]["anything"] = "changed"
        with self.assertRaisesRegex(ValueError, "metadata"):
            checker.compare(self.baseline, self.candidate)

    def test_revision_validation_runs_even_for_equal_pair(self):
        with patch.object(checker, "validate_revision", side_effect=ValueError("invalid checkout")):
            with self.assertRaisesRegex(ValueError, "invalid checkout"):
                checker.compare(self.baseline, self.candidate)


OLD_LOCK = '''version = 4

[[package]]
name = "dna-core"
version = "0.1.0"

[[package]]
name = "dna-data"
version = "0.1.0"
dependencies = ["dna-core"]

[[package]]
name = "dna-physics"
version = "0.1.0"
dependencies = ["dna-core", "dna-data", "upstream"]

[[package]]
name = "dna-chemistry"
version = "0.1.0"
dependencies = ["dna-core"]

[[package]]
name = "dna-sim"
version = "0.1.0"
dependencies = ["dna-chemistry", "dna-core", "dna-data", "dna-physics"]

[[package]]
name = "upstream"
version = "1.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0123456789abcdef"
'''

NEW_LOCK = (OLD_LOCK.replace("dna-core", "g4-core")
            .replace("dna-data", "g4-data")
            .replace("dna-physics", "g4-physics")
            .replace("dna-chemistry", "g4-chemistry")
            .replace("dna-sim", "g4-sim"))


class RenameSnapshotTests(unittest.TestCase):
    def setUp(self):
        scratch = checker.SCRATCH
        scratch.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.git("init", "-q")
        self.git("config", "core.autocrlf", "false")
        self.git("config", "user.name", "Snapshot Test")
        self.git("config", "user.email", "snapshot@example.invalid")
        self.old_revision = self.commit_lock(OLD_LOCK)
        self.new_revision = self.commit_lock(NEW_LOCK)
        self.baseline = self.snapshot(self.old_revision, "dna-sim", OLD_LOCK)
        self.candidate = self.snapshot(self.new_revision, "g4-sim", NEW_LOCK)

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, text=True).strip()

    def commit_lock(self, contents):
        (self.root / "Cargo.lock").write_text(contents, encoding="utf-8", newline="\n")
        self.git("add", "Cargo.lock")
        self.git("commit", "-qm", "lock fixture", "--allow-empty")
        return self.git("rev-parse", "HEAD")

    def snapshot(self, revision, package, lock):
        return {
            "schema": checker.SCHEMA,
            "metadata": {"revision": revision, "package": package},
            "inputs": {"Cargo.lock": hashlib.sha256(lock.encode()).hexdigest(),
                       "data/MANIFEST.toml": "unchanged"},
            "toolchain": {"rustc": "same compiler"},
            "runs": [{"energy_ev": "0x4059000000000000", "species": ["H3Op", "OH"],
                      "species_csv_header": "species,x_nm,y_nm,z_nm,time_ps"}],
        }

    def test_only_planned_rename_passes(self):
        checker.compare(self.baseline, self.candidate, root=self.root, allow_crate_rename=True)

    def test_windows_checkout_hash_matches_committed_lock(self):
        # Captures hash checkout bytes; Git stores the same text with LF endings.
        self.baseline["inputs"]["Cargo.lock"] = hashlib.sha256(
            OLD_LOCK.replace("\n", "\r\n").encode()).hexdigest()
        checker.compare(self.baseline, self.candidate, root=self.root, allow_crate_rename=True)

    def test_strict_mode_still_rejects_rename(self):
        with self.assertRaises(ValueError):
            checker.compare(self.baseline, self.candidate, root=self.root)

    def test_rename_mode_rejects_wrong_snapshot_package(self):
        self.candidate["metadata"]["package"] = "g4-physics"
        with self.assertRaisesRegex(ValueError, "package"):
            checker.compare(self.baseline, self.candidate, root=self.root, allow_crate_rename=True)

    def test_rename_mode_rejects_unverified_lock_hash(self):
        for document in (self.baseline, self.candidate):
            with self.subTest(package=document["metadata"]["package"]):
                recorded = document["inputs"]["Cargo.lock"]
                document["inputs"]["Cargo.lock"] = "0" * 64
                with self.assertRaisesRegex(ValueError, "Cargo.lock"):
                    checker.compare(self.baseline, self.candidate, root=self.root, allow_crate_rename=True)
                document["inputs"]["Cargo.lock"] = recorded

    def test_rename_mode_rejects_invalid_revision(self):
        self.candidate["metadata"]["revision"] = "0" * 40
        with self.assertRaises((ValueError, subprocess.CalledProcessError)):
            checker.compare(self.baseline, self.candidate, root=self.root, allow_crate_rename=True)

    def test_rename_mode_rejects_lockfile_mutations(self):
        mutations = {
            "third-party version": NEW_LOCK.replace('version = "1.2.3"', 'version = "1.2.4"'),
            "third-party source": NEW_LOCK.replace("crates.io-index", "other-index"),
            "third-party checksum": NEW_LOCK.replace("0123456789abcdef", "0123456789abcdee"),
            "local version": NEW_LOCK.replace('name = "g4-core"\nversion = "0.1.0"',
                                               'name = "g4-core"\nversion = "0.2.0"'),
            "unexpected local rename": NEW_LOCK.replace('name = "g4-data"', 'name = "other-data"'),
            "unexpected dependency": NEW_LOCK.replace('dependencies = ["g4-core"]',
                                                       'dependencies = ["upstream"]', 1),
            "reordered local dependencies": NEW_LOCK.replace(
                'dependencies = ["g4-core", "g4-data", "upstream"]',
                'dependencies = ["g4-data", "g4-core", "upstream"]'),
        }
        for name, lock in mutations.items():
            with self.subTest(name=name):
                revision = self.commit_lock(lock)
                candidate = self.snapshot(revision, "g4-sim", lock)
                with self.assertRaises(ValueError):
                    checker.compare(self.baseline, candidate, root=self.root, allow_crate_rename=True)

    def test_rename_mode_keeps_science_and_inputs_exact(self):
        mutations = (
            ("one bit", lambda c: c["runs"][0].update(energy_ev="0x4059000000000001")),
            ("data", lambda c: c["inputs"].update({"data/MANIFEST.toml": "changed"})),
            ("schema", lambda c: c.update(schema="other")),
            ("csv", lambda c: c["runs"][0].update(species_csv_header="changed")),
            ("species order", lambda c: c["runs"][0]["species"].reverse()),
            ("toolchain", lambda c: c["toolchain"].update(rustc="other")),
            ("metadata", lambda c: c["metadata"].update(extra="field")),
        )
        for name, mutate in mutations:
            with self.subTest(name=name):
                candidate = copy.deepcopy(self.candidate)
                mutate(candidate)
                with self.assertRaises(ValueError):
                    checker.compare(self.baseline, candidate, root=self.root, allow_crate_rename=True)


class GraphTests(unittest.TestCase):
    def test_valid_graph_and_disconnected_member(self):
        graph = {"g4-physics": {"g4-core"}, "g4-core": set(), "g4-chemistry": {"g4-physics"}}
        self.assertEqual(checker.check_reachable("g4-physics", graph, {"g4-chemistry"}),
                         {"g4-physics", "g4-core"})

    def test_direct_and_transitive_forbidden_edges(self):
        for middle in ("g4-chemistry", "third-party"):
            graph = {"g4-physics": {middle}, "third-party": {"g4-chemistry"}, "g4-chemistry": set()}
            with self.subTest(middle=middle), self.assertRaisesRegex(ValueError, "forbidden dependency"):
                checker.check_reachable("g4-physics", graph, {"g4-chemistry"})

    def test_default_and_no_default_sim(self):
        graph = {"g4-sim": {"g4-physics", "g4-chemistry"}, "g4-physics": set(), "g4-chemistry": set()}
        checker.check_reachable("g4-sim", graph, set())
        with self.assertRaisesRegex(ValueError, "g4-chemistry"):
            checker.check_reachable("g4-sim", graph, {"g4-chemistry"})
        graph["g4-sim"].remove("g4-chemistry")
        checker.check_reachable("g4-sim", graph, {"g4-chemistry"})

    def test_tree_parser_retains_transitive_edges(self):
        root, graph = checker.resolved_tree("0g4-physics v0.1.0 (local)\n1helper v1.0.0\n2g4-chemistry v0.1.0 (local)")
        with self.assertRaisesRegex(ValueError, "helper.*g4-chemistry"):
            checker.check_reachable(root, graph, {"g4-chemistry"})

    def test_malformed_or_incomplete_graph_rejected(self):
        for output in ("", "garbage", "0root v1\n2child v1", "0root v1\n0other v1"):
            with self.subTest(output=output), self.assertRaises(ValueError):
                checker.resolved_tree(output)
        with self.assertRaisesRegex(ValueError, "missing resolved node"):
            checker.check_reachable("root", {"root": {"missing"}}, set())


class CaptureTests(unittest.TestCase):
    def test_profiles_and_exclusive_directory(self):
        checker.SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=checker.SCRATCH) as temp:
            root = Path(temp)
            revision = "a" * 40
            for no_default in (False, True):
                directory = root / "target/crate-split-baseline" / (revision + "-" + str(no_default))
                def run(args, **kwargs):
                    if args[0] == "git":
                        return subprocess.CompletedProcess(args, 0)
                    self.assertEqual("--no-default-features" in args, no_default)
                    profile = args[-1][2:]
                    return subprocess.CompletedProcess(args, 0, json.dumps({
                        "schema": checker.SCHEMA, "profile": profile, "runs": [{}, {}, {}]}))
                with patch.object(checker, "command", return_value=revision), \
                     patch.object(checker, "input_identity", return_value={"input": "hash"}), \
                     patch.object(checker, "toolchain_identity", return_value={"tool": "same"}), \
                     patch.object(checker.subprocess, "run", side_effect=run):
                    checker.capture(directory, "g4-sim", root=root, no_default_features=no_default)
                    expected = {"physics.json"} if no_default else {"physics.json", "physchem.json"}
                    self.assertEqual({p.name for p in directory.iterdir()}, expected)
                    before = {p.name: p.read_bytes() for p in directory.iterdir()}
                    with self.assertRaises(FileExistsError):
                        checker.capture(directory, "g4-sim", root=root, no_default_features=no_default)
                    self.assertEqual(before, {p.name: p.read_bytes() for p in directory.iterdir()})


if __name__ == "__main__":
    unittest.main()
