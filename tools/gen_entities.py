import argparse
import json
import math
import os
import re
import struct
import sys

RE_ROOT = "/Users/baptiste.parmantier/Development/rspigot/re"
DEFAULT_DUMP = RE_ROOT + "/datagen/generated/reports/entity_types.json"
DEFAULT_REGISTRIES = RE_ROOT + "/datagen/generated/reports/registries.json"
CRATE = os.path.normpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "crates", "rspigot-entities")
)
DEFAULT_FAMILIES = os.path.join(CRATE, "families.txt")
DEFAULT_OUT = os.path.join(CRATE, "src")
EXPECTED_VERSION = "26.3"
NAMESPACE = "minecraft:"

RESERVED_STRUCTS = {
    "EntityType", "EntityTypeId", "EntityKind", "EntityDimensions", "EntityBehavior",
    "MobCategory", "Plain", "Self", "Option", "Some", "None", "Result", "Ok", "Err", "Vec",
    "String", "Box", "Default", "Clone", "Copy", "Send", "Sync", "Sized", "Drop", "Fn", "FnMut",
    "FnOnce", "Iterator", "IntoIterator", "Extend", "ToString", "ToOwned", "PartialEq", "Eq",
    "PartialOrd", "Ord", "AsRef", "AsMut", "Into", "From", "TryFrom", "TryInto", "FromIterator",
    "DoubleEndedIterator", "ExactSizeIterator", "Unpin",
}

RUST_KEYWORDS = {
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
}

FAMILY_TYPE = re.compile(r"^[A-Z][A-Za-z0-9]*(<(true|false|[0-9]+)(, *(true|false|[0-9]+))*>)?$")
MODULE_NAME = re.compile(r"^[a-z][a-z0-9_]*$")
STRUCT_NAME = re.compile(r"^[A-Z][A-Za-z0-9]*$")
CLASS_NAME = re.compile(r"^[A-Za-z_$][A-Za-z0-9_$]*$")
WIDTH = 100


def fail(message):
    sys.exit("gen_entities: " + message)


def load_json(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def struct_name(path):
    return "".join(part[:1].upper() + part[1:] for part in path.split("_"))


def f32(value):
    return struct.unpack("<f", struct.pack("<f", value))[0]


def finite(value, what):
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        fail("%s is not a finite number: %r" % (what, value))
    return float(value)


def f32_literal(value, what):
    target = f32(finite(value, what))
    candidate = repr(target)
    for digits in range(1, 10):
        candidate = "%.*g" % (digits, target)
        if f32(float(candidate)) == target:
            break
    text = repr(float(candidate))
    if not re.fullmatch(r"-?[0-9]+\.[0-9]+", text):
        fail("%s float %r has no plain literal form" % (what, value))
    return text


def f64_literal(value, what):
    text = repr(finite(value, what))
    if not re.fullmatch(r"-?[0-9]+\.[0-9]+", text):
        fail("%s float %r has no plain literal form" % (what, value))
    return text


def rust_string(value):
    if not re.fullmatch(r"[a-z0-9_:/.]+", value):
        fail("unexpected string " + repr(value))
    return '"' + value + '"'


def boolean(value, what):
    if not isinstance(value, bool):
        fail("%s is not a boolean: %r" % (what, value))
    return "true" if value else "false"


def integer(value, low, high, what):
    if isinstance(value, bool) or not isinstance(value, int) or not low <= value <= high:
        fail("%s out of range [%d, %d]: %r" % (what, low, high, value))
    return value


def registry_names(registries, key):
    registry = registries.get(key)
    if not registry:
        fail("registries.json has no " + key)
    by_id = {}
    for name, entry in registry["entries"].items():
        by_id[entry["protocol_id"]] = name
    if sorted(by_id) != list(range(len(registry["entries"]))):
        fail(key + " protocol ids are not dense")
    return [by_id[index] for index in range(len(by_id))]


def read_families(path, entities):
    classes = {entity["class"] for entity in entities}
    names = {entity["name"] for entity in entities}
    by_class = {}
    by_entity = {}
    if not os.path.exists(path):
        return by_class, by_entity
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().splitlines()
    for number, raw in enumerate(lines, 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split()
        if len(parts) != 2:
            fail("%s:%d: expected '<key> <RustFamilyType>'" % (path, number))
        key, family = parts
        if not FAMILY_TYPE.match(family):
            fail("%s:%d: invalid family type %r" % (path, number, family))
        if key.startswith("entity:"):
            name = key[len("entity:"):]
            if ":" not in name:
                name = NAMESPACE + name
            if name not in names:
                fail("%s:%d: unknown entity type %r" % (path, number, name))
            if name in by_entity:
                fail("%s:%d: duplicate entity type %r" % (path, number, name))
            by_entity[name] = family
        else:
            if key not in classes:
                fail("%s:%d: unknown vanilla class %r" % (path, number, key))
            if key in by_class:
                fail("%s:%d: duplicate class %r" % (path, number, key))
            by_class[key] = family
    return by_class, by_entity


def validate_categories(dump):
    categories = dump["mob_categories"]
    seen = set()
    for index, category in enumerate(categories):
        name = category["name"]
        if category["ordinal"] != index:
            fail("mob category %s ordinal %r != %d" % (name, category["ordinal"], index))
        if category["serialized_name"] != name:
            fail("mob category %s serialized as %r" % (name, category["serialized_name"]))
        if not MODULE_NAME.match(name) or name in seen:
            fail("unsupported mob category name " + repr(name))
        seen.add(name)
        category["variant"] = struct_name(name)
        if not STRUCT_NAME.match(category["variant"]):
            fail("unsupported mob category variant " + category["variant"])
        maximum = category["max_instances_per_chunk"]
        if maximum == -1:
            category["max"] = "None"
        else:
            category["max"] = "Some(%d)" % integer(maximum, 0, 65535, name + " max instances")
        integer(category["despawn_distance"], 0, 65535, name + " despawn distance")
        integer(category["no_despawn_distance"], 0, 65535, name + " no despawn distance")
        boolean(category["is_friendly"], name + " friendly")
        boolean(category["is_persistent"], name + " persistent")
    return {category["name"]: category for category in categories}


def validate(dump, registries):
    if dump.get("version") != EXPECTED_VERSION:
        fail("dump version %r != %r" % (dump.get("version"), EXPECTED_VERSION))
    categories = validate_categories(dump)
    entity_names = registry_names(registries, "minecraft:entity_type")
    item_names = registry_names(registries, "minecraft:item")
    attribute_names = registry_names(registries, "minecraft:attribute")
    dumped_attributes = [attribute["name"] for attribute in dump["attributes"]]
    if dumped_attributes != attribute_names:
        fail("dump attributes differ from registries.json")
    attribute_order = {name: index for index, name in enumerate(attribute_names)}
    entities = dump["entity_types"]
    if [entity["name"] for entity in entities] != entity_names:
        fail("entity type names or order differ from registries.json")
    no_update = dump["no_update_interval"]
    eggs = {}
    structs = set()
    for index, entity in enumerate(entities):
        name = entity["name"]
        if entity["id"] != index:
            fail("%s registry id %r != %d" % (name, entity["id"], index))
        if not name.startswith(NAMESPACE):
            fail("unsupported entity namespace " + name)
        path = name[len(NAMESPACE):]
        if not MODULE_NAME.match(path) or path in RUST_KEYWORDS:
            fail("unsupported entity name " + name)
        entity["path"] = path
        entity["struct"] = struct_name(path)
        if not STRUCT_NAME.match(entity["struct"]) or entity["struct"] in RESERVED_STRUCTS:
            fail("unsupported struct name %s for %s" % (entity["struct"], name))
        if entity["struct"] in structs:
            fail("duplicate struct name " + entity["struct"])
        structs.add(entity["struct"])
        if not CLASS_NAME.match(entity["class"]):
            fail("%s has unsupported class %r" % (name, entity["class"]))
        if entity["category"] not in categories:
            fail("%s has unknown category %r" % (name, entity["category"]))
        for key in ("width", "height", "eye_height"):
            if finite(entity[key], name + " " + key) < 0:
                fail("%s negative %s" % (name, key))
        for key in ("fire_immune", "can_summon", "can_serialize", "can_spawn_far_from_player", "living", "has_update_interval", "track_deltas", "allowed_in_peaceful"):
            boolean(entity[key], name + " " + key)
        integer(entity["client_tracking_range"], 0, 255, name + " tracking range")
        interval = entity["update_interval"]
        if entity["has_update_interval"]:
            integer(interval, 1, 255, name + " update interval")
        elif interval != no_update:
            fail("%s has no update interval but reports %r" % (name, interval))
        egg = entity["spawn_egg"]
        if egg is not None:
            integer(egg, 0, len(item_names) - 1, name + " spawn egg")
            if egg in eggs:
                fail("%s and %s share spawn egg %d" % (eggs[egg], name, egg))
            if item_names[egg] != name + "_spawn_egg":
                fail("%s spawn egg %d is %s" % (name, egg, item_names[egg]))
            eggs[egg] = name
        ancestors = entity["ancestors"]
        if not ancestors or ancestors[0] != entity["class"] or ancestors[-1] != "Entity":
            fail("%s has ancestry %r" % (name, ancestors))
        for ancestor in ancestors:
            if not CLASS_NAME.match(ancestor):
                fail("%s has unsupported ancestor %r" % (name, ancestor))
        if ("LivingEntity" in ancestors) != entity["living"]:
            fail("%s living %r disagrees with ancestry" % (name, entity["living"]))
        attributes = entity["default_attributes"]
        if entity["living"] != bool(attributes):
            fail("%s living %r with %d default attributes" % (name, entity["living"], len(attributes)))
        previous = -1
        for attribute in attributes:
            if attribute["name"] not in attribute_order:
                fail("%s has unknown attribute %r" % (name, attribute["name"]))
            if attribute_order[attribute["name"]] <= previous:
                fail("%s attributes not in registry order" % name)
            previous = attribute_order[attribute["name"]]
            finite(attribute["base"], name + " " + attribute["name"])
            if attribute["base"] != attribute["value"]:
                fail("%s %s base %r differs from value %r" % (name, attribute["name"], attribute["base"], attribute["value"]))
    return categories


def entity_file(entity, categories, behavior):
    name = entity["name"]
    category = categories[entity["category"]]["variant"]
    interval = "Some(%d)" % entity["update_interval"] if entity["has_update_interval"] else "None"
    egg = "None" if entity["spawn_egg"] is None else "Some(%d)" % entity["spawn_egg"]
    lines = [
        "use crate::{EntityType, EntityTypeId, MobCategory};",
        "",
        "pub struct %s;" % entity["struct"],
        "",
        "impl EntityType for %s {" % entity["struct"],
        "    const NAME: &'static str = %s;" % rust_string(name),
        "    const ID: EntityTypeId = EntityTypeId(%d);" % entity["id"],
        '    const CLASS: &\'static str = "%s";' % entity["class"],
        "    const CATEGORY: MobCategory = MobCategory::%s;" % category,
        "    const WIDTH: f32 = %s;" % f32_literal(entity["width"], name + " width"),
        "    const HEIGHT: f32 = %s;" % f32_literal(entity["height"], name + " height"),
        "    const EYE_HEIGHT: f32 = %s;" % f32_literal(entity["eye_height"], name + " eye height"),
        "    const FIRE_IMMUNE: bool = %s;" % boolean(entity["fire_immune"], name),
        "    const SUMMONABLE: bool = %s;" % boolean(entity["can_summon"], name),
        "    const CAN_SERIALIZE: bool = %s;" % boolean(entity["can_serialize"], name),
        "    const ALLOWED_IN_PEACEFUL: bool = %s;" % boolean(entity["allowed_in_peaceful"], name),
        "    const CAN_SPAWN_FAR_FROM_PLAYER: bool = %s;" % boolean(entity["can_spawn_far_from_player"], name),
        "    const CLIENT_TRACKING_RANGE: u8 = %d;" % entity["client_tracking_range"],
        "    const UPDATE_INTERVAL: Option<u8> = %s;" % interval,
        "    const TRACK_DELTAS: bool = %s;" % boolean(entity["track_deltas"], name),
        "    const SPAWN_EGG: Option<u16> = %s;" % egg,
        "    const LIVING: bool = %s;" % boolean(entity["living"], name),
    ]
    ancestors = ", ".join('"%s"' % ancestor for ancestor in entity["ancestors"])
    single = "    const ANCESTORS: &'static [&'static str] = &[%s];" % ancestors
    if len(single) <= WIDTH:
        lines.append(single)
    else:
        lines.append("    const ANCESTORS: &'static [&'static str] = &[")
        lines.extend('        "%s",' % ancestor for ancestor in entity["ancestors"])
        lines.append("    ];")
    attributes = entity["default_attributes"]
    if attributes:
        lines.append("    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[")
        for attribute in attributes:
            lines.append(
                "        (%s, %s),"
                % (rust_string(attribute["name"]), f64_literal(attribute["base"], name + " " + attribute["name"]))
            )
        lines.append("    ];")
    else:
        lines.append("    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];")
    lines.append("    type Behavior = crate::family::%s;" % behavior)
    lines.append("}")
    for line in lines:
        if len(line) > WIDTH:
            fail("%s line too long: %s" % (name, line))
    return "\n".join(lines) + "\n"


def categories_file(categories):
    ordered = sorted(categories.values(), key=lambda category: category["ordinal"])
    lines = ["crate::category::mob_categories! {"]
    lines.append("    count: %d," % len(ordered))
    lines.append("    categories: [")
    for category in ordered:
        name = category["name"]
        lines.append("        %s {" % category["variant"])
        lines.append("            name: %s," % rust_string(name))
        lines.append("            max_instances_per_chunk: %s," % category["max"])
        lines.append("            friendly: %s," % boolean(category["is_friendly"], name))
        lines.append("            persistent: %s," % boolean(category["is_persistent"], name))
        lines.append("            despawn_distance: %d," % category["despawn_distance"])
        lines.append("            no_despawn_distance: %d," % category["no_despawn_distance"])
        lines.append("        },")
    lines.append("    ],")
    lines.append("}")
    return "\n".join(lines) + "\n"


def kind_file(entities):
    lines = ["crate::lookup::entity_kinds! {"]
    lines.append("    count: %d," % len(entities))
    lines.append("    kinds: [")
    lines.extend("        %s," % entity["struct"] for entity in entities)
    lines.append("    ],")
    lines.append("}")
    lines.append("")
    by_path = sorted(entities, key=lambda entity: entity["path"])
    lines.append("pub(crate) static NAMES: [(&str, EntityKind); %d] = [" % len(entities))
    lines.extend('    ("%s", EntityKind::%s),' % (entity["path"], entity["struct"]) for entity in by_path)
    lines.append("];")
    lines.append("")
    eggs = sorted((entity["spawn_egg"], entity["struct"]) for entity in entities if entity["spawn_egg"] is not None)
    lines.append("pub(crate) static SPAWN_EGGS: [(u16, EntityKind); %d] = [" % len(eggs))
    lines.extend("    (%d, EntityKind::%s)," % egg for egg in eggs)
    lines.append("];")
    return "\n".join(lines) + "\n"


def mod_file(entities):
    ordered = sorted(entities, key=lambda entity: entity["path"])
    lines = ["mod %s;" % entity["path"] for entity in ordered]
    lines.append("")
    lines.extend("pub use %s::%s;" % (entity["path"], entity["struct"]) for entity in ordered)
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


def top(counter, limit):
    return sorted(counter.items(), key=lambda pair: (-pair[1], pair[0]))[:limit]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--dump", default=DEFAULT_DUMP)
    parser.add_argument("--registries", default=DEFAULT_REGISTRIES)
    parser.add_argument("--families", default=DEFAULT_FAMILIES)
    parser.add_argument("--out", default=DEFAULT_OUT)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--stats", action="store_true")
    args = parser.parse_args()

    dump = load_json(args.dump)
    registries = load_json(args.registries)
    categories = validate(dump, registries)
    entities = dump["entity_types"]
    by_class, by_entity = read_families(args.families, entities)

    types_dir = os.path.join(args.out, "types")
    if not args.check:
        os.makedirs(types_dir, exist_ok=True)
    stats = {"files": 0, "bytes": 0, "written": 0, "stale": []}
    expected_files = {"mod.rs"}
    families = {}
    for entity in entities:
        behavior = by_entity.get(entity["name"]) or by_class.get(entity["class"]) or "Plain"
        families[behavior] = families.get(behavior, 0) + 1
        file_name = entity["path"] + ".rs"
        expected_files.add(file_name)
        write(os.path.join(types_dir, file_name), entity_file(entity, categories, behavior), stats, args.check)
    write(os.path.join(types_dir, "mod.rs"), mod_file(entities), stats, args.check)
    write(os.path.join(args.out, "kind.rs"), kind_file(entities), stats, args.check)
    write(os.path.join(args.out, "categories.rs"), categories_file(categories), stats, args.check)
    removed = 0
    if os.path.isdir(types_dir):
        for entry in sorted(os.listdir(types_dir)):
            if entry.endswith(".rs") and entry not in expected_files:
                removed += 1
                if args.check:
                    stats["stale"].append(os.path.join(types_dir, entry))
                else:
                    os.remove(os.path.join(types_dir, entry))
    print(
        "entities %d categories %d classes %d living %d spawn_eggs %d families %d files %d bytes %d written %d removed %d"
        % (
            len(entities),
            len(categories),
            len({entity["class"] for entity in entities}),
            sum(1 for entity in entities if entity["living"]),
            sum(1 for entity in entities if entity["spawn_egg"] is not None),
            len(by_class) + len(by_entity),
            stats["files"],
            stats["bytes"],
            stats["written"],
            removed,
        )
    )
    if args.stats:
        per_category = {}
        per_class = {}
        for entity in entities:
            per_category[entity["category"]] = per_category.get(entity["category"], 0) + 1
            per_class[entity["class"]] = per_class.get(entity["class"], 0) + 1
        print("per category " + " ".join("%s=%d" % pair for pair in top(per_category, len(per_category))))
        print("per class top 30 " + " ".join("%s=%d" % pair for pair in top(per_class, 30)))
        print("per family " + " ".join("%s=%d" % pair for pair in top(families, len(families))))
    if args.check and stats["stale"]:
        for path in stats["stale"]:
            print("stale " + path, file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
