import argparse
import json
import os
import re
import subprocess
import sys

RE_ROOT = "/Users/baptiste.parmantier/Development/rspigot/re"
DEFAULT_REPORTS = RE_ROOT + "/datagen/generated/reports"
DEFAULT_DATAPACK = RE_ROOT + "/server/data/minecraft"
DEFAULT_CLASSES = RE_ROOT + "/server"
DEFAULT_JAVAP = "/opt/homebrew/opt/openjdk@25/bin/javap"
DEFAULT_OUT = os.path.normpath(
    os.path.join(
        os.path.dirname(os.path.abspath(__file__)),
        "..",
        "crates",
        "rspigot-data",
        "src",
        "generated",
    )
)

SYNCED = [
    ("BIOME", "worldgen/biome"),
    ("CHAT_TYPE", "chat_type"),
    ("TRIM_PATTERN", "trim_pattern"),
    ("TRIM_MATERIAL", "trim_material"),
    ("WOLF_VARIANT", "wolf_variant"),
    ("WOLF_SOUND_VARIANT", "wolf_sound_variant"),
    ("PIG_VARIANT", "pig_variant"),
    ("PIG_SOUND_VARIANT", "pig_sound_variant"),
    ("FROG_VARIANT", "frog_variant"),
    ("CAT_VARIANT", "cat_variant"),
    ("CAT_SOUND_VARIANT", "cat_sound_variant"),
    ("COW_SOUND_VARIANT", "cow_sound_variant"),
    ("COW_VARIANT", "cow_variant"),
    ("CHICKEN_SOUND_VARIANT", "chicken_sound_variant"),
    ("CHICKEN_VARIANT", "chicken_variant"),
    ("ZOMBIE_NAUTILUS_VARIANT", "zombie_nautilus_variant"),
    ("PAINTING_VARIANT", "painting_variant"),
    ("SULFUR_CUBE_ARCHETYPE", "sulfur_cube_archetype"),
    ("DIMENSION_TYPE", "dimension_type"),
    ("DAMAGE_TYPE", "damage_type"),
    ("BANNER_PATTERN", "banner_pattern"),
    ("ENCHANTMENT", "enchantment"),
    ("JUKEBOX_SONG", "jukebox_song"),
    ("INSTRUMENT", "instrument"),
    ("TEST_ENVIRONMENT", "test_environment"),
    ("TEST_INSTANCE", "test_instance"),
    ("DIALOG", "dialog"),
    ("WORLD_CLOCK", "world_clock"),
    ("TIMELINE", "timeline"),
    ("DECORATED_POT_PATTERN", "decorated_pot_pattern"),
    ("BLOCK_TRANSFORMER", "block_transformer"),
    ("BLOCK_STATE_PROVIDER", "worldgen/block_state_provider"),
]

ITEM_BLOCK_EXCEPTIONS = {
    "minecraft:redstone": "minecraft:redstone_wire",
    "minecraft:string": "minecraft:tripwire",
    "minecraft:powder_snow_bucket": "minecraft:powder_snow",
    "minecraft:wheat_seeds": "minecraft:wheat",
    "minecraft:cocoa_beans": "minecraft:cocoa",
    "minecraft:carrot": "minecraft:carrots",
    "minecraft:potato": "minecraft:potatoes",
    "minecraft:torchflower_seeds": "minecraft:torchflower_crop",
    "minecraft:pitcher_pod": "minecraft:pitcher_crop",
    "minecraft:beetroot_seeds": "minecraft:beetroots",
    "minecraft:sweet_berries": "minecraft:sweet_berry_bush",
    "minecraft:glow_berries": "minecraft:cave_vines",
    "minecraft:pumpkin_seeds": "minecraft:pumpkin_stem",
    "minecraft:melon_seeds": "minecraft:melon_stem",
}

ITEM_BLOCK_DENY = {"minecraft:wheat"}

AIR_BLOCKS = {
    "AIR": "minecraft:air",
    "VOID_AIR": "minecraft:void_air",
    "CAVE_AIR": "minecraft:cave_air",
}

EXPECTED_AIR_STATES = {"AIR": 0, "VOID_AIR": 18649, "CAVE_AIR": 18650}

NO_STATE = 65535
LINE_WIDTH = 100
NAME_RE = re.compile(r"^[a-z0-9_.\-/]+$")
ID_RE = re.compile(r"^[a-z0-9_.\-]+:[a-z0-9_.\-/]+$")


def fail(message):
    raise SystemExit("error: " + message)


def load_json(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def normalize_id(value):
    if ":" not in value:
        value = "minecraft:" + value
    assert ID_RE.match(value), value
    return value


def wrapped(tokens, indent):
    lines = []
    line = ""
    for token in tokens:
        piece = token + ","
        if line and len(indent) + len(line) + 1 + len(piece) > LINE_WIDTH:
            lines.append(indent + line)
            line = piece
        elif line:
            line = line + " " + piece
        else:
            line = piece
    if line:
        lines.append(indent + line)
    return lines


def quoted(values):
    for value in values:
        assert NAME_RE.match(value.replace(":", "/", 1)), value
    return ['"' + value + '"' for value in values]


def array_lines(tokens, indent, head, tail):
    if not tokens:
        return [indent + head + "&[]" + tail]
    lines = [indent + head + "&["]
    lines.extend(wrapped(tokens, indent + "    "))
    lines.append(indent + "]" + tail)
    return lines


def java_lines(javap, classes, class_name):
    result = subprocess.run(
        [javap, "-c", "-p", "-classpath", classes, class_name],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.split("\n")


def static_init_start(lines):
    for index, line in enumerate(lines):
        if line.strip() == "static {};":
            return index
    fail("static initializer not found")


def java_registry_keys(lines):
    current = None
    keys = {}
    for line in lines[static_init_start(lines) :]:
        match = re.search(r"// String (.*)$", line)
        if match:
            current = match.group(1)
            continue
        match = re.search(r"putstatic\s+#\d+\s+// Field (\w+):", line)
        if match:
            keys[match.group(1)] = current
    return keys


def java_synchronized_order(lines):
    names = []
    for line in lines[static_init_start(lines) :]:
        match = re.search(
            r"getstatic\s+#\d+\s+// Field net/minecraft/core/registries/Registries\.(\w+):",
            line,
        )
        if match:
            names.append(match.group(1))
            continue
        match = re.search(r"putstatic\s+#\d+\s+// Field (\w+):", line)
        if match:
            if match.group(1) == "SYNCHRONIZED_REGISTRIES":
                return names
            names = []
    fail("synchronized registries not found")


def verify_synced_against_java(javap, classes):
    if not os.path.exists(javap):
        print("javap not found, synced registry verification skipped", file=sys.stderr)
        return False
    keys = java_registry_keys(
        java_lines(javap, classes, "net.minecraft.core.registries.Registries")
    )
    order = java_synchronized_order(
        java_lines(javap, classes, "net.minecraft.resources.RegistryDataLoader")
    )
    expected = [constant for constant, _ in SYNCED]
    assert order == expected, (order, expected)
    for constant, path in SYNCED:
        assert keys[constant] == path, (constant, keys[constant], path)
    return True


def json_names(root):
    names = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames.sort()
        for filename in filenames:
            if filename.endswith(".json"):
                relative = os.path.relpath(os.path.join(dirpath, filename), root)
                names.append(relative[: -len(".json")].replace(os.sep, "/"))
    return sorted(names)


def build_blocks(blocks, registries):
    entries = registries["minecraft:block"]["entries"]
    assert set(entries) == set(blocks)
    order = sorted(blocks, key=lambda name: entries[name]["protocol_id"])
    assert [entries[name]["protocol_id"] for name in order] == list(range(len(order)))

    definitions = set()
    for name in order:
        for key, values in blocks[name].get("properties", {}).items():
            assert len(set(values)) == len(values), name
            definitions.add((key, tuple(values)))
    definition_list = sorted(definitions)
    definition_index = {definition: i for i, definition in enumerate(definition_list)}
    assert len(definition_list) < 256

    refs = []
    ref_cache = {}
    rows = []
    next_state = 0
    total_listed = 0
    for name in order:
        spec = blocks[name]
        assert set(spec) <= {"properties", "states"}, name
        props = spec.get("properties", {})
        keys = sorted(props)
        sizes = [len(props[key]) for key in keys]
        count = 1
        for size in sizes:
            count *= size
        states = spec["states"]
        assert len(states) == count, name
        total_listed += len(states)
        first = states[0]["id"]
        assert first == next_state, name
        seen = set()
        default = None
        for state in states:
            state_props = state.get("properties", {})
            assert sorted(state_props) == keys, name
            index = 0
            for key, size in zip(keys, sizes):
                index = index * size + props[key].index(state_props[key])
            assert state["id"] == first + index, (name, state["id"], first + index)
            assert index not in seen, name
            seen.add(index)
            if state.get("default"):
                assert default is None, name
                default = state["id"]
        assert len(seen) == count and default is not None, name
        next_state = first + count
        ref_key = tuple(definition_index[(key, tuple(props[key]))] for key in keys)
        assert len(ref_key) < 256
        if ref_key not in ref_cache:
            ref_cache[ref_key] = len(refs)
            refs.extend(ref_key)
        rows.append((name, first, default, ref_cache[ref_key], len(ref_key)))

    assert next_state == total_listed
    assert next_state < NO_STATE
    assert len(refs) < 65536
    return order, definition_list, refs, rows, next_state


def emit_blocks(order, definition_list, refs, rows, state_count):
    names = {name: index for index, name in enumerate(order)}
    out = [
        "use crate::blocks::{Block, Prop};",
        "",
        "pub(crate) const STATE_COUNT: usize = %d;" % state_count,
    ]
    first_of = {row[0]: row[1] for row in rows}
    for constant, block in AIR_BLOCKS.items():
        state = first_of[block]
        assert state == EXPECTED_AIR_STATES[constant], (constant, state)
        out.append("pub(crate) const %s: u16 = %d;" % (constant, state))
    out.append("")
    out.append("pub(crate) static PROPS: [Prop; %d] = [" % len(definition_list))
    for key, values in definition_list:
        out.append(
            "    Prop { name: %s, values: &[%s] },"
            % (quoted([key])[0], ", ".join(quoted(values)))
        )
    out.append("];")
    out.append("")
    out.append("pub(crate) static PROP_REFS: [u8; %d] = [" % len(refs))
    out.extend(wrapped([str(ref) for ref in refs], "    "))
    out.append("];")
    out.append("")
    out.append("pub(crate) static BLOCKS: [Block; %d] = [" % len(rows))
    for name, first, default, ref_start, ref_len in rows:
        out.append(
            '    Block { name: "%s", first: %d, default: %d, ref_start: %d, ref_len: %d },'
            % (name, first, default, ref_start, ref_len)
        )
    out.append("];")
    out.append("")
    by_name = sorted(order)
    out.append("pub(crate) static BY_NAME: [u16; %d] = [" % len(by_name))
    out.extend(wrapped([str(names[name]) for name in by_name], "    "))
    out.append("];")
    return "\n".join(out) + "\n"


def build_static_registries(registries):
    result = []
    for registry_id in sorted(registries):
        spec = registries[registry_id]
        entries = spec["entries"]
        by_id = sorted(entries, key=lambda name: entries[name]["protocol_id"])
        assert [entries[name]["protocol_id"] for name in by_id] == list(range(len(by_id)))
        assert len(by_id) < 65536
        positions = {name: index for index, name in enumerate(by_id)}
        by_name = [positions[name] for name in sorted(by_id)]
        result.append((registry_id, by_id, by_name))
    return result


def emit_static_registries(static_registries):
    index_of = {registry_id: i for i, (registry_id, _, _) in enumerate(static_registries)}
    out = [
        "use crate::registries::StaticRegistry;",
        "",
        "pub(crate) const ENTITY_TYPE: usize = %d;" % index_of["minecraft:entity_type"],
        "pub(crate) const ITEM: usize = %d;" % index_of["minecraft:item"],
        "",
        "pub(crate) static STATIC: [StaticRegistry; %d] = [" % len(static_registries),
    ]
    for registry_id, by_id, by_name in static_registries:
        out.append("    StaticRegistry {")
        out.append('        id: "%s",' % registry_id)
        out.extend(array_lines(quoted(by_id), "        ", "entries: ", ","))
        out.extend(array_lines([str(i) for i in by_name], "        ", "by_name: ", ","))
        out.append("    },")
    out.append("];")
    return "\n".join(out) + "\n"


def build_items(registries, components_dir, blocks_default):
    entries = registries["minecraft:item"]["entries"]
    order = sorted(entries, key=lambda name: entries[name]["protocol_id"])
    assert [entries[name]["protocol_id"] for name in order] == list(range(len(order)))
    assert len(order) < 65535
    files = {
        filename[: -len(".json")]
        for filename in os.listdir(components_dir)
        if filename.endswith(".json")
    }
    assert files == {name.split(":", 1)[1] for name in order}
    for item, block in ITEM_BLOCK_EXCEPTIONS.items():
        assert item in entries and block in blocks_default, (item, block)
        assert item not in blocks_default, item
    for item in ITEM_BLOCK_DENY:
        assert item in entries and item in blocks_default, item
    stacks = []
    states = []
    for name in order:
        path = os.path.join(components_dir, name.split(":", 1)[1] + ".json")
        stack = load_json(path)["components"]["minecraft:max_stack_size"]
        assert isinstance(stack, int) and 1 <= stack <= 99, (name, stack)
        stacks.append(stack)
        if name in ITEM_BLOCK_EXCEPTIONS:
            states.append(blocks_default[ITEM_BLOCK_EXCEPTIONS[name]])
        elif name in blocks_default and name not in ITEM_BLOCK_DENY:
            states.append(blocks_default[name])
        else:
            states.append(NO_STATE)
    return order, stacks, states


def emit_items(stacks, states):
    out = [
        "pub(crate) const NO_STATE: u16 = %d;" % NO_STATE,
        "",
        "pub(crate) static MAX_STACK: [u8; %d] = [" % len(stacks),
    ]
    out.extend(wrapped([str(stack) for stack in stacks], "    "))
    out.append("];")
    out.append("")
    out.append("pub(crate) static BLOCK_STATE: [u16; %d] = [" % len(states))
    out.extend(wrapped([str(state) for state in states], "    "))
    out.append("];")
    return "\n".join(out) + "\n"


def build_synced(datapack):
    result = []
    missing = []
    for constant, path in SYNCED:
        directory = os.path.join(datapack, path)
        if not os.path.isdir(directory):
            missing.append(path)
            names = []
        else:
            names = json_names(directory)
        for name in names:
            assert NAME_RE.match(name), name
        result.append((constant, "minecraft:" + path, ["minecraft:" + n for n in names]))
    return result, missing


def emit_synced(synced):
    out = ["use crate::registries::SyncedRegistry;", ""]
    for index, (constant, _, _) in enumerate(synced):
        if constant in ("BIOME", "DIMENSION_TYPE"):
            out.append("pub(crate) const %s: usize = %d;" % (constant, index))
    out.append("")
    out.append("pub static SYNCED: &[SyncedRegistry] = &[")
    for _, registry_id, entries in synced:
        out.append("    SyncedRegistry {")
        out.append('        id: "%s",' % registry_id)
        out.extend(array_lines(quoted(entries), "        ", "entries: ", ","))
        out.append("    },")
    out.append("];")
    return "\n".join(out) + "\n"


def datapack_registry_dirs(datapack):
    known = set()
    for entry in sorted(os.listdir(datapack)):
        if entry in ("tags", "datapacks") or not os.path.isdir(os.path.join(datapack, entry)):
            continue
        if entry == "worldgen":
            for child in sorted(os.listdir(os.path.join(datapack, entry))):
                if os.path.isdir(os.path.join(datapack, entry, child)):
                    known.add(entry + "/" + child)
        else:
            known.add(entry)
    return known


def read_tag_files(datapack, known):
    root = os.path.join(datapack, "tags")
    table = {}
    for relative in json_names(root):
        parts = relative.split("/")
        registry = None
        for length in range(len(parts) - 1, 0, -1):
            if "/".join(parts[:length]) in known:
                registry = "/".join(parts[:length])
                tag = "/".join(parts[length:])
                break
        if registry is None:
            fail("tag file without known registry: " + relative)
        values = load_json(os.path.join(root, relative + ".json"))["values"]
        table.setdefault(registry, {})["minecraft:" + tag] = values
    return table


def parse_tag_value(value):
    if isinstance(value, str):
        return value, True
    assert set(value) <= {"id", "required"}, value
    return value["id"], value.get("required", True)


def resolve_tags(registry, tag_files, ids):
    cache = {}

    def resolve(tag, stack):
        if tag in cache:
            return cache[tag]
        assert tag not in stack, (registry, tag)
        collected = set()
        for value in tag_files[tag]:
            text, required = parse_tag_value(value)
            if text.startswith("#"):
                reference = normalize_id(text[1:])
                if reference in tag_files:
                    collected |= resolve(reference, stack | {tag})
                elif required:
                    fail("unknown tag %s in %s %s" % (reference, registry, tag))
            else:
                entry = normalize_id(text)
                if entry in ids:
                    collected.add(ids[entry])
                elif required:
                    fail("unknown entry %s in %s %s" % (entry, registry, tag))
        cache[tag] = collected
        return collected

    return {tag: sorted(resolve(tag, frozenset())) for tag in sorted(tag_files)}


def build_tags(datapack, static_registries, synced):
    static_ids = {
        registry_id: {name: i for i, name in enumerate(by_id)}
        for registry_id, by_id, _ in static_registries
    }
    synced_ids = {
        registry_id: {name: i for i, name in enumerate(entries)}
        for _, registry_id, entries in synced
    }
    known = set(registry_id[len("minecraft:") :] for registry_id in static_ids)
    known |= set(registry_id[len("minecraft:") :] for registry_id in synced_ids)
    known |= datapack_registry_dirs(datapack)
    table = read_tag_files(datapack, known)
    result = {}
    skipped = []
    for registry in sorted(table):
        registry_id = "minecraft:" + registry
        if registry_id in static_ids:
            ids = static_ids[registry_id]
        elif registry_id in synced_ids:
            ids = synced_ids[registry_id]
        else:
            skipped.append(registry_id)
            continue
        result[registry_id] = resolve_tags(registry, table[registry], ids)
    return result, skipped


def emit_tags(tag_table):
    out = ["use crate::tags::RegistryTags;", "", "pub static TAGS: &[RegistryTags] = &["]
    for registry_id in sorted(tag_table):
        out.append("    RegistryTags {")
        out.append('        registry: "%s",' % registry_id)
        out.append("        tags: &[")
        for tag in sorted(tag_table[registry_id]):
            assert NAME_RE.match(tag.replace(":", "/", 1)), tag
            ids = [str(i) for i in tag_table[registry_id][tag]]
            line = '            ("%s", &[%s]),' % (tag, ", ".join(ids))
            if len(line) <= LINE_WIDTH:
                out.append(line)
            else:
                out.append('            ("%s", &[' % tag)
                out.extend(wrapped(ids, " " * 16))
                out.append("            ]),")
        out.append("        ],")
        out.append("    },")
    out.append("];")
    return "\n".join(out) + "\n"


def emit_mod():
    return "\n".join(
        [
            "pub mod blocks;",
            "pub mod items;",
            "pub mod static_registries;",
            "pub mod synced;",
            "pub mod tags;",
            "",
        ]
    )


def write_files(out_dir, files):
    os.makedirs(out_dir, exist_ok=True)
    for filename, text in files.items():
        assert "//" not in text, filename
        with open(os.path.join(out_dir, filename), "w", encoding="utf-8") as handle:
            handle.write(text)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--reports", default=DEFAULT_REPORTS)
    parser.add_argument("--datapack", default=DEFAULT_DATAPACK)
    parser.add_argument("--classes", default=DEFAULT_CLASSES)
    parser.add_argument("--javap", default=DEFAULT_JAVAP)
    parser.add_argument("--out", default=DEFAULT_OUT)
    parser.add_argument("--no-javap", action="store_true")
    args = parser.parse_args()

    verified = False
    if not args.no_javap:
        verified = verify_synced_against_java(args.javap, args.classes)

    blocks = load_json(os.path.join(args.reports, "blocks.json"))
    registries = load_json(os.path.join(args.reports, "registries.json"))

    order, definitions, refs, rows, state_count = build_blocks(blocks, registries)
    blocks_default = {row[0]: row[2] for row in rows}
    item_order, stacks, states = build_items(
        registries,
        os.path.join(args.reports, "minecraft", "components", "item"),
        blocks_default,
    )
    static_registries = build_static_registries(registries)
    synced, missing = build_synced(args.datapack)
    tag_table, skipped = build_tags(args.datapack, static_registries, synced)

    files = {
        "mod.rs": emit_mod(),
        "blocks.rs": emit_blocks(order, definitions, refs, rows, state_count),
        "items.rs": emit_items(stacks, states),
        "static_registries.rs": emit_static_registries(static_registries),
        "synced.rs": emit_synced(synced),
        "tags.rs": emit_tags(tag_table),
    }
    write_files(args.out, files)

    total = 0
    for filename, text in files.items():
        size = len(text.encode("utf-8"))
        total += size
        print("%-24s %9d bytes" % (filename, size))
    print("%-24s %9d bytes" % ("total", total))
    print("blocks %d states %d properties %d" % (len(order), state_count, len(definitions)))
    print("items %d block items %d" % (len(item_order), sum(1 for s in states if s != NO_STATE)))
    print("static registries %d" % len(static_registries))
    print("javap verification %s" % ("passed" if verified else "skipped"))
    for _, registry_id, entries in synced:
        print("synced %-45s %5d" % (registry_id, len(entries)))
    print("synced without directory: %s" % (", ".join(missing) if missing else "none"))
    for registry_id in sorted(tag_table):
        print("tags %-45s %5d" % (registry_id, len(tag_table[registry_id])))
    print("tag registries skipped: %s" % (", ".join(skipped) if skipped else "none"))


if __name__ == "__main__":
    main()
