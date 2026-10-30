import argparse
import json
import math
import os
import re
import struct
import sys

RE_ROOT = "/Users/baptiste.parmantier/Development/rspigot/re"
REPORTS = RE_ROOT + "/datagen/generated/reports"
DEFAULT_DUMP = REPORTS + "/items.json"
DEFAULT_REGISTRIES = REPORTS + "/registries.json"
DEFAULT_COMPONENTS = REPORTS + "/minecraft/components/item"
CRATE = os.path.normpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "crates", "rspigot-items")
)
DEFAULT_FAMILIES = os.path.join(CRATE, "families.txt")
DEFAULT_OUT = os.path.join(CRATE, "src")
EXPECTED_VERSION = "26.3"
NAMESPACE = "minecraft:"
BLOCK_FAMILY = "BlockItem"
DEFAULT_FAMILY = "Plain"

RARITIES = ["common", "uncommon", "rare", "epic"]
EQUIPMENT_SLOTS = ["mainhand", "offhand", "feet", "legs", "chest", "head", "body", "saddle"]

RESERVED_STRUCTS = {
    "Item", "ItemId", "ItemKind", "ItemBehavior", "ItemSet", "ItemTag", "BlockSet", "BlockTag",
    "BlockId", "BlockKind", "Tool", "ToolRule", "Food", "Rarity", "EquipmentSlot", "Plain",
    "BlockItem", "Self", "Option", "Some", "None", "Result", "Ok", "Err", "Vec", "Box",
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
TAG_NAME = re.compile(r"^minecraft:[a-z0-9_/]+$")
WIDTH = 100


def fail(message):
    sys.exit("gen_items: " + message)


def load_json(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def struct_name(path):
    return "".join(part[:1].upper() + part[1:] for part in path.split("_"))


def f32(value):
    try:
        return struct.unpack("<f", struct.pack("<f", value))[0]
    except OverflowError:
        return math.copysign(math.inf, value)


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
    mantissa, _, exponent = candidate.lower().partition("e")
    if "." not in mantissa:
        mantissa += ".0"
    text = mantissa + ("e" + str(int(exponent)) if exponent else "")
    if not re.fullmatch(r"-?[0-9]+\.[0-9]+(e[0-9]+)?", text) or f32(float(text)) != target:
        fail("%s float %r has no literal form" % (what, value))
    return text


def rust_string(value):
    if not re.fullmatch(r"[a-z0-9_:/]+", value):
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


def tag_constant(prefix, name):
    if not TAG_NAME.match(name):
        fail("unsupported tag name " + repr(name))
    return prefix + "_" + re.sub(r"[^a-z0-9]", "_", name[len(NAMESPACE):]).upper()


def read_families(path, items):
    classes = {item["class"] for item in items}
    names = {item["name"] for item in items}
    by_class = {}
    by_item = {}
    if not os.path.exists(path):
        return by_class, by_item
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
        if key.startswith("item:"):
            name = key[len("item:"):]
            if ":" not in name:
                name = NAMESPACE + name
            if name not in names:
                fail("%s:%d: unknown item %r" % (path, number, name))
            if name in by_item:
                fail("%s:%d: duplicate item %r" % (path, number, name))
            by_item[name] = family
        else:
            if key not in classes:
                fail("%s:%d: unknown vanilla class %r" % (path, number, key))
            if key in by_class:
                fail("%s:%d: duplicate class %r" % (path, number, key))
            by_class[key] = family
    return by_class, by_item


def report_set(value, names, what):
    if isinstance(value, str) and value.startswith("#"):
        return {"tag": value[1:] if ":" in value else NAMESPACE + value[1:]}
    values = [value] if isinstance(value, str) else value
    if not isinstance(values, list):
        fail("%s unsupported holder set %r" % (what, value))
    ids = []
    for entry in values:
        entry = entry if ":" in entry else NAMESPACE + entry
        if entry not in names:
            fail("%s unknown entry %r" % (what, entry))
        ids.append(names[entry])
    return {"ids": ids}


def dump_set(value, key, what):
    if set(value) == {"tag"}:
        return {"tag": value["tag"]}
    if set(value) == {key}:
        return {"ids": value[key]}
    fail("%s unsupported dumped set %r" % (what, value))


def same_float(left, right):
    return f32(finite(left, "report")) == f32(finite(right, "dump"))


def check_report(item, report, block_ids, item_ids):
    name = item["name"]
    components = report["components"]

    def mismatch(component, expected, actual):
        fail("%s %s: report %r, dump %r" % (name, component, expected, actual))

    if components.get("minecraft:max_stack_size") != item["max_stack_size"]:
        mismatch("max_stack_size", components.get("minecraft:max_stack_size"), item["max_stack_size"])
    if components.get("minecraft:max_damage") != item["max_damage"]:
        mismatch("max_damage", components.get("minecraft:max_damage"), item["max_damage"])
    if components.get("minecraft:rarity") != item["rarity"]:
        mismatch("rarity", components.get("minecraft:rarity"), item["rarity"])
    tool = components.get("minecraft:tool")
    if (tool is None) != (item["tool"] is None):
        mismatch("tool", tool, item["tool"])
    if tool is not None:
        dumped = item["tool"]
        if len(tool.get("rules", [])) != len(dumped["rules"]):
            mismatch("tool rules", tool.get("rules"), dumped["rules"])
        for rule, mine in zip(tool.get("rules", []), dumped["rules"]):
            if report_set(rule["blocks"], block_ids, name) != dump_set(mine["blocks"], "blocks", name):
                mismatch("tool rule blocks", rule["blocks"], mine["blocks"])
            if ("speed" in rule) != (mine["speed"] is not None):
                mismatch("tool rule speed", rule.get("speed"), mine["speed"])
            if "speed" in rule and not same_float(rule["speed"], mine["speed"]):
                mismatch("tool rule speed", rule["speed"], mine["speed"])
            if rule.get("correct_for_drops") != mine["correct_for_drops"]:
                mismatch("tool rule drops", rule.get("correct_for_drops"), mine["correct_for_drops"])
        if "default_mining_speed" in tool and not same_float(tool["default_mining_speed"], dumped["default_mining_speed"]):
            mismatch("default_mining_speed", tool["default_mining_speed"], dumped["default_mining_speed"])
        for key in ("damage_per_block", "can_destroy_blocks_in_creative"):
            if key in tool and tool[key] != dumped[key]:
                mismatch(key, tool[key], dumped[key])
    food = components.get("minecraft:food")
    if (food is None) != (item["food"] is None):
        mismatch("food", food, item["food"])
    if food is not None:
        if food["nutrition"] != item["food"]["nutrition"]:
            mismatch("nutrition", food["nutrition"], item["food"]["nutrition"])
        if not same_float(food["saturation"], item["food"]["saturation"]):
            mismatch("saturation", food["saturation"], item["food"]["saturation"])
        if "can_always_eat" in food and food["can_always_eat"] != item["food"]["can_always_eat"]:
            mismatch("can_always_eat", food["can_always_eat"], item["food"]["can_always_eat"])
    equippable = components.get("minecraft:equippable")
    if (None if equippable is None else equippable["slot"]) != item["equippable"]:
        mismatch("equippable", equippable, item["equippable"])
    enchantable = components.get("minecraft:enchantable")
    if (None if enchantable is None else enchantable["value"]) != item["enchantable"]:
        mismatch("enchantable", enchantable, item["enchantable"])
    repairable = components.get("minecraft:repairable")
    if (repairable is None) != (item["repairable"] is None):
        mismatch("repairable", repairable, item["repairable"])
    if repairable is not None:
        if report_set(repairable["items"], item_ids, name) != dump_set(item["repairable"], "items", name):
            mismatch("repairable items", repairable["items"], item["repairable"])
    resistant = components.get("minecraft:damage_resistant")
    if (resistant is None) != (item["damage_resistant"] is None):
        mismatch("damage_resistant", resistant, item["damage_resistant"])
    if resistant is not None and resistant["types"] != "#" + item["damage_resistant"]["tag"]:
        mismatch("damage_resistant", resistant, item["damage_resistant"])
    remainder = components.get("minecraft:use_remainder")
    if (remainder is None) != (item["use_remainder"] is None):
        mismatch("use_remainder", remainder, item["use_remainder"])
    if remainder is not None:
        if set(remainder) != {"id"} or item_ids.get(remainder["id"]) != item["use_remainder"]["item"]:
            mismatch("use_remainder", remainder, item["use_remainder"])


def check_tool_semantics(item, block_tags, block_count):
    tool = item["tool"]

    def contains(rule, block):
        blocks = rule["blocks"]
        if "tag" in blocks:
            return block in block_tags[blocks["tag"]]
        return block in blocks["blocks"]

    correct = []
    speeds = []
    default = f32(tool["default_mining_speed"])
    for block in range(block_count):
        drops = next(
            (rule["correct_for_drops"] for rule in tool["rules"] if rule["correct_for_drops"] is not None and contains(rule, block)),
            False,
        )
        if drops:
            correct.append(block)
        speed = next(
            (f32(rule["speed"]) for rule in tool["rules"] if rule["speed"] is not None and contains(rule, block)),
            default,
        )
        if speed != default:
            speeds.append([block, speed])
    if correct != tool["correct_for_drops"]:
        fail("%s correct_for_drops disagrees with Tool.isCorrectForDrops" % item["name"])
    dumped = [[block, f32(speed)] for block, speed in tool["mining_speeds"]]
    if speeds != dumped:
        fail("%s mining speeds disagree with Tool.getMiningSpeed" % item["name"])


def validate(dump, registries, components_dir):
    if dump.get("version") != EXPECTED_VERSION:
        fail("dump version %r != %r" % (dump.get("version"), EXPECTED_VERSION))
    if dump["rarities"] != RARITIES:
        fail("vanilla rarities changed: %r" % dump["rarities"])
    if dump["equipment_slots"] != EQUIPMENT_SLOTS:
        fail("vanilla equipment slots changed: %r" % dump["equipment_slots"])
    item_names = registry_names(registries, "minecraft:item")
    block_names = registry_names(registries, "minecraft:block")
    integer(dump["absolute_max_stack_size"], 1, 255, "absolute max stack size")
    integer(dump["default_max_stack_size"], 1, dump["absolute_max_stack_size"], "default max stack size")
    if dump["block_count"] != len(block_names):
        fail("dump block count %r != registry %d" % (dump["block_count"], len(block_names)))
    items = dump["items"]
    if [item["name"] for item in items] != item_names:
        fail("item names or order differ from registries.json")
    report_files = {entry[: -len(".json")] for entry in os.listdir(components_dir) if entry.endswith(".json")}
    if report_files != {name[len(NAMESPACE):] for name in item_names}:
        fail("component reports differ from the item registry")
    block_ids = {name: index for index, name in enumerate(block_names)}
    item_ids = {name: index for index, name in enumerate(item_names)}
    block_tags = {}
    for tag in dump["block_tags"]:
        tag_constant("BLOCK", tag["name"])
        if tag["blocks"] != sorted(set(tag["blocks"])):
            fail("block tag %s is not sorted" % tag["name"])
        for block in tag["blocks"]:
            integer(block, 0, len(block_names) - 1, tag["name"])
        block_tags[tag["name"]] = set(tag["blocks"])
    item_tags = {}
    for tag in dump["item_tags"]:
        tag_constant("ITEM", tag["name"])
        if tag["items"] != sorted(set(tag["items"])):
            fail("item tag %s is not sorted" % tag["name"])
        for entry in tag["items"]:
            integer(entry, 0, len(item_names) - 1, tag["name"])
        item_tags[tag["name"]] = set(tag["items"])
    structs = set()
    placed = {}
    for index, item in enumerate(items):
        name = item["name"]
        if item["id"] != index:
            fail("%s registry id %r != %d" % (name, item["id"], index))
        if not name.startswith(NAMESPACE):
            fail("unsupported item namespace " + name)
        path = name[len(NAMESPACE):]
        if not MODULE_NAME.match(path) or path in RUST_KEYWORDS or path == "mod":
            fail("unsupported item name " + name)
        item["path"] = path
        item["struct"] = struct_name(path)
        if not STRUCT_NAME.match(item["struct"]) or item["struct"] in RESERVED_STRUCTS:
            fail("unsupported struct name %s for %s" % (item["struct"], name))
        if item["struct"] in structs:
            fail("duplicate struct name " + item["struct"])
        structs.add(item["struct"])
        if not CLASS_NAME.match(item["class"]):
            fail("%s has unsupported class %r" % (name, item["class"]))
        integer(item["max_stack_size"], 1, dump["absolute_max_stack_size"], name + " max stack size")
        if item["default_max_stack_size"] != item["max_stack_size"]:
            fail("%s default max stack size disagrees with its component" % name)
        if item["max_damage"] is not None:
            integer(item["max_damage"], 1, 2**31 - 1, name + " max damage")
        if item["rarity"] not in RARITIES:
            fail("%s has unknown rarity %r" % (name, item["rarity"]))
        boolean(item["block_item"], name + " block item")
        if item["block_item"] != (item["block"] is not None):
            fail("%s block item flag disagrees with its block" % name)
        if item["block"] is not None:
            integer(item["block"], 0, len(block_names) - 1, name + " block")
            if item["block"] in placed:
                fail("%s and %s place the same block" % (placed[item["block"]], name))
            placed[item["block"]] = name
        if item["wall_block"] is not None:
            integer(item["wall_block"], 0, len(block_names) - 1, name + " wall block")
        tool = item["tool"]
        if tool is not None:
            for rule in tool["rules"]:
                blocks = dump_set(rule["blocks"], "blocks", name)
                if "tag" in blocks and blocks["tag"] not in block_tags:
                    fail("%s references undumped block tag %s" % (name, blocks["tag"]))
                if "ids" in blocks:
                    if not blocks["ids"]:
                        fail("%s has an empty direct block set" % name)
                    for block in blocks["ids"]:
                        integer(block, 0, len(block_names) - 1, name + " tool block")
                if rule["speed"] is not None:
                    finite(rule["speed"], name + " tool speed")
                if rule["correct_for_drops"] is not None:
                    boolean(rule["correct_for_drops"], name + " tool drops")
            finite(tool["default_mining_speed"], name + " default mining speed")
            integer(tool["damage_per_block"], 0, 2**31 - 1, name + " damage per block")
            boolean(tool["can_destroy_blocks_in_creative"], name + " creative")
            check_tool_semantics(item, block_tags, len(block_names))
        food = item["food"]
        if food is not None:
            integer(food["nutrition"], 0, 2**31 - 1, name + " nutrition")
            finite(food["saturation"], name + " saturation")
            boolean(food["can_always_eat"], name + " can always eat")
        if item["equippable"] is not None and item["equippable"] not in EQUIPMENT_SLOTS:
            fail("%s has unknown equipment slot %r" % (name, item["equippable"]))
        if item["enchantable"] is not None:
            integer(item["enchantable"], 1, 2**31 - 1, name + " enchantable")
        if item["repairable"] is not None:
            repair = dump_set(item["repairable"], "items", name)
            if "tag" in repair and repair["tag"] not in item_tags:
                fail("%s references undumped item tag %s" % (name, repair["tag"]))
            for entry in repair.get("ids", []):
                integer(entry, 0, len(item_names) - 1, name + " repair item")
        if item["damage_resistant"] is not None:
            tag = item["damage_resistant"]["tag"]
            if not TAG_NAME.match(tag):
                fail("%s has unsupported damage tag %r" % (name, tag))
        remainder = item["use_remainder"]
        if remainder is not None:
            integer(remainder["item"], 0, len(item_names) - 1, name + " use remainder")
            if remainder["count"] != 1 or remainder["patch_empty"] is not True:
                fail("%s use remainder is not a plain single item" % name)
        report = load_json(os.path.join(components_dir, path + ".json"))
        check_report(item, report, block_ids, item_ids)
    return block_names


def option(value, render):
    return "None" if value is None else "Some(%s)" % render(value)


def wrap(tokens, indent):
    lines = []
    current = ""
    for token in tokens:
        piece = token + ","
        if current and len(indent) + len(current) + 1 + len(piece) > WIDTH:
            lines.append(indent + current)
            current = piece
        else:
            current = piece if not current else current + " " + piece
    if current:
        lines.append(indent + current)
    return lines


def assignment(name, type_name, value):
    line = "    const %s: %s = %s;" % (name, type_name, value)
    if len(line) <= WIDTH:
        return [line]
    return ["    const %s: %s =" % (name, type_name), "        %s;" % value]


def block_set(value, used):
    if "tag" in value:
        used.add("tags")
        return "BlockSet::Tag(&tags::%s)" % tag_constant("BLOCK", value["tag"])
    used.add("BlockId")
    return "BlockSet::Direct(&[%s])" % ", ".join("BlockId(%d)" % block for block in sorted(set(value["blocks"])))


def item_set(value, used):
    if "tag" in value:
        used.add("tags")
        return "ItemSet::Tag(&tags::%s)" % tag_constant("ITEM", value["tag"])
    return "ItemSet::Direct(&[%s])" % ", ".join("ItemId(%d)" % entry for entry in sorted(set(value["items"])))


def tool_lines(tool, name, used):
    if tool is None:
        return ["    const TOOL: Option<Tool> = None;"]
    lines = ["    const TOOL: Option<Tool> = Some(Tool {"]
    if tool["rules"]:
        used.add("BlockSet")
        used.add("ToolRule")
        lines.append("        rules: &[")
        for rule in tool["rules"]:
            lines.append("            ToolRule {")
            lines.append("                blocks: %s," % block_set(rule["blocks"], used))
            speed = option(rule["speed"], lambda value: f32_literal(value, name + " speed"))
            lines.append("                speed: %s," % speed)
            drops = option(rule["correct_for_drops"], lambda value: boolean(value, name))
            lines.append("                correct_for_drops: %s," % drops)
            lines.append("            },")
        lines.append("        ],")
    else:
        lines.append("        rules: &[],")
    lines.append("        default_mining_speed: %s," % f32_literal(tool["default_mining_speed"], name))
    lines.append("        damage_per_block: %d," % tool["damage_per_block"])
    lines.append(
        "        can_destroy_blocks_in_creative: %s,"
        % boolean(tool["can_destroy_blocks_in_creative"], name)
    )
    lines.append("    });")
    return lines


def food_lines(food, name):
    if food is None:
        return ["    const FOOD: Option<Food> = None;"]
    return [
        "    const FOOD: Option<Food> = Some(Food {",
        "        nutrition: %d," % food["nutrition"],
        "        saturation: %s," % f32_literal(food["saturation"], name + " saturation"),
        "        can_always_eat: %s," % boolean(food["can_always_eat"], name),
        "    });",
    ]


def item_file(item, behavior):
    name = item["name"]
    used = set()
    body = [
        "    const NAME: &'static str = %s;" % rust_string(name),
        "    const ID: ItemId = ItemId(%d);" % item["id"],
        '    const CLASS: &\'static str = "%s";' % item["class"],
        "    const MAX_STACK_SIZE: u8 = %d;" % item["max_stack_size"],
        "    const MAX_DAMAGE: Option<u32> = %s;" % option(item["max_damage"], str),
        "    const RARITY: Rarity = Rarity::%s;" % struct_name(item["rarity"]),
        "    const BLOCK: Option<BlockId> = %s;" % option(item["block"], lambda block: "BlockId(%d)" % block),
    ]
    body.extend(tool_lines(item["tool"], name, used))
    body.extend(food_lines(item["food"], name))
    body.append(
        "    const EQUIPPABLE: Option<EquipmentSlot> = %s;"
        % option(item["equippable"], lambda slot: "EquipmentSlot::%s" % struct_name(slot))
    )
    body.append("    const ENCHANTABLE: Option<u32> = %s;" % option(item["enchantable"], str))
    repairable = option(item["repairable"], lambda value: item_set(value, used))
    body.extend(assignment("REPAIRABLE", "Option<ItemSet>", repairable))
    resistant = option(item["damage_resistant"], lambda value: rust_string(value["tag"]))
    body.append("    const DAMAGE_RESISTANT: Option<&'static str> = %s;" % resistant)
    remainder = option(item["use_remainder"], lambda value: "ItemId(%d)" % value["item"])
    body.append("    const USE_REMAINDER: Option<ItemId> = %s;" % remainder)
    body.append("    type Behavior = crate::family::%s;" % behavior)
    names = ["EquipmentSlot", "Food", "Item", "ItemId", "ItemSet", "Rarity", "Tool"]
    names.extend(name for name in ("BlockSet", "ToolRule") if name in used)
    if "tags" in used:
        names.append("tags")
    names.sort(key=lambda entry: (entry[:1].islower(), entry))
    lines = [
        "use rspigot_blocks::BlockId;",
        "",
        "use crate::{%s};" % ", ".join(names),
        "",
        "pub struct %s;" % item["struct"],
        "",
        "impl Item for %s {" % item["struct"],
    ]
    lines.extend(body)
    lines.append("}")
    for line in lines:
        if len(line) > WIDTH:
            fail("%s line too long: %s" % (name, line))
    return "\n".join(lines) + "\n"


def tags_file(dump):
    lines = [
        "use rspigot_blocks::BlockId;",
        "",
        "use crate::{BlockTag, ItemId, ItemTag};",
    ]
    for tag in dump["block_tags"]:
        lines.append("")
        lines.append("pub(crate) static %s: BlockTag = BlockTag {" % tag_constant("BLOCK", tag["name"]))
        lines.append("    name: %s," % rust_string(tag["name"]))
        lines.append("    blocks: &[")
        lines.extend(wrap(["BlockId(%d)" % block for block in tag["blocks"]], "        "))
        lines.append("    ],")
        lines.append("};")
    for tag in dump["item_tags"]:
        lines.append("")
        lines.append("pub(crate) static %s: ItemTag = ItemTag {" % tag_constant("ITEM", tag["name"]))
        lines.append("    name: %s," % rust_string(tag["name"]))
        lines.append("    items: &[")
        lines.extend(wrap(["ItemId(%d)" % entry for entry in tag["items"]], "        "))
        lines.append("    ],")
        lines.append("};")
    for line in lines:
        if len(line) > WIDTH:
            fail("tags line too long: " + line)
    return "\n".join(lines) + "\n"


def kind_file(items, block_count, default_stack):
    lines = ["pub const DEFAULT_MAX_STACK_SIZE: u8 = %d;" % default_stack, ""]
    lines.append("crate::lookup::item_kinds! {")
    lines.append("    count: %d," % len(items))
    lines.append("    kinds: [")
    lines.extend("        %s," % item["struct"] for item in items)
    lines.append("    ],")
    lines.append("}")
    lines.append("")
    by_path = sorted(items, key=lambda item: item["path"])
    lines.append("pub(crate) static NAMES: [(&str, ItemKind); %d] = [" % len(items))
    lines.extend('    ("%s", ItemKind::%s),' % (item["path"], item["struct"]) for item in by_path)
    lines.append("];")
    lines.append("")
    table = [None] * block_count
    for item in items:
        if item["block"] is not None:
            table[item["block"]] = item["struct"]
    lines.append("pub(crate) static BLOCKS: [Option<ItemKind>; %d] = [" % block_count)
    lines.extend("    None," if entry is None else "    Some(ItemKind::%s)," % entry for entry in table)
    lines.append("];")
    return "\n".join(lines) + "\n"


def mod_file(items):
    ordered = sorted(items, key=lambda item: item["path"])
    lines = ["mod %s;" % item["path"] for item in ordered]
    lines.append("")
    lines.extend("pub use %s::%s;" % (item["path"], item["struct"]) for item in ordered)
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
    parser.add_argument("--components", default=DEFAULT_COMPONENTS)
    parser.add_argument("--families", default=DEFAULT_FAMILIES)
    parser.add_argument("--out", default=DEFAULT_OUT)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--stats", action="store_true")
    args = parser.parse_args()

    dump = load_json(args.dump)
    registries = load_json(args.registries)
    block_names = validate(dump, registries, args.components)
    items = dump["items"]
    by_class, by_item = read_families(args.families, items)

    items_dir = os.path.join(args.out, "items")
    if not args.check:
        os.makedirs(items_dir, exist_ok=True)
    stats = {"files": 0, "bytes": 0, "written": 0, "stale": []}
    expected_files = {"mod.rs"}
    families = {}
    for item in items:
        behavior = by_item.get(item["name"]) or by_class.get(item["class"]) or DEFAULT_FAMILY
        if (behavior == BLOCK_FAMILY) != (item["block"] is not None):
            fail("%s of class %s maps to family %s" % (item["name"], item["class"], behavior))
        families[behavior] = families.get(behavior, 0) + 1
        file_name = item["path"] + ".rs"
        expected_files.add(file_name)
        write(os.path.join(items_dir, file_name), item_file(item, behavior), stats, args.check)
    write(os.path.join(items_dir, "mod.rs"), mod_file(items), stats, args.check)
    kind = kind_file(items, len(block_names), dump["default_max_stack_size"])
    write(os.path.join(args.out, "kind.rs"), kind, stats, args.check)
    write(os.path.join(args.out, "tags.rs"), tags_file(dump), stats, args.check)
    removed = 0
    if os.path.isdir(items_dir):
        for entry in sorted(os.listdir(items_dir)):
            if entry.endswith(".rs") and entry not in expected_files:
                removed += 1
                if args.check:
                    stats["stale"].append(os.path.join(items_dir, entry))
                else:
                    os.remove(os.path.join(items_dir, entry))
    print(
        "items %d classes %d block_items %d tools %d foods %d equippable %d block_tags %d item_tags %d families %d files %d bytes %d written %d removed %d"
        % (
            len(items),
            len({item["class"] for item in items}),
            sum(1 for item in items if item["block"] is not None),
            sum(1 for item in items if item["tool"] is not None),
            sum(1 for item in items if item["food"] is not None),
            sum(1 for item in items if item["equippable"] is not None),
            len(dump["block_tags"]),
            len(dump["item_tags"]),
            len(by_class) + len(by_item),
            stats["files"],
            stats["bytes"],
            stats["written"],
            removed,
        )
    )
    if args.stats:
        per_class = {}
        for item in items:
            per_class[item["class"]] = per_class.get(item["class"], 0) + 1
        print("per class top 30 " + " ".join("%s=%d" % pair for pair in top(per_class, 30)))
        print("per family " + " ".join("%s=%d" % pair for pair in top(families, len(families))))
    if args.check and stats["stale"]:
        for path in stats["stale"]:
            print("stale " + path, file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
