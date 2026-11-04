import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent
GENERATOR = REPO / "tools" / "gen_plugin_events.py"
DUMP = Path(os.environ.get("RSPIGOT_API_DUMP", REPO.parent / "re" / "api-dump")) / "event-types.json"
PROTO = "crates/rspigot-plugin-protocol/proto/rspigot/plugin/v1"
INPUTS = [
    "tools/plugin-events/catalog.json",
    "tools/plugin-events/overrides.toml",
    f"{PROTO}/types.proto",
    f"{PROTO}/plugin.proto",
    "crates/rspigot-plugin/src/lib.rs",
]
OUTPUTS = [
    f"{PROTO}/events.proto",
    "crates/rspigot-core/src/plugins/events.rs",
    "crates/rspigot-plugins/src/convert/events.rs",
    "crates/rspigot-plugin/src/events.rs",
    "crates/rspigot-plugin-macros/src/events.rs",
    "crates/rspigot-plugin/EVENTS.md",
    "crates/rspigot-plugin/tests/every_event.rs",
]


def generator(root, *arguments):
    return subprocess.run(
        [sys.executable, "-I", str(GENERATOR), "--root", str(root), *arguments],
        capture_output=True, text=True,
    )


class Generator(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.root = Path(self.scratch.name)
        for relative in INPUTS + OUTPUTS:
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPO / relative, target)

    def tearDown(self):
        self.scratch.cleanup()

    def catalog(self):
        return json.loads((self.root / INPUTS[0]).read_text())

    def write_catalog(self, catalog):
        (self.root / INPUTS[0]).write_text(json.dumps(catalog))

    def outputs(self):
        return {relative: (self.root / relative).read_bytes() for relative in OUTPUTS}

    def test_the_committed_files_are_fresh_and_two_runs_write_the_same_bytes(self):
        self.assertEqual(generator(self.root, "--check").returncode, 0)
        committed = self.outputs()
        self.assertEqual(generator(self.root).returncode, 0)
        first = self.outputs()
        self.assertEqual(generator(self.root).returncode, 0)
        self.assertEqual(self.outputs(), first)
        self.assertEqual(first, committed)

    def test_check_fails_after_a_manual_edit_of_any_generated_file(self):
        for relative in OUTPUTS:
            path = self.root / relative
            original = path.read_text()
            path.write_text(original + " ")
            failed = generator(self.root, "--check")
            self.assertEqual(failed.returncode, 1, relative)
            self.assertIn(relative, failed.stderr)
            path.write_text(original)
        self.assertEqual(generator(self.root, "--check").returncode, 0)

    def test_an_unknown_java_type_fails_the_generation(self):
        catalog = self.catalog()
        event = next(event for event in catalog["events"] if event["name"] == "PlayerJoin")
        event["getters"].append(
            {"name": "getMystery", "declared_in": event["java"], "type": "org.example.Mystery", "nullable": None, "key": "mystery"}
        )
        event["fields"]["mystery"] = 99
        self.write_catalog(catalog)
        failed = generator(self.root, "--check")
        self.assertEqual(failed.returncode, 1)
        self.assertIn("org.example.Mystery", failed.stderr)

    def test_an_event_without_a_state_fails_the_generation(self):
        overrides = self.root / INPUTS[1]
        overrides.write_text(overrides.read_text().replace('"ChunkLoad", ', ""))
        failed = generator(self.root, "--check")
        self.assertEqual(failed.returncode, 1)
        self.assertIn("ChunkLoad must have exactly one state", failed.stderr)

    @unittest.skipUnless(DUMP.exists(), "the API dump is not on this machine")
    def test_check_dump_passes_on_the_api_dump_and_fails_when_the_catalog_lacks_an_event(self):
        self.assertEqual(generator(self.root, "--check-dump", str(DUMP)).returncode, 0)
        catalog = self.catalog()
        catalog["events"] = [event for event in catalog["events"] if event["name"] != "EntityDamageByEntity"]
        self.write_catalog(catalog)
        failed = generator(self.root, "--check-dump", str(DUMP))
        self.assertEqual(failed.returncode, 1)
        self.assertIn("org.bukkit.event.entity.EntityDamageByEntityEvent", failed.stderr)


if __name__ == "__main__":
    unittest.main()
