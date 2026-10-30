import argparse
import json
import math
import os
import re
import struct
import sys

RE_ROOT = "/Users/baptiste.parmantier/Development/rspigot/re"
DEFAULT_PROPERTIES = RE_ROOT + "/datagen/generated/reports/block_properties.json"
DEFAULT_BLOCKS = RE_ROOT + "/datagen/generated/reports/blocks.json"
CRATE = os.path.normpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "crates", "rspigot-blocks")
)
DEFAULT_FAMILIES = os.path.join(CRATE, "families.txt")
DEFAULT_OUT = os.path.join(CRATE, "src")
EXPECTED_VERSION = "26.3"

FLAG_SOURCES = {
    "AIR": "is_air",
    "BLOCKS_MOTION": "blocks_motion",
    "HAS_COLLISION": "has_collision",
    "CAN_OCCLUDE": "can_occlude",
    "SOLID_RENDER": "is_solid_render",
    "PROPAGATES_SKYLIGHT": "propagates_skylight_down",
    "REPLACEABLE": "can_be_replaced",
    "WATER": "water",
    "LAVA": "lava",
    "RANDOMLY_TICKS": "is_randomly_ticking",
    "FULL_COLLISION_CUBE": "is_collision_shape_full_block",
    "LEGACY_SOLID": "is_solid",
    "REDSTONE_CONDUCTOR": "is_redstone_conductor",
    "SIGNAL_SOURCE": "is_signal_source",
    "PATHFINDABLE_LAND": "pathfindable_land",
}

SPAWN_SURFACES = {
    "default": "Default",
    "never": "Never",
    "always": "Always",
    "ocelot_or_parrot": "OcelotOrParrot",
    "polar_bear": "PolarBear",
    "fire_immune": "FireImmune",
}

RESERVED_STRUCTS = {
    "Aabb", "Block", "BlockId", "BlockKind", "BlockSettings", "BlockBehavior", "Face",
    "FaceSupport", "Linked", "LinkedProperty", "PerState", "PlaceContext", "Placement",
    "Property", "ShapeId", "SpawnSurface", "StateFlags", "StateId", "Support", "UseContext", "UseResult", "Self", "Option", "Some", "None", "Result", "Ok",
    "Err", "Vec", "String", "Box", "Default", "Clone", "Copy", "Send", "Sync", "Sized",
    "Drop", "Fn", "FnMut", "FnOnce", "Iterator", "IntoIterator", "Extend", "ToString",
    "ToOwned", "PartialEq", "Eq", "PartialOrd", "Ord", "AsRef", "AsMut", "Into", "From",
    "TryFrom", "TryInto", "FromIterator", "DoubleEndedIterator", "ExactSizeIterator", "Unpin",
}

RUST_KEYWORDS = {
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
}

STURDY_KEYS = ("sturdy_full", "sturdy_center", "sturdy_rigid")
FACE_MASK = (1 << 6) - 1
EMPTY_SHAPE = ()
FULL_CUBE = ((0.0, 0.0, 0.0, 1.0, 1.0, 1.0),)
FAMILY_TYPE = re.compile(r"^[A-Z][A-Za-z0-9]*(<(true|false|[0-9]+)(, *(true|false|[0-9]+))*>)?$")
MODULE_NAME = re.compile(r"^[a-z][a-z0-9_]*$")
STRUCT_NAME = re.compile(r"^[A-Z][A-Za-z0-9]*$")
WIDTH = 100


def fail(message):
    sys.exit("gen_blocks: " + message)


def load_json(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def flag_bits(lib_rs):
    with open(lib_rs, encoding="utf-8") as handle:
        source = handle.read()
    block = re.search(r"impl StateFlags \{(.*?)\n\}", source, re.S)
    if not block:
        fail("impl StateFlags not found in " + lib_rs)
    bits = {}
    for name, expr in re.findall(r"pub const (\w+): Self = Self\(([^)]*)\);", block.group(1)):
        expr = expr.strip()
        shifted = re.fullmatch(r"1 << (\d+)", expr)
        if shifted:
            bits[name] = 1 << int(shifted.group(1))
        elif re.fullmatch(r"\d+", expr):
            bits[name] = int(expr)
        else:
            fail("unsupported StateFlags constant " + name + " = " + expr)
    if set(bits) != set(FLAG_SOURCES):
        fail("StateFlags constants " + str(sorted(bits)) + " != " + str(sorted(FLAG_SOURCES)))
    return {FLAG_SOURCES[name]: bit for name, bit in bits.items()}


def read_families(path, blocks):
    classes = {block["class"] for block in blocks}
    names = {block["name"] for block in blocks}
    by_class = {}
    by_block = {}
    if not os.path.exists(path):
        return by_class, by_block
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
        if key.startswith("block:"):
            name = key[len("block:"):]
            if ":" not in name:
                name = "minecraft:" + name
            if name not in names:
                fail("%s:%d: unknown block %r" % (path, number, name))
            if name in by_block:
                fail("%s:%d: duplicate block %r" % (path, number, name))
            by_block[name] = family
        else:
            if key not in classes:
                fail("%s:%d: unknown vanilla class %r" % (path, number, key))
            if key in by_class:
                fail("%s:%d: duplicate class %r" % (path, number, key))
            by_class[key] = family
    return by_class, by_block


def struct_name(path):
    return "".join(part[:1].upper() + part[1:] for part in path.split("_"))


def f32(value):
    return struct.unpack("<f", struct.pack("<f", value))[0]


def f32_literal(value):
    if not math.isfinite(value):
        fail("non finite float " + repr(value))
    target = f32(value)
    for digits in range(1, 10):
        candidate = "%.*g" % (digits, target)
        if f32(float(candidate)) == target:
            break
    text = repr(float(candidate))
    if not re.search(r"[.eE]", text):
        text += ".0"
    return text


def f64_literal(value):
    if not math.isfinite(value):
        fail("non finite double " + repr(value))
    text = repr(float(value) + 0.0)
    if not re.search(r"[.eE]", text):
        text += ".0"
    return text


def read_shapes(dump):
    shapes = []
    for index, raw in enumerate(dump.get("collision_shapes", ())):
        boxes = []
        for box in raw:
            if len(box) != 6 or not all(isinstance(value, (int, float)) for value in box):
                fail("collision shape %d has a malformed box %r" % (index, box))
            box = tuple(float(value) + 0.0 for value in box)
            if not all(math.isfinite(value) for value in box):
                fail("collision shape %d has a non finite box %r" % (index, box))
            if not all(box[axis] < box[axis + 3] for axis in range(3)):
                fail("collision shape %d has an empty box %r" % (index, box))
            boxes.append(box)
        shapes.append(tuple(boxes))
    if not shapes:
        fail("dump has no collision_shapes")
    return shapes


def intern_shapes(blocks, shapes):
    table = [EMPTY_SHAPE, FULL_CUBE]
    ids = {shape: index for index, shape in enumerate(table)}
    for block in blocks:
        collision = []
        support = []
        for state in block["states"]:
            index = state.get("collision_shape")
            if not isinstance(index, int) or isinstance(index, bool) or not 0 <= index < len(shapes):
                fail("%s state %d has collision shape %r" % (block["name"], state["id"], index))
            shape = shapes[index]
            if state["has_collision"] != bool(shape):
                fail("%s state %d has_collision disagrees with its shape" % (block["name"], state["id"]))
            if state["is_collision_shape_full_block"] != (shape == FULL_CUBE):
                fail("%s state %d full block flag disagrees with its shape" % (block["name"], state["id"]))
            if shape not in ids:
                ids[shape] = len(table)
                table.append(shape)
            collision.append(ids[shape])
            masks = []
            for key in STURDY_KEYS:
                mask = state.get(key)
                if not isinstance(mask, int) or isinstance(mask, bool) or mask & ~FACE_MASK:
                    fail("%s state %d has %s %r" % (block["name"], state["id"], key, mask))
                masks.append(mask)
            if masks[0] & ~masks[1] or masks[0] & ~masks[2]:
                fail("%s state %d is full sturdy without center or rigid" % (block["name"], state["id"]))
            support.append(tuple(masks))
        block["collision"] = collision
        block["support"] = support
    if len(table) > 1 << 16:
        fail("too many collision shapes: %d" % len(table))
    return table


def share_per_state(blocks):
    shared = {"collision": {}, "support": {}}
    for block in blocks:
        for field, prefix in (("collision", "COLLISION"), ("support", "SUPPORT")):
            values = tuple(block[field])
            block[field + "_shared"] = None
            if all(value == values[0] for value in values):
                continue
            names = shared[field]
            if values not in names:
                names[values] = "%s_%s" % (prefix, block["path"].upper())
            block[field + "_shared"] = names[values]
    return shared


def render_shape_id(value):
    return "ShapeId(%d)" % value


def render_support(masks):
    return "FaceSupport(%d, %d, %d)" % masks


def aabb_literal(box):
    return "Aabb::new(%s)" % ", ".join(f64_literal(value) for value in box)


def shapes_file(table, shared):
    lines = [
        "use crate::{Aabb, FaceSupport, ShapeId};",
        "",
        "pub const SHAPE_COUNT: usize = %d;" % len(table),
        "",
        "pub static SHAPES: [&[Aabb]; SHAPE_COUNT] = [",
    ]
    for shape in table:
        boxes = [aabb_literal(box) for box in shape]
        inline = "    &[%s]," % ", ".join(boxes)
        if len(inline) <= WIDTH:
            lines.append(inline)
        else:
            lines.append("    &[")
            lines.extend("        %s," % box for box in boxes)
            lines.append("    ],")
    lines.append("];")
    for field, type_name, render in (
        ("collision", "ShapeId", render_shape_id),
        ("support", "FaceSupport", render_support),
    ):
        for values, name in shared[field].items():
            lines.append("")
            lines.append("pub(crate) const %s: &[%s] = &[" % (name, type_name))
            lines.extend(wrap([render(value) for value in values], "    "))
            lines.append("];")
    return "\n".join(lines) + "\n"


def use_line(names):
    line = "use crate::{%s};" % ", ".join(names)
    if len(line) <= WIDTH:
        return [line]
    return ["use crate::{"] + wrap(names, "    ") + ["};"]


def decompose(offset, properties):
    values = []
    for index, (name, options) in enumerate(properties):
        stride = 1
        for _, later in properties[index + 1:]:
            stride *= len(later)
        values.append((name, options[(offset // stride) % len(options)]))
    return values


def validate(dump, reference):
    if dump.get("version") != EXPECTED_VERSION:
        fail("dump version %r != %r" % (dump.get("version"), EXPECTED_VERSION))
    blocks = dump["blocks"]
    if set(reference) != {block["name"] for block in blocks} or len(reference) != len(blocks):
        fail("block names differ from blocks.json")
    next_state = 0
    for index, block in enumerate(blocks):
        name = block["name"]
        if block["id"] != index:
            fail("%s registry id %d != %d" % (name, block["id"], index))
        properties = [(prop["name"], prop["values"]) for prop in block["properties"]]
        property_names = [prop_name for prop_name, _ in properties]
        if property_names != sorted(property_names):
            fail("%s properties not alphabetical: %s" % (name, property_names))
        expected = reference[name]
        if expected.get("properties", {}) != dict(properties):
            fail("%s properties differ from blocks.json" % name)
        states = block["states"]
        if len(states) != len(expected["states"]):
            fail("%s state count differs from blocks.json" % name)
        combinations = 1
        for _, options in properties:
            combinations *= len(options)
        if combinations != len(states):
            fail("%s has %d states for %d combinations" % (name, len(states), combinations))
        first = next_state
        for offset, state in enumerate(states):
            if state["id"] != first + offset:
                fail("%s state %d expected id %d" % (name, state["id"], first + offset))
            if list(state["properties"].items()) != decompose(offset, properties):
                fail("%s state %d is not last-property-fastest" % (name, state["id"]))
        reference_states = {}
        for state in expected["states"]:
            reference_states[state["id"]] = state.get("properties", {})
            if state.get("default") and state["id"] != block["default_state"]:
                fail("%s default state %d != blocks.json %d" % (name, block["default_state"], state["id"]))
        for state in states:
            if reference_states.get(state["id"]) != state["properties"]:
                fail("%s state %d differs from blocks.json" % (name, state["id"]))
        if not first <= block["default_state"] < first + len(states):
            fail("%s default state outside its range" % name)
        next_state += len(states)
    block_ids = {block["id"] for block in blocks}
    for entry in dump["block_items"]:
        if entry["block"] not in block_ids:
            fail("item %s points to unknown block %d" % (entry["name"], entry["block"]))
        if blocks[entry["block"]]["item"] != entry["id"]:
            fail("item %s block %s has item %r" % (entry["name"], blocks[entry["block"]]["name"], blocks[entry["block"]]["item"]))
    return next_state


def wrap(items, indent):
    lines = []
    current = ""
    for item in items:
        piece = item + ","
        if current and len(indent) + len(current) + 1 + len(piece) > WIDTH:
            lines.append(indent + current)
            current = piece
        else:
            current = piece if not current else current + " " + piece
    if current:
        lines.append(indent + current)
    return lines


def shared_per_state(name, type_name, values, render, shared):
    if shared is None:
        return per_state(name, type_name, values, render)
    head = "    const %s: PerState<%s> =" % (name, type_name)
    body = "PerState::Varies(crate::shapes::%s);" % shared
    if len(head) + 1 + len(body) <= WIDTH:
        return [head + " " + body]
    return [head, "        " + body]


def per_state(name, type_name, values, render):
    if all(value == values[0] for value in values):
        return ["    const %s: PerState<%s> = PerState::Uniform(%s);" % (name, type_name, render(values[0]))]
    lines = ["    const %s: PerState<%s> = PerState::Varies(&[" % (name, type_name)]
    lines.extend(wrap([render(value) for value in values], "        "))
    lines.append("    ]);")
    return lines


def rust_string(value):
    if not re.fullmatch(r"[a-z0-9_:]+", value):
        fail("unexpected string " + repr(value))
    return '"' + value + '"'


def block_file(block, first_state, item, wall, behavior, bits):
    name = block["name"]
    struct = block["struct"]
    states = block["states"]
    surface = SPAWN_SURFACES.get(block["spawn_surface"])
    if surface is None:
        fail("%s has unknown spawn surface %r" % (name, block["spawn_surface"]))
    imports = [
        "Block", "BlockId", "BlockSettings", "FaceSupport", "PerState", "Property", "ShapeId",
        "StateFlags", "StateId",
    ]
    if surface != "Default":
        imports.insert(imports.index("StateFlags"), "SpawnSurface")
    lines = use_line(imports)
    lines.extend(["", "pub struct %s;" % struct, ""])
    lines.append("impl Block for %s {" % struct)
    lines.append("    const NAME: &'static str = %s;" % rust_string(name))
    lines.append("    const ID: BlockId = BlockId(%d);" % block["id"])
    lines.append("    const FIRST_STATE: StateId = StateId(%d);" % first_state)
    lines.append("    const STATE_COUNT: u16 = %d;" % len(states))
    lines.append("    const DEFAULT_STATE: StateId = StateId(%d);" % block["default_state"])
    if block["properties"]:
        lines.append("    const PROPERTIES: &'static [Property] = &[")
        for prop in block["properties"]:
            values = ", ".join(rust_string(value) for value in prop["values"])
            line = "        Property { name: %s, values: &[%s] }," % (rust_string(prop["name"]), values)
            if len(line) <= WIDTH:
                lines.append(line)
            else:
                lines.append("        Property {")
                lines.append("            name: %s," % rust_string(prop["name"]))
                lines.append("            values: &[")
                lines.extend(wrap([rust_string(value) for value in prop["values"]], "                "))
                lines.append("            ],")
                lines.append("        },")
        lines.append("    ];")
    else:
        lines.append("    const PROPERTIES: &'static [Property] = &[];")
    lines.append("    const ITEM: Option<u16> = %s;" % ("None" if item is None else "Some(%d)" % item))
    lines.append(
        "    const WALL_VARIANT: Option<BlockId> = %s;" % ("None" if wall is None else "Some(BlockId(%d))" % wall)
    )
    lines.append("    const SETTINGS: BlockSettings = BlockSettings {")
    lines.append("        hardness: %s," % f32_literal(block["destroy_time"]))
    lines.append("        resistance: %s," % f32_literal(block["explosion_resistance"]))
    lines.append("        friction: %s," % f32_literal(block["friction"]))
    lines.append("        speed_factor: %s," % f32_literal(block["speed_factor"]))
    lines.append("        jump_factor: %s," % f32_literal(block["jump_factor"]))
    lines.append("        bounce_restitution: %s," % f32_literal(block["bounce_restitution"]))
    lines.append("        fall_distance_reduction: %s," % f32_literal(block["fall_distance_reduction"]))
    lines.append("        requires_correct_tool: %s," % str(bool(block["requires_correct_tool_for_drops"])).lower())
    lines.append("        ignited_by_lava: %s," % str(bool(block["ignited_by_lava"])).lower())
    lines.append("    };")
    emission = [state["light_emission"] for state in states]
    dampening = [state["light_dampening"] for state in states]
    for value in emission + dampening:
        if not 0 <= value <= 15:
            fail("%s light value %d out of range" % (name, value))
    flags = []
    for state in states:
        value = 0
        for key, bit in bits.items():
            if not isinstance(state[key], bool):
                fail("%s state %d flag %s is not a boolean" % (name, state["id"], key))
            if state[key]:
                value |= bit
        flags.append(value)
    lines.extend(per_state("LIGHT_EMISSION", "u8", emission, str))
    lines.extend(per_state("LIGHT_DAMPENING", "u8", dampening, str))
    lines.extend(per_state("FLAGS", "StateFlags", flags, lambda value: "StateFlags(%d)" % value))
    lines.extend(
        shared_per_state("COLLISION", "ShapeId", block["collision"], render_shape_id, block["collision_shared"])
    )
    lines.extend(
        shared_per_state("FACE_SUPPORT", "FaceSupport", block["support"], render_support, block["support_shared"])
    )
    if surface != "Default":
        lines.append("    const SPAWN_SURFACE: SpawnSurface = SpawnSurface::%s;" % surface)
    lines.append("    type Behavior = crate::family::%s;" % behavior)
    lines.append("}")
    return "\n".join(lines) + "\n"


def kind_file(blocks, state_count, item_table):
    lines = ["crate::state::block_kinds! {"]
    lines.append("    blocks: %d," % len(blocks))
    lines.append("    states: %d," % state_count)
    lines.append("    kinds: [")
    lines.extend("        %s," % block["struct"] for block in blocks)
    lines.append("    ],")
    lines.append("}")
    lines.append("")
    by_path = sorted(blocks, key=lambda block: block["path"])
    lines.append("pub(crate) static NAMES: [(&str, BlockKind); %d] = [" % len(blocks))
    lines.extend('    ("%s", BlockKind::%s),' % (block["path"], block["struct"]) for block in by_path)
    lines.append("];")
    lines.append("")
    lines.append("pub(crate) static ITEMS: [Option<BlockKind>; %d] = [" % len(item_table))
    for block in item_table:
        lines.append("    None," if block is None else "    Some(BlockKind::%s)," % block["struct"])
    lines.append("];")
    return "\n".join(lines) + "\n"


def mod_file(blocks):
    ordered = sorted(blocks, key=lambda block: block["path"])
    lines = ["mod %s;" % block["path"] for block in ordered]
    lines.append("")
    lines.extend("pub use %s::%s;" % (block["path"], block["struct"]) for block in ordered)
    return "\n".join(lines) + "\n"


def write(path, content, stats):
    stats["files"] += 1
    stats["bytes"] += len(content.encode("utf-8"))
    if os.path.exists(path):
        with open(path, encoding="utf-8") as handle:
            if handle.read() == content:
                return
    stats["written"] += 1
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(content)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--properties", default=DEFAULT_PROPERTIES)
    parser.add_argument("--blocks", default=DEFAULT_BLOCKS)
    parser.add_argument("--families", default=DEFAULT_FAMILIES)
    parser.add_argument("--out", default=DEFAULT_OUT)
    args = parser.parse_args()

    dump = load_json(args.properties)
    reference = load_json(args.blocks)
    blocks = dump["blocks"]
    state_count = validate(dump, reference)
    bits = flag_bits(os.path.join(args.out, "lib.rs"))
    by_class, by_block = read_families(args.families, blocks)

    structs = set()
    for block in blocks:
        namespace, _, path = block["name"].partition(":")
        if namespace != "minecraft" or not MODULE_NAME.match(path) or path in RUST_KEYWORDS:
            fail("unsupported block name " + block["name"])
        block["path"] = path
        block["struct"] = struct_name(path)
        if not STRUCT_NAME.match(block["struct"]) or block["struct"] in RESERVED_STRUCTS:
            fail("unsupported struct name %s for %s" % (block["struct"], block["name"]))
        if block["struct"] in structs:
            fail("duplicate struct name " + block["struct"])
        structs.add(block["struct"])

    shape_table = intern_shapes(blocks, read_shapes(dump))
    shared = share_per_state(blocks)

    item_max = max((entry["id"] for entry in dump["block_items"]), default=-1)
    item_table = [None] * (item_max + 1)
    for entry in dump["block_items"]:
        item_table[entry["id"]] = blocks[entry["block"]]

    blocks_dir = os.path.join(args.out, "blocks")
    os.makedirs(blocks_dir, exist_ok=True)
    stats = {"files": 0, "bytes": 0, "written": 0}
    expected_files = {"mod.rs"}
    for block in blocks:
        wall = block["wall_variant"]
        if block["item_block"] != block["id"] or wall == block["id"]:
            wall = None
        behavior = by_block.get(block["name"]) or by_class.get(block["class"]) or "Plain"
        content = block_file(block, block["states"][0]["id"], block["item"], wall, behavior, bits)
        file_name = block["path"] + ".rs"
        expected_files.add(file_name)
        write(os.path.join(blocks_dir, file_name), content, stats)
    write(os.path.join(blocks_dir, "mod.rs"), mod_file(blocks), stats)
    write(os.path.join(args.out, "kind.rs"), kind_file(blocks, state_count, item_table), stats)
    write(os.path.join(args.out, "shapes.rs"), shapes_file(shape_table, shared), stats)
    removed = 0
    for entry in sorted(os.listdir(blocks_dir)):
        if entry.endswith(".rs") and entry not in expected_files:
            os.remove(os.path.join(blocks_dir, entry))
            removed += 1
    print(
        "blocks %d states %d shapes %d boxes %d shared %d classes %d families %d files %d bytes %d written %d removed %d"
        % (
            len(blocks),
            state_count,
            len(shape_table),
            sum(len(shape) for shape in shape_table),
            len(shared["collision"]) + len(shared["support"]),
            len({block["class"] for block in blocks}),
            len(by_class) + len(by_block),
            stats["files"],
            stats["bytes"],
            stats["written"],
            removed,
        )
    )


if __name__ == "__main__":
    main()
