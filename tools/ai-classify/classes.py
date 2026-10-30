import json
import os
import re
import sys

LAND_FALSE = {
    "CopperGolemStatueBlock": "0295f5669125028d",
    "ShelfBlock": "0295f5669125028d",
    "SlabBlock": "34c5a862109ccf3f",
}

HANDLED = {
    "BlockBehaviour": "b06e3ebde939e681",
    "VegetationBlock": "9bb37b42b6fdd119",
    "DoorBlock": "ed301fce5ba23d01",
    "FenceGateBlock": "f28d7dd78ab987eb",
    "TrapDoorBlock": "eee138cad956495f",
    "SnowLayerBlock": "9b0c2f6bbbb17fdb",
    "LiquidBlock": "49b412e004160ccc",
    "PowderSnowBlock": "true",
}

GROUPS = [
    "Trapdoor",
    "PowderSnow",
    "Damaging",
    "StickyHoney",
    "Cocoa",
    "DamageCautious",
    "Burning",
    "Campfire",
    "WoodenDoor",
    "IronDoor",
    "Rail",
    "Leaves",
    "Fence",
    "FenceGate",
    "SnowLayer",
    "Liquid",
    "Unpathable",
]

EXPECTED_CAULDRONS = ["cauldron", "water_cauldron", "lava_cauldron", "powder_snow_cauldron"]


def fail(message):
    sys.exit("ai-classify classes: " + message)


def load_json(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


class Tags:
    def __init__(self, root):
        self.root = root

    def members(self, name):
        path = os.path.join(self.root, name + ".json")
        if not os.path.exists(path):
            fail("missing tag " + path)
        out = []
        for value in load_json(path)["values"]:
            if isinstance(value, dict):
                fail("unsupported tag entry in " + path)
            if value.startswith("#"):
                out += self.members(value[1:].split(":", 1)[1])
            else:
                out.append(value.split(":", 1)[1])
        return out


def read_owners(path):
    owners = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            simple, chain, owner, fingerprint = line.rstrip("\n").split("|")
            owners[simple] = (chain.split(">"), owner, fingerprint)
    return owners


def read_structs(path):
    with open(path, encoding="utf-8") as handle:
        return dict(re.findall(r"pub use (\w+)::(\w+);", handle.read()))


def rule(owner, fingerprint):
    if fingerprint == "false":
        return "never"
    if owner in LAND_FALSE:
        if LAND_FALSE[owner] != fingerprint:
            fail("isPathfindable changed in " + owner)
        return "never"
    if owner in HANDLED:
        if HANDLED[owner] != fingerprint:
            fail("isPathfindable changed in " + owner)
        return owner
    fail("unhandled isPathfindable owner " + owner + " " + fingerprint)


def group_of(name, chain, land, tags):
    if name in tags["trapdoor"]:
        return "Trapdoor"
    if name == "powder_snow":
        return "PowderSnow"
    if name in ("cactus", "sweet_berry_bush"):
        return "Damaging"
    if name == "honey_block":
        return "StickyHoney"
    if name == "cocoa":
        return "Cocoa"
    if name == "wither_rose" or name in tags["speleothems"]:
        return "DamageCautious"
    if name in tags["fire"] or name in ("magma_block", "lava_cauldron"):
        return "Burning"
    if name in tags["campfires"]:
        return "Campfire"
    if "DoorBlock" in chain:
        return "IronDoor" if name == "iron_door" else "WoodenDoor"
    if "BaseRailBlock" in chain:
        return "Rail"
    if "LeavesBlock" in chain:
        return "Leaves"
    if name in tags["fence"]:
        return "Fence"
    if "FenceGateBlock" in chain:
        return "FenceGate"
    if land == "SnowLayerBlock":
        return "SnowLayer"
    if land == "LiquidBlock":
        return "Liquid"
    if land == "never":
        return "Unpathable"
    if land in ("TrapDoorBlock", "DoorBlock", "FenceGateBlock", "PowderSnowBlock"):
        fail("block " + name + " escaped its group")
    return None


def main():
    if len(sys.argv) != 6:
        fail("usage: classes.py BLOCK_PROPERTIES HIER TAGS_DIR BLOCKS_MOD OUT")
    dump, hier, tags_dir, blocks_mod, out = sys.argv[1:6]
    data = load_json(dump)
    if data.get("version") != "26.3":
        fail("unexpected dump version " + repr(data.get("version")))
    owners = read_owners(hier)
    structs = read_structs(blocks_mod)
    tags = Tags(tags_dir)
    sets = {
        "trapdoor": set(tags.members("trapdoors")) | {"lily_pad", "big_dripleaf"},
        "speleothems": set(tags.members("speleothems")),
        "fire": set(tags.members("fire")),
        "campfires": set(tags.members("campfires")),
        "fence": set(tags.members("fences")) | set(tags.members("walls")),
    }
    if tags.members("cauldrons") != EXPECTED_CAULDRONS:
        fail("cauldrons tag changed")
    groups = {group: [] for group in GROUPS}
    for block in data["blocks"]:
        name = block["name"].split(":", 1)[1]
        if block["class"] not in owners:
            fail("no hierarchy for " + block["class"])
        chain, owner, fingerprint = owners[block["class"]]
        group = group_of(name, chain, rule(owner, fingerprint), sets)
        if group is None:
            continue
        if name not in structs:
            fail("no block struct for " + name)
        groups[group].append(structs[name])
    lines = ["pub(crate) const fn class_of(kind: BlockKind) -> BlockClass {", "    match kind {"]
    for group in GROUPS:
        if not groups[group]:
            fail("empty group " + group)
        arms = " | ".join("BlockKind::" + member for member in groups[group])
        lines.append("        " + arms + " => BlockClass::" + group + ",")
    lines += ["        _ => BlockClass::Plain,", "    }", "}", ""]
    with open(out, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines))


main()
