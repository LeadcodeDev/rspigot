import argparse
import json
import os
import re
import sys

RE_ROOT = "/Users/baptiste.parmantier/Development/rspigot/re"
DEFAULT_REGISTRIES = RE_ROOT + "/datagen/generated/reports/registries.json"
DEFAULT_ENCHANTMENTS = RE_ROOT + "/server/data/minecraft/enchantment"
DEFAULT_PARTICLE_TYPES = RE_ROOT + "/ghidra-out/net/minecraft/core/particles/ParticleTypes.c"
REPO = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
DEFAULT_OUT = os.path.join(REPO, "crates", "rspigot-plugin", "src", "keys")
DEFAULT_CORE_PARTICLES = os.path.join(REPO, "crates", "rspigot-domain", "src", "particle.rs")
NAMESPACE = "minecraft:"

REGISTRY_LISTS = [
    ("Material", "material", "minecraft:item", 1658),
    ("BlockType", "block_type", "minecraft:block", 1286),
    ("EffectType", "effect_type", "minecraft:mob_effect", 40),
    ("EntityType", "entity_type", "minecraft:entity_type", 161),
    ("Sound", "sound", "minecraft:sound_event", 1991),
]
ENCHANTMENTS = 43
PARTICLE_REGISTRY = "minecraft:particle_type"
PARTICLE_TYPES = 128
DATALESS_PARTICLES = 106
CODEC_PARTICLES = 22
PARTICLE_DATA = [
    ("item", "(Material)", "(_)"),
    ("block", "(BlockType)", "(_)"),
    ("block_marker", "(BlockType)", "(_)"),
    ("falling_dust", "(BlockType)", "(_)"),
    ("dust_pillar", "(BlockType)", "(_)"),
    ("block_crumble", "(BlockType)", "(_)"),
    ("dust", " { color: Color, scale: f32 }", " { .. }"),
]
BLOCK_DATA = "(BlockType)"

KEY_PATH = re.compile(r"^[a-z0-9_.]+$")
VARIANT_NAME = re.compile(r"^[A-Z][A-Za-z0-9]*$")
REGISTER = re.compile(r'\(\*ParticleTypes_register\)\("([^"]*)",(true|false)(,[^)]*)?\)')
CORE_LIST = r"pub const %s: \[&str; [0-9]+\] = \[(.*?)\];"
STRING = re.compile(r'"([^"]*)"')


def fail(message):
    sys.exit("gen_plugin_keys: " + message)


def load_json(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def read_text(path):
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def strip_namespace(key):
    if not key.startswith(NAMESPACE):
        fail("%s is not in the %s namespace" % (key, NAMESPACE))
    return key[len(NAMESPACE):]


def variant_name(path):
    parts = re.split(r"[._]", path)
    if any(not part for part in parts):
        fail("%s has an empty name part" % path)
    return "".join(part[:1].upper() + part[1:] for part in parts)


def keys_of(paths, enum):
    keys = []
    seen = {}
    for path in paths:
        if not KEY_PATH.match(path):
            fail("%s key %r has unexpected characters" % (enum, path))
        name = variant_name(path)
        if not VARIANT_NAME.match(name):
            fail("%s variant %s of %s does not start with a letter" % (enum, name, path))
        if name in seen:
            fail("%s variant %s comes from both %s and %s" % (enum, name, seen[name], path))
        seen[name] = path
        keys.append((name, path))
    return keys


def registry_paths(registries, registry, expected):
    entries = registries.get(registry, {}).get("entries")
    if not entries:
        fail("registries.json has no " + registry)
    ordered = sorted(entries.items(), key=lambda item: item[1]["protocol_id"])
    paths = []
    for index, (key, entry) in enumerate(ordered):
        if entry["protocol_id"] != index:
            fail("%s %s protocol id %d != %d" % (registry, key, entry["protocol_id"], index))
        paths.append(strip_namespace(key))
    if len(paths) != expected:
        fail("%s has %d entries, expected %d" % (registry, len(paths), expected))
    return paths


def enchantment_paths(directory):
    paths = sorted(
        entry[: -len(".json")]
        for entry in os.listdir(directory)
        if entry.endswith(".json") and os.path.isfile(os.path.join(directory, entry))
    )
    if len(paths) != ENCHANTMENTS:
        fail("%s has %d enchantments, expected %d" % (directory, len(paths), ENCHANTMENTS))
    return paths


def particle_kinds(registries, particle_types):
    registered = registry_paths(registries, PARTICLE_REGISTRY, PARTICLE_TYPES)
    calls = REGISTER.findall(read_text(particle_types))
    names = [name for name, _, _ in calls]
    if names != registered:
        fail("%s does not register the %s entries in protocol order" % (particle_types, PARTICLE_REGISTRY))
    dataless = [name for name, _, codec in calls if not codec]
    codec = [name for name, _, codec in calls if codec]
    if len(dataless) != DATALESS_PARTICLES:
        fail("%d particles without data, expected %d" % (len(dataless), DATALESS_PARTICLES))
    if len(codec) != CODEC_PARTICLES:
        fail("%d particles with a codec, expected %d" % (len(codec), CODEC_PARTICLES))
    for path, _, _ in PARTICLE_DATA:
        if path not in codec:
            fail("particle %s is not registered with a codec" % path)
    return dataless, codec


def core_list(source, name, path):
    found = re.search(CORE_LIST % name, source, re.DOTALL)
    if not found:
        fail("%s has no %s" % (path, name))
    return sorted(strip_namespace(key) for key in STRING.findall(found.group(1)))


def check_core_particles(path, codec):
    source = read_text(path)
    with_data = core_list(source, "PARTICLES_WITH_DATA", path)
    if with_data != sorted(codec):
        fail("%s PARTICLES_WITH_DATA differs from the particles registered with a codec" % path)
    blocks = core_list(source, "BLOCK_PARTICLES", path)
    sdk_blocks = sorted(kind for kind, data, _ in PARTICLE_DATA if data == BLOCK_DATA)
    if blocks != sdk_blocks:
        fail("%s BLOCK_PARTICLES %s differs from the SDK block particles %s" % (path, blocks, sdk_blocks))


def lookup_lines(enum, keys):
    ordered = sorted(keys, key=lambda pair: pair[1])
    lines = [
        "    pub fn from_key(key: &str) -> Option<Self> {",
        '        let path = key.strip_prefix("%s").unwrap_or(key);' % NAMESPACE,
        "        let by_key = &Self::BY_KEY;",
        "        by_key",
        "            .binary_search_by(|(candidate, _)| candidate.cmp(&path))",
        "            .ok()",
        "            .map(|index| by_key[index].1)",
        "    }",
        "",
        "    const BY_KEY: [(&'static str, Self); %d] = [" % len(ordered),
    ]
    lines.extend('        ("%s", Self::%s),' % (path, name) for name, path in ordered)
    lines.append("    ];")
    return lines


def enum_file(enum, keys, derives, header=(), data=()):
    lines = list(header)
    if header:
        lines.append("")
    lines.append("#[derive(%s)]" % ", ".join(derives))
    lines.append("pub enum %s {" % enum)
    lines.extend("    %s," % name for name, _ in keys)
    lines.extend("    %s%s," % (name, fields) for name, _, fields, _ in data)
    lines.extend([
        "}",
        "",
        "impl %s {" % enum,
        "    pub const ALL: &'static [Self] = &[",
    ])
    lines.extend("        Self::%s," % name for name, _ in keys)
    lines.extend([
        "    ];",
        "",
        "    pub const fn key(self) -> &'static str {",
        "        match self {",
    ])
    lines.extend('            Self::%s => "%s%s",' % (name, NAMESPACE, path) for name, path in keys)
    lines.extend(
        '            Self::%s%s => "%s%s",' % (name, pattern, NAMESPACE, path)
        for name, path, _, pattern in data
    )
    lines.extend([
        "        }",
        "    }",
        "",
    ])
    lines.extend(lookup_lines(enum, keys))
    lines.append("}")
    return "\n".join(lines) + "\n"


def particle_file(dataless):
    keys = keys_of(dataless, "Particle")
    data = [(variant_name(path), path, fields, pattern) for path, fields, pattern in PARTICLE_DATA]
    names = {name for name, _ in keys}
    for name, path, _, _ in data:
        if name in names:
            fail("Particle variant %s of %s is also a particle without data" % (name, path))
    header = ["use super::{BlockType, Material};", "use crate::Color;"]
    return enum_file("Particle", keys, ["Debug", "Clone", "Copy", "PartialEq"], header, data)


def mod_file(modules):
    ordered = sorted(modules)
    lines = ["mod %s;" % module for module, _ in ordered]
    lines.append("")
    lines.extend("pub use %s::%s;" % (module, enum) for module, enum in ordered)
    return "\n".join(lines) + "\n"


def write(path, content, stats, check):
    stats["files"] += 1
    stats["bytes"] += len(content.encode("utf-8"))
    if os.path.exists(path):
        with open(path, encoding="utf-8") as handle:
            if handle.read() == content:
                return
    stats["written"] += 1
    if check:
        stats["stale"].append(path)
        return
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(content)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--registries", default=DEFAULT_REGISTRIES)
    parser.add_argument("--enchantments", default=DEFAULT_ENCHANTMENTS)
    parser.add_argument("--particle-types", default=DEFAULT_PARTICLE_TYPES)
    parser.add_argument("--core-particles", default=DEFAULT_CORE_PARTICLES)
    parser.add_argument("--out", default=DEFAULT_OUT)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    registries = load_json(args.registries)
    derives = ["Debug", "Clone", "Copy", "PartialEq", "Eq", "Hash"]
    files = {}
    counts = []
    for enum, module, registry, expected in REGISTRY_LISTS:
        keys = keys_of(registry_paths(registries, registry, expected), enum)
        files[(module, enum)] = enum_file(enum, keys, derives)
        counts.append((module, len(keys)))
    enchantments = keys_of(enchantment_paths(args.enchantments), "Enchantment")
    files[("enchantment", "Enchantment")] = enum_file("Enchantment", enchantments, derives)
    counts.append(("enchantment", len(enchantments)))
    dataless, codec = particle_kinds(registries, args.particle_types)
    check_core_particles(args.core_particles, codec)
    files[("particle", "Particle")] = particle_file(dataless)
    counts.append(("particle", len(dataless)))

    if not args.check:
        os.makedirs(args.out, exist_ok=True)
    stats = {"files": 0, "bytes": 0, "written": 0, "stale": []}
    expected_files = {"mod.rs"}
    for (module, _), content in sorted(files.items()):
        expected_files.add(module + ".rs")
        write(os.path.join(args.out, module + ".rs"), content, stats, args.check)
    write(os.path.join(args.out, "mod.rs"), mod_file(files), stats, args.check)
    removed = 0
    if os.path.isdir(args.out):
        for entry in sorted(os.listdir(args.out)):
            if entry.endswith(".rs") and entry not in expected_files:
                removed += 1
                if args.check:
                    stats["stale"].append(os.path.join(args.out, entry))
                else:
                    os.remove(os.path.join(args.out, entry))
    print(
        "%s files %d bytes %d written %d removed %d"
        % (
            " ".join("%s %d" % pair for pair in counts),
            stats["files"],
            stats["bytes"],
            stats["written"],
            removed,
        )
    )
    if args.check and stats["stale"]:
        for path in stats["stale"]:
            print("stale " + path, file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
