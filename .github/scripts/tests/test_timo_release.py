import hashlib
import importlib.util
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "timo_release.py"
SPEC = importlib.util.spec_from_file_location("timo_release", SCRIPT)
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)
VERSION = "0.10.1-timo"


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "Cargo.toml").write_text(f'[package]\nversion = "{VERSION}"\n')
        (self.root / "LICENSE").write_text("MIT License\n")
        (self.root / "README.md").write_text("Serie fork\n")

    def build_fixture(self, target):
        binary = self.root / "target" / target / "release" / "serie"
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"fake release binary\n")
        return release.create_archive(self.root, VERSION, target, self.root / "artifacts")

    def test_version_and_tag_must_identify_the_same_fork_release(self):
        self.assertEqual(release.release_version(self.root), VERSION)
        release.validate_tag(f"v{VERSION}", VERSION)
        for tag in ["v0.10.1", "v0.10.1-timo.2", "v0.10.1-timo.1;echo bad"]:
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                release.validate_tag(tag, VERSION)
        (self.root / "Cargo.toml").write_text('[package]\nversion = "0.10.1"\n')
        with self.assertRaises(ValueError):
            release.release_version(self.root)

    def test_archive_has_executable_license_and_reproducible_metadata(self):
        target = release.ALL_TARGETS[0]
        archive = self.build_fixture(target)
        original = archive.read_bytes()
        with tarfile.open(archive) as package:
            self.assertEqual(package.getnames(), ["serie", "LICENSE", "README.md"])
            binary = package.getmember("serie")
            self.assertEqual(binary.mode, 0o755)
            self.assertEqual(binary.mtime, 0)
            self.assertEqual(package.extractfile("serie").read(), b"fake release binary\n")
        release.create_archive(self.root, VERSION, target, archive.parent)
        self.assertEqual(archive.read_bytes(), original)

    def test_missing_binary_and_unsupported_target_fail(self):
        with self.assertRaises(ValueError):
            release.create_archive(self.root, VERSION, release.ALL_TARGETS[0], self.root / "out")
        with self.assertRaises(ValueError):
            release.archive_name(VERSION, "unknown-target")

    def test_formula_covers_every_platform_with_actual_archive_checksums(self):
        archives = [self.build_fixture(target) for target in release.ALL_TARGETS]
        output = self.root / "serie-timo.rb"
        release.generate_formula(VERSION, archives[0].parent, output)
        formula = output.read_text()
        self.assertIn("class SerieTimo < Formula", formula)
        self.assertIn('conflicts_with "serie"', formula)
        for archive in archives:
            self.assertIn(f"/v{VERSION}/{archive.name}", formula)
            self.assertIn(hashlib.sha256(archive.read_bytes()).hexdigest(), formula)
        manifest = (archives[0].parent / "SHA256SUMS").read_text().splitlines()
        self.assertEqual(len(manifest), 4)
        for line in manifest:
            digest, name = line.split("  ")
            self.assertEqual(digest, hashlib.sha256((archives[0].parent / name).read_bytes()).hexdigest())
        result = subprocess.run(["ruby", "-c", str(output)], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_incomplete_matrix_does_not_generate_a_partial_formula(self):
        archive = self.build_fixture(release.ALL_TARGETS[0])
        output = self.root / "serie-timo.rb"
        with self.assertRaises(ValueError):
            release.generate_formula(VERSION, archive.parent, output)
        self.assertFalse(output.exists())
        self.assertFalse((archive.parent / "SHA256SUMS").exists())


if __name__ == "__main__":
    unittest.main()
