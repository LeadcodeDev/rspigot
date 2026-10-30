import argparse
import json
import math
import os
import re
import struct
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

AIR_BLOCKS = {
    "AIR": "minecraft:air",
    "VOID_AIR": "minecraft:void_air",
    "CAVE_AIR": "minecraft:cave_air",
}

EXPECTED_AIR_STATES = {"AIR": 0, "VOID_AIR": 18649, "CAVE_AIR": 18650}

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


def build_items(registries, components_dir):
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
    stacks = []
    for name in order:
        path = os.path.join(components_dir, name.split(":", 1)[1] + ".json")
        stack = load_json(path)["components"]["minecraft:max_stack_size"]
        assert isinstance(stack, int) and 1 <= stack <= 99, (name, stack)
        stacks.append(stack)
    return order, stacks


def emit_items(stacks):
    out = ["pub(crate) static MAX_STACK: [u8; %d] = [" % len(stacks)]
    out.extend(wrapped([str(stack) for stack in stacks], "    "))
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


COMPONENT_FUNCTIONS = {
    "minecraft:copy_components",
    "minecraft:copy_custom_data",
    "minecraft:copy_name",
    "minecraft:copy_state",
    "minecraft:set_components",
    "minecraft:set_custom_data",
    "minecraft:set_lore",
    "minecraft:set_name",
    "minecraft:toggle_tooltips",
}

ENTITY_TARGETS = {
    "this": "This",
    "attacker": "Attacker",
    "direct_attacker": "DirectAttacker",
    "attacking_player": "AttackingPlayer",
    "target_entity": "TargetEntity",
    "interacting_entity": "InteractingEntity",
}

COMPOSITE_ENTRIES = {
    "minecraft:alternatives": "Alternatives",
    "minecraft:sequence": "Sequence",
    "minecraft:group": "Group",
}

LOOT_TYPES = [
    "BlockPredicate",
    "BonusFormula",
    "Condition",
    "EnchantmentPredicate",
    "Entry",
    "EntryKind",
    "EntityTarget",
    "Function",
    "FunctionKind",
    "IntProvider",
    "IntRange",
    "ItemPredicate",
    "Leaf",
    "LootTable",
    "Output",
    "Pool",
]


class Atom:
    def __init__(self, text):
        self.text = text


class Group:
    def __init__(self, open_, items, close, spaced=False):
        self.open = open_
        self.items = items
        self.close = close
        self.spaced = spaced


class Prefix:
    def __init__(self, text, node):
        self.text = text
        self.node = node


def flat(node):
    if isinstance(node, Atom):
        return node.text
    if isinstance(node, Prefix):
        return node.text + flat(node.node)
    inner = ", ".join(flat(item) for item in node.items)
    if node.spaced and inner:
        inner = " " + inner + " "
    return node.open + inner + node.close


def render(node, indent, prefix="", suffix=""):
    text = indent + prefix + flat(node) + suffix
    if len(text) <= LINE_WIDTH or isinstance(node, Atom) or (
        isinstance(node, Group) and not node.items
    ):
        return [text]
    if isinstance(node, Prefix):
        return render(node.node, indent, prefix + node.text, suffix)
    if len(node.items) == 1 and not node.spaced:
        return render(node.items[0], indent, prefix + node.open, node.close + suffix)
    lines = [indent + prefix + node.open]
    for item in node.items:
        lines.extend(render(item, indent + "    ", "", ","))
    lines.append(indent + node.close + suffix)
    return lines


def rs_struct(name, fields):
    return Group(name + " {", [Prefix(key + ": ", value) for key, value in fields], "}", True)


def rs_call(name, args):
    return Group(name + "(", args, ")")


def rs_slice(items):
    return Group("&[", items, "]")


def rs_array(items):
    return Group("[", items, "]")


def rs_tuple(items):
    return Group("(", items, ")")


def rs_ref(node):
    return Prefix("&", node)


def rs_option(node):
    return Atom("None") if node is None else rs_call("Some", [node])


def rs_str(value):
    assert isinstance(value, str) and re.match(r"^[a-z0-9_.:/\-]+$", value), value
    return Atom('"' + value + '"')


def rs_bool(value):
    assert isinstance(value, bool), value
    return Atom("true" if value else "false")


def rs_int(value):
    assert isinstance(value, int) and not isinstance(value, bool), value
    assert -(2**31) <= value < 2**31, value
    return Atom(str(value))


def to_f32(value):
    try:
        return struct.unpack("<f", struct.pack("<f", value))[0]
    except OverflowError:
        return math.inf


def rs_f32(value):
    assert isinstance(value, (int, float)) and not isinstance(value, bool), value
    target = to_f32(float(value))
    assert math.isfinite(target), value
    for digits in range(1, 10):
        text = "%.*g" % (digits, target)
        if to_f32(float(text)) == target:
            break
    if "e" in text:
        mantissa, exponent = text.split("e")
        text = mantissa + "e" + str(int(exponent))
    elif "." not in text:
        text = text + ".0"
    return Atom(text)


def expect_keys(value, allowed, where):
    assert isinstance(value, dict), (where, value)
    extra = set(value) - set(allowed)
    if extra:
        fail("unsupported keys %s in %s: %s" % (sorted(extra), where, json.dumps(value)))


class LootBuilder:
    def __init__(self, block_ids, item_ids, enchantments, block_properties, tag_table, where):
        self.block_ids = block_ids
        self.item_ids = item_ids
        self.enchantments = enchantments
        self.block_properties = block_properties
        self.tag_table = tag_table
        self.where = where
        self.references = set()
        self.ignored = {}

    def note_ignored(self, construct):
        self.ignored.setdefault(construct, set()).add(self.where)

    def holder_set(self, value, registry):
        ids = self.block_ids if registry == "minecraft:block" else self.item_ids
        if isinstance(value, str) and value.startswith("#"):
            tag = normalize_id(value[1:])
            if tag not in self.tag_table[registry]:
                fail("unknown %s tag %s in %s" % (registry, tag, self.where))
            return rs_call("HolderSet::Tag", [rs_str(tag)])
        values = [value] if isinstance(value, str) else value
        assert isinstance(values, list) and values, (self.where, value)
        resolved = []
        for entry in values:
            name = normalize_id(entry)
            if name not in ids:
                fail("unknown %s entry %s in %s" % (registry, name, self.where))
            resolved.append(ids[name])
        return rs_call("HolderSet::Direct", [rs_slice([Atom(str(i)) for i in sorted(set(resolved))])])

    def block_names(self, value):
        if isinstance(value, str) and value.startswith("#"):
            tag = normalize_id(value[1:])
            names = {name for name, i in self.block_ids.items() if i in set(self.tag_table["minecraft:block"][tag])}
            return sorted(names)
        values = [value] if isinstance(value, str) else value
        return [normalize_id(entry) for entry in values]

    def block_predicate(self, predicate):
        expect_keys(predicate, {"blocks", "state"}, self.where)
        state = predicate.get("state", {})
        assert isinstance(state, dict), (self.where, state)
        blocks = predicate.get("blocks")
        if blocks is not None:
            for name in self.block_names(blocks):
                properties = self.block_properties[name]
                for key, value in state.items():
                    if not isinstance(value, str):
                        fail("non exact state matcher in %s" % self.where)
                    if key not in properties or value not in properties[key]:
                        fail("state %s=%s not valid for %s in %s" % (key, value, name, self.where))
        pairs = [rs_tuple([rs_str(key), rs_str(state[key])]) for key in sorted(state)]
        return rs_struct(
            "BlockPredicate",
            [
                ("blocks", rs_option(None if blocks is None else self.holder_set(blocks, "minecraft:block"))),
                ("state", rs_slice(pairs)),
            ],
        )

    def int_range(self, value):
        if isinstance(value, int) and not isinstance(value, bool):
            return rs_struct("IntRange", [("min", rs_option(rs_int(value))), ("max", rs_option(rs_int(value)))])
        expect_keys(value, {"min", "max"}, self.where)
        return rs_struct(
            "IntRange",
            [
                ("min", rs_option(None if "min" not in value else rs_int(value["min"]))),
                ("max", rs_option(None if "max" not in value else rs_int(value["max"]))),
            ],
        )

    def enchantment_predicate(self, predicate):
        expect_keys(predicate, {"enchantments", "levels"}, self.where)
        names = None
        if "enchantments" in predicate:
            value = predicate["enchantments"]
            values = [value] if isinstance(value, str) else value
            names = []
            for entry in values:
                if entry.startswith("#"):
                    fail("enchantment tag in %s" % self.where)
                name = normalize_id(entry)
                if name not in self.enchantments:
                    fail("unknown enchantment %s in %s" % (name, self.where))
                names.append(rs_str(name))
        levels = None if "levels" not in predicate else self.int_range(predicate["levels"])
        return rs_struct(
            "EnchantmentPredicate",
            [
                ("enchantments", rs_option(None if names is None else rs_slice(names))),
                ("levels", rs_option(levels)),
            ],
        )

    def item_predicate(self, predicate):
        expect_keys(predicate, {"items", "predicates"}, self.where)
        sub = predicate.get("predicates", {})
        expect_keys(sub, {"minecraft:enchantments"}, self.where)
        enchantments = [self.enchantment_predicate(p) for p in sub.get("minecraft:enchantments", [])]
        items = predicate.get("items")
        return rs_struct(
            "ItemPredicate",
            [
                ("items", rs_option(None if items is None else self.holder_set(items, "minecraft:item"))),
                ("enchantments", rs_slice(enchantments)),
            ],
        )

    def float_constant(self, value):
        if isinstance(value, dict):
            expect_keys(value, {"type", "value"}, self.where)
            if value.get("type") != "minecraft:constant":
                fail("unsupported float provider %s in %s" % (json.dumps(value), self.where))
            value = value["value"]
        return rs_f32(value)

    def int_constant(self, value):
        if isinstance(value, dict):
            expect_keys(value, {"type", "value"}, self.where)
            if value.get("type") != "minecraft:constant":
                fail("unsupported nested int provider %s in %s" % (json.dumps(value), self.where))
            value = value["value"]
        return rs_int(value)

    def int_provider(self, value):
        if not isinstance(value, dict):
            return rs_call("IntProvider::Constant", [rs_int(value)])
        kind = value.get("type")
        if kind == "minecraft:constant":
            expect_keys(value, {"type", "value"}, self.where)
            return rs_call("IntProvider::Constant", [rs_int(value["value"])])
        if kind == "minecraft:uniform":
            expect_keys(value, {"type", "min", "max"}, self.where)
            return rs_struct(
                "IntProvider::Uniform",
                [("min", self.int_constant(value["min"])), ("max", self.int_constant(value["max"]))],
            )
        if kind == "minecraft:binomial":
            expect_keys(value, {"type", "n", "p"}, self.where)
            return rs_struct(
                "IntProvider::Binomial",
                [("n", self.int_constant(value["n"])), ("p", self.float_constant(value["p"]))],
            )
        fail("unsupported int provider %s in %s" % (json.dumps(value), self.where))

    def condition(self, value):
        if isinstance(value, str):
            name = normalize_id(value)
            self.references.add(name)
            return rs_call("Condition::Reference", [rs_str(name)])
        kind = value.get("type") if isinstance(value, dict) else None
        if kind == "minecraft:survives_explosion":
            expect_keys(value, {"type"}, self.where)
            return Atom("Condition::SurvivesExplosion")
        if kind == "minecraft:match_block":
            rest = {k: v for k, v in value.items() if k != "type"}
            return rs_call("Condition::MatchBlock", [self.block_predicate(rest)])
        if kind == "minecraft:match_tool":
            expect_keys(value, {"type", "predicate"}, self.where)
            return rs_call("Condition::MatchTool", [self.item_predicate(value.get("predicate", {}))])
        if kind == "minecraft:location_check":
            expect_keys(value, {"type", "offsetX", "offsetY", "offsetZ", "predicate"}, self.where)
            predicate = value.get("predicate", {})
            expect_keys(predicate, {"block"}, self.where)
            block = predicate.get("block")
            self.note_ignored("location_check")
            offset = rs_array([rs_int(value.get(axis, 0)) for axis in ("offsetX", "offsetY", "offsetZ")])
            return rs_struct(
                "Condition::LocationCheck",
                [
                    ("offset", offset),
                    ("block", rs_option(None if block is None else self.block_predicate(block))),
                ],
            )
        if kind == "minecraft:entity_properties":
            expect_keys(value, {"type", "entity", "predicate"}, self.where)
            if value.get("predicate", {}) != {}:
                fail("non empty entity predicate in %s" % self.where)
            target = ENTITY_TARGETS[value["entity"]]
            return rs_call("Condition::EntityProperties", [Atom("EntityTarget::" + target)])
        if kind == "minecraft:random_chance":
            expect_keys(value, {"type", "chance"}, self.where)
            return rs_call("Condition::RandomChance", [self.float_constant(value["chance"])])
        if kind == "minecraft:table_bonus":
            expect_keys(value, {"type", "enchantment", "chances"}, self.where)
            enchantment = normalize_id(value["enchantment"])
            assert enchantment in self.enchantments, (self.where, enchantment)
            assert value["chances"], self.where
            return rs_struct(
                "Condition::TableBonus",
                [
                    ("enchantment", rs_str(enchantment)),
                    ("chances", rs_slice([rs_f32(c) for c in value["chances"]])),
                ],
            )
        if kind == "minecraft:inverted":
            expect_keys(value, {"type", "term"}, self.where)
            return rs_call("Condition::Inverted", [rs_ref(self.condition(value["term"]))])
        if kind in ("minecraft:any_of", "minecraft:all_of"):
            expect_keys(value, {"type", "terms"}, self.where)
            variant = "Condition::AnyOf" if kind == "minecraft:any_of" else "Condition::AllOf"
            return rs_call(variant, [rs_slice([self.condition(term) for term in value["terms"]])])
        fail("unsupported condition %s in %s" % (json.dumps(value), self.where))

    def optional_condition(self, owner):
        return rs_option(None if "condition" not in owner else self.condition(owner["condition"]))

    def bonus_formula(self, value):
        formula = value["formula"]
        parameters = value.get("parameters")
        if formula == "minecraft:ore_drops":
            assert parameters is None, self.where
            return Atom("BonusFormula::OreDrops")
        if formula == "minecraft:uniform_bonus_count":
            expect_keys(parameters, {"bonusMultiplier"}, self.where)
            return rs_struct(
                "BonusFormula::UniformBonusCount",
                [("bonus_multiplier", rs_int(parameters["bonusMultiplier"]))],
            )
        if formula == "minecraft:binomial_with_bonus_count":
            expect_keys(parameters, {"extra", "probability"}, self.where)
            return rs_struct(
                "BonusFormula::BinomialWithBonusCount",
                [
                    ("extra", rs_int(parameters["extra"])),
                    ("probability", rs_f32(parameters["probability"])),
                ],
            )
        fail("unsupported bonus formula %s in %s" % (formula, self.where))

    def function(self, value):
        assert isinstance(value, dict), (self.where, value)
        kind = value.get("type")
        if kind == "minecraft:set_count":
            expect_keys(value, {"type", "condition", "count", "add"}, self.where)
            body = rs_struct(
                "FunctionKind::SetCount",
                [("count", self.int_provider(value["count"])), ("add", rs_bool(value.get("add", False)))],
            )
        elif kind == "minecraft:apply_bonus":
            expect_keys(value, {"type", "condition", "enchantment", "formula", "parameters"}, self.where)
            enchantment = normalize_id(value["enchantment"])
            assert enchantment in self.enchantments, (self.where, enchantment)
            body = rs_struct(
                "FunctionKind::ApplyBonus",
                [("enchantment", rs_str(enchantment)), ("formula", self.bonus_formula(value))],
            )
        elif kind == "minecraft:limit_count":
            expect_keys(value, {"type", "condition", "limit"}, self.where)
            limit = value["limit"]
            expect_keys(limit, {"min", "max"}, self.where)
            body = rs_struct(
                "FunctionKind::LimitCount",
                [
                    ("min", rs_option(None if "min" not in limit else self.int_provider(limit["min"]))),
                    ("max", rs_option(None if "max" not in limit else self.int_provider(limit["max"]))),
                ],
            )
        elif kind == "minecraft:explosion_decay":
            expect_keys(value, {"type", "condition"}, self.where)
            body = Atom("FunctionKind::ExplosionDecay")
        elif kind in COMPONENT_FUNCTIONS:
            self.note_ignored(kind)
            body = rs_call("FunctionKind::Components", [rs_str(kind)])
        else:
            fail("unsupported function %s in %s" % (json.dumps(value), self.where))
        return rs_struct("Function", [("condition", self.optional_condition(value)), ("kind", body)])

    def modifiers(self, owner):
        value = owner.get("modifier")
        if value is None:
            return rs_slice([])
        values = value if isinstance(value, list) else [value]
        return rs_slice([self.function(item) for item in values])

    def leaf(self, value, output):
        return rs_call(
            "EntryKind::Leaf",
            [
                rs_struct(
                    "Leaf",
                    [
                        ("weight", rs_int(value.get("weight", 1))),
                        ("quality", rs_int(value.get("quality", 0))),
                        ("output", output),
                    ],
                )
            ],
        )

    def entry(self, value):
        kind = value.get("type")
        leaf_keys = {"type", "condition", "modifier", "weight", "quality"}
        if kind == "minecraft:item":
            expect_keys(value, leaf_keys | {"name"}, self.where)
            name = normalize_id(value["name"])
            if name not in self.item_ids:
                fail("unknown item %s in %s" % (name, self.where))
            body = self.leaf(value, rs_call("Output::Item", [Atom(str(self.item_ids[name]))]))
        elif kind == "minecraft:tag":
            expect_keys(value, leaf_keys | {"items", "expand"}, self.where)
            body = self.leaf(
                value,
                rs_struct(
                    "Output::Items",
                    [
                        ("items", self.holder_set(value["items"], "minecraft:item")),
                        ("expand", rs_bool(value.get("expand", False))),
                    ],
                ),
            )
        elif kind == "minecraft:empty":
            expect_keys(value, leaf_keys, self.where)
            body = self.leaf(value, Atom("Output::Empty"))
        elif kind == "minecraft:dynamic":
            expect_keys(value, leaf_keys | {"name"}, self.where)
            self.note_ignored("dynamic")
            body = self.leaf(value, rs_call("Output::Dynamic", [rs_str(normalize_id(value["name"]))]))
        elif kind in COMPOSITE_ENTRIES:
            expect_keys(value, {"type", "condition", "modifier", "children"}, self.where)
            children = rs_slice([self.entry(child) for child in value["children"]])
            body = rs_call("EntryKind::" + COMPOSITE_ENTRIES[kind], [children])
        else:
            fail("unsupported entry %s in %s" % (json.dumps(value), self.where))
        return rs_struct(
            "Entry",
            [
                ("condition", self.optional_condition(value)),
                ("modifier", self.modifiers(value)),
                ("kind", body),
            ],
        )

    def pool(self, value):
        expect_keys(value, {"rolls", "bonus_rolls", "condition", "modifier", "entries"}, self.where)
        return rs_struct(
            "Pool",
            [
                ("rolls", self.int_provider(value["rolls"])),
                ("bonus_rolls", self.float_constant(value.get("bonus_rolls", 0.0))),
                ("condition", self.optional_condition(value)),
                ("modifier", self.modifiers(value)),
                ("entries", rs_slice([self.entry(entry) for entry in value["entries"]])),
            ],
        )

    def table(self, value):
        expect_keys(value, {"type", "pools", "modifier", "random_sequence"}, self.where)
        if value["type"] != "minecraft:block":
            fail("unexpected loot table type %s in %s" % (value["type"], self.where))
        return rs_struct(
            "LootTable",
            [
                ("pools", rs_slice([self.pool(pool) for pool in value.get("pools", [])])),
                ("modifier", self.modifiers(value)),
            ],
        )


def load_block_properties(reports):
    blocks = load_json(os.path.join(reports, "block_properties.json"))["blocks"]
    assert [block["id"] for block in blocks] == list(range(len(blocks)))
    return blocks


def build_loot(datapack, block_reports, static_registries, synced, tag_table):
    static_ids = {
        registry_id: {name: i for i, name in enumerate(by_id)}
        for registry_id, by_id, _ in static_registries
    }
    block_ids = static_ids["minecraft:block"]
    item_ids = static_ids["minecraft:item"]
    enchantments = set(dict((rid, e) for _, rid, e in synced)["minecraft:enchantment"])
    assert [block["name"] for block in block_reports] == sorted(block_ids, key=block_ids.get)
    block_properties = {
        block["name"]: {p["name"]: p["values"] for p in block["properties"]} for block in block_reports
    }
    references = set()
    ignored = {}

    def builder(where):
        return LootBuilder(block_ids, item_ids, enchantments, block_properties, tag_table, where)

    tables = []
    for path in json_names(os.path.join(datapack, "loot_table", "blocks")):
        assert NAME_RE.match(path), path
        name = "minecraft:blocks/" + path
        current = builder(name)
        node = current.table(load_json(os.path.join(datapack, "loot_table", "blocks", path + ".json")))
        references |= current.references
        for construct, owners in current.ignored.items():
            ignored.setdefault(construct, set()).update(owners)
        tables.append((name, node))

    predicates = []
    for path in json_names(os.path.join(datapack, "predicate")):
        name = "minecraft:" + path
        current = builder(name)
        node = current.condition(load_json(os.path.join(datapack, "predicate", path + ".json")))
        references |= current.references
        predicates.append((name, node, current.references))
    predicate_names = {name for name, _, _ in predicates}
    missing = references - predicate_names
    if missing:
        fail("unresolved predicate references: " + ", ".join(sorted(missing)))
    graph = {name: refs for name, _, refs in predicates}

    def check_cycle(name, stack):
        assert name not in stack, ("predicate cycle", name)
        for child in graph[name]:
            check_cycle(child, stack | {name})

    for name in graph:
        check_cycle(name, frozenset())

    table_index = {name: index for index, (name, _) in enumerate(tables)}
    assert len(tables) < 65535
    block_tables = []
    wall_mapped = []
    for block in block_reports:
        path = block["name"].split(":", 1)[1]
        own = table_index.get("minecraft:blocks/" + path)
        if own is None:
            floors = [
                other
                for other in block_reports
                if other["wall_variant"] == block["id"]
                and other["id"] != block["id"]
                and "minecraft:blocks/" + other["name"].split(":", 1)[1] in table_index
            ]
            assert len(floors) <= 1, (block["name"], floors)
            if floors:
                own = table_index["minecraft:blocks/" + floors[0]["name"].split(":", 1)[1]]
                wall_mapped.append(block["name"])
        block_tables.append(65535 if own is None else own)
    return tables, block_tables, predicates, wall_mapped, ignored


def used_imports(text, names):
    return [name for name in names if re.search(r"\b" + name + r"\b", text)]


def emit_loot(tables, block_tables, predicates):
    body = ["pub(crate) static TABLES: [(&str, LootTable); %d] = [" % len(tables)]
    for name, node in tables:
        body.extend(render(rs_tuple([rs_str(name), node]), "    ", "", ","))
    body.append("];")
    body.append("")
    body.append("pub(crate) static BLOCK_TABLES: [u16; %d] = [" % len(block_tables))
    body.extend(wrapped([str(index) for index in block_tables], "    "))
    body.append("];")
    body.append("")
    body.append("pub(crate) static PREDICATES: [(&str, Condition); %d] = [" % len(predicates))
    for name, node, _ in predicates:
        body.extend(render(rs_tuple([rs_str(name), node]), "    ", "", ","))
    body.append("];")
    text = "\n".join(body) + "\n"
    head = ["use crate::loot::{%s};" % ", ".join(used_imports(text, LOOT_TYPES))]
    if "HolderSet::" in text:
        head.append("use crate::tags::HolderSet;")
    return "\n".join(head) + "\n\n" + text


def build_tools(registries, components_dir, static_registries, tag_table):
    static_ids = {
        registry_id: {name: i for i, name in enumerate(by_id)}
        for registry_id, by_id, _ in static_registries
    }
    builder = LootBuilder(static_ids["minecraft:block"], static_ids["minecraft:item"], set(), {}, tag_table, "")
    entries = registries["minecraft:item"]["entries"]
    order = sorted(entries, key=lambda name: entries[name]["protocol_id"])
    tools = []
    for index, name in enumerate(order):
        path = os.path.join(components_dir, name.split(":", 1)[1] + ".json")
        tool = load_json(path)["components"].get("minecraft:tool")
        if tool is None:
            continue
        builder.where = name
        expect_keys(
            tool,
            {"rules", "default_mining_speed", "damage_per_block", "can_destroy_blocks_in_creative"},
            name,
        )
        rules = []
        for rule in tool["rules"]:
            expect_keys(rule, {"blocks", "speed", "correct_for_drops"}, name)
            rules.append(
                rs_struct(
                    "ToolRule",
                    [
                        ("blocks", builder.holder_set(rule["blocks"], "minecraft:block")),
                        ("speed", rs_option(None if "speed" not in rule else rs_f32(rule["speed"]))),
                        (
                            "correct_for_drops",
                            rs_option(
                                None
                                if "correct_for_drops" not in rule
                                else rs_bool(rule["correct_for_drops"])
                            ),
                        ),
                    ],
                )
            )
        node = rs_struct(
            "Tool",
            [
                ("rules", rs_slice(rules)),
                ("default_mining_speed", rs_f32(tool.get("default_mining_speed", 1.0))),
                ("damage_per_block", rs_int(tool.get("damage_per_block", 1))),
                (
                    "can_destroy_blocks_in_creative",
                    rs_bool(tool.get("can_destroy_blocks_in_creative", True)),
                ),
            ],
        )
        tools.append((index, node))
    return tools


def emit_tools(tools):
    body = ["pub(crate) static TOOLS: [(u16, Tool); %d] = [" % len(tools)]
    for index, node in tools:
        body.extend(render(rs_tuple([Atom(str(index)), node]), "    ", "", ","))
    body.append("];")
    text = "\n".join(body) + "\n"
    head = ["use crate::items::{%s};" % ", ".join(used_imports(text, ["Tool", "ToolRule"]))]
    head.append("use crate::tags::HolderSet;")
    return "\n".join(head) + "\n\n" + text


def emit_mod():
    return "\n".join(
        [
            "pub mod items;",
            "pub mod loot;",
            "pub mod static_registries;",
            "pub mod synced;",
            "pub mod tags;",
            "pub mod tools;",
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

    registries = load_json(os.path.join(args.reports, "registries.json"))

    item_order, stacks = build_items(
        registries,
        os.path.join(args.reports, "minecraft", "components", "item"),
    )
    static_registries = build_static_registries(registries)
    synced, missing = build_synced(args.datapack)
    tag_table, skipped = build_tags(args.datapack, static_registries, synced)
    block_reports = load_block_properties(args.reports)
    tables, block_tables, predicates, wall_mapped, ignored = build_loot(
        args.datapack, block_reports, static_registries, synced, tag_table
    )
    tools = build_tools(
        registries,
        os.path.join(args.reports, "minecraft", "components", "item"),
        static_registries,
        tag_table,
    )

    files = {
        "mod.rs": emit_mod(),
        "items.rs": emit_items(stacks),
        "static_registries.rs": emit_static_registries(static_registries),
        "synced.rs": emit_synced(synced),
        "tags.rs": emit_tags(tag_table),
        "loot.rs": emit_loot(tables, block_tables, predicates),
        "tools.rs": emit_tools(tools),
    }
    write_files(args.out, files)

    total = 0
    for filename, text in files.items():
        size = len(text.encode("utf-8"))
        total += size
        print("%-24s %9d bytes" % (filename, size))
    print("%-24s %9d bytes" % ("total", total))
    print("items %d" % len(item_order))
    print("static registries %d" % len(static_registries))
    print("javap verification %s" % ("passed" if verified else "skipped"))
    for _, registry_id, entries in synced:
        print("synced %-45s %5d" % (registry_id, len(entries)))
    print("synced without directory: %s" % (", ".join(missing) if missing else "none"))
    for registry_id in sorted(tag_table):
        print("tags %-45s %5d" % (registry_id, len(tag_table[registry_id])))
    print("tag registries skipped: %s" % (", ".join(skipped) if skipped else "none"))
    print("loot tables %d" % len(tables))
    print("blocks with loot table %d" % sum(1 for index in block_tables if index != 65535))
    print("wall variants using floor table %d" % len(wall_mapped))
    print("named predicates %d" % len(predicates))
    print("tools %d" % len(tools))
    for construct in sorted(ignored):
        owners = sorted(ignored[construct])
        print("ignored %s in %d tables: %s" % (construct, len(owners), ", ".join(owners)))


if __name__ == "__main__":
    main()
