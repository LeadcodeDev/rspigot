import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CATALOG = HERE / "catalog.json"
JAR_NAME = "spigot-api-26.3-R0.1-SNAPSHOT.jar"
DEFAULT_JAVAP = "/opt/homebrew/opt/openjdk@25/bin/javap"

MODIFIERS = {
    "public", "protected", "private", "abstract", "final", "static", "default",
    "synchronized", "native", "strictfp", "transient", "volatile", "sealed",
}
PRIMITIVES = {"boolean", "byte", "short", "char", "int", "long", "float", "double"}
GETTER = re.compile(r"^(get|is|has|can|should)[A-Z]")
MUTATOR = re.compile(r"^(set|add|remove|clear|put)[A-Z]")
EXCLUDED = {
    "getHandlers", "getHandlerList", "getEventName", "isAsynchronous",
    "isCancelled", "setCancelled",
}
CONTAINERS = {"java.util.List", "java.util.Set", "java.util.Collection", "java.util.Map"}
MAX_LISTED_CONSTANTS = 200
EVENT = "org.bukkit.event.Event"
FIRST = [
    "org.bukkit.event.player.PlayerJoinEvent",
    "org.bukkit.event.player.PlayerQuitEvent",
    "org.bukkit.event.player.PlayerMoveEvent",
    "org.bukkit.event.inventory.InventoryClickEvent",
    "org.bukkit.event.inventory.InventoryCloseEvent",
]


def split_top(text):
    parts, depth, current = [], 0, ""
    for char in text:
        if char == "<":
            depth += 1
        elif char == ">":
            depth -= 1
        if char == "," and depth == 0:
            parts.append(current.strip())
            current = ""
        else:
            current += char
    if current.strip():
        parts.append(current.strip())
    return parts


def balanced_end(text):
    depth = 0
    for index, char in enumerate(text):
        if char == "<":
            depth += 1
        elif char == ">":
            depth -= 1
            if depth == 0:
                return index + 1
    raise ValueError(text)


def strip_modifiers(text):
    found = []
    while True:
        head, _, rest = text.partition(" ")
        if head in MODIFIERS:
            found.append(head)
            text = rest
        else:
            return found, text


def parse_member(line):
    text = line.strip().removesuffix(";")
    if "(" not in text:
        name = re.search(r"([\w$]+)$", text)
        if name is None:
            return None
        modifiers, rest = strip_modifiers(text[: name.start()].strip())
        return {"field": True, "name": name[1], "type": rest, "mods": modifiers}
    opening = text.index("(")
    name = re.search(r"([\w$.]+)$", text[:opening])
    if "." in name[1]:
        return None
    modifiers, rest = strip_modifiers(text[: name.start()].strip())
    if rest.startswith("<"):
        rest = rest[balanced_end(rest):].strip()
    if not rest:
        return None
    return {
        "field": False,
        "name": name[1],
        "ret": rest,
        "params": split_top(text[opening + 1 : text.rindex(")")]),
        "mods": modifiers,
    }


def parse_dump(text):
    classes = {}
    current = None
    for line in text.splitlines():
        if not line.strip() or line.startswith("Compiled from"):
            continue
        if not line.startswith(" "):
            if line == "}":
                current = None
                continue
            header = re.match(r"^((?:\w+ )*?)(class|interface) ([\w.$]+)(.*) \{$", line)
            rest = header[4]
            if rest.startswith("<"):
                rest = rest[balanced_end(rest):]
            implements = re.search(r" implements (.*)$", rest)
            if implements:
                rest = rest[: implements.start()]
            extends = re.search(r" extends (.*)$", rest)
            current = {
                "kind": header[2],
                "abstract": "abstract" in header[1].split() or header[2] == "interface",
                "extends": split_top(extends[1]) if extends else [],
                "implements": split_top(implements[1]) if implements else [],
                "methods": [],
                "fields": [],
            }
            classes[header[3]] = current
            continue
        member = parse_member(line)
        if member is None:
            continue
        current["fields" if member["field"] else "methods"].append(member)
    return classes


def base_name(type_name):
    return type_name.split("<")[0]


def superclasses(classes, name):
    chain, current = [], name
    while current in classes and current not in chain:
        chain.append(current)
        entry = classes[current]
        if entry["kind"] != "class" or not entry["extends"]:
            break
        current = base_name(entry["extends"][0])
    return chain


def all_supertypes(classes, name):
    seen, queue = set(), [name]
    while queue:
        current = queue.pop()
        if current in seen or current not in classes:
            continue
        seen.add(current)
        queue.extend(base_name(parent) for parent in classes[current]["extends"] + classes[current]["implements"])
    return seen


def listenable(classes):
    return sorted(
        name for name, entry in classes.items()
        if name.startswith("org.bukkit.event.")
        and name.endswith("Event")
        and "$" not in name
        and entry["kind"] == "class"
        and not entry["abstract"]
        and EVENT in superclasses(classes, name)[1:]
    )


def declares_handler_list(classes, name):
    return any(
        method["name"] == "getHandlerList" and "static" in method["mods"] and not method["params"]
        for method in classes[name]["methods"]
    )


def annotations(javap, jar, names):
    found = {}
    ordered = sorted(names)
    for start in range(0, len(ordered), 60):
        out = subprocess.run(
            [javap, "-v", "-cp", str(jar), *ordered[start : start + 60]],
            capture_output=True, text=True, check=True,
        ).stdout
        for part in out.split("Classfile ")[1:]:
            owner = re.search(r"!/(org/bukkit/[^\s]+)\.class", part.splitlines()[0])[1].replace("/", ".")
            read_class(part, owner, found)
    return found


def read_class(part, owner, found):
    member = None
    section = None
    parameter = None
    for line in part.splitlines():
        if re.fullmatch(r"  \S.*;", line) and line.split()[0] in MODIFIERS:
            parsed = parse_member(line)
            member = None
            if parsed and not parsed["field"]:
                member = {"deprecated": False, "returns": None, "params": [None] * len(parsed["params"])}
                found[(owner, parsed["name"], tuple(parsed["params"]))] = member
            section = None
            continue
        if member is None:
            continue
        if re.fullmatch(r"    \S.*:.*", line):
            section = line.strip().split(":")[0]
            parameter = None
            if line == "    Deprecated: true":
                member["deprecated"] = True
            continue
        if section == "RuntimeInvisibleParameterAnnotations":
            match = re.fullmatch(r"      parameter (\d+):", line)
            if match:
                parameter = int(match[1])
                continue
        nullity = re.fullmatch(r"\s+org\.jetbrains\.annotations\.(Nullable|NotNull)", line)
        if nullity is None:
            continue
        value = "nullable" if nullity[1] == "Nullable" else "not_null"
        if section == "RuntimeInvisibleAnnotations":
            member["returns"] = value
        elif section == "RuntimeInvisibleParameterAnnotations" and parameter is not None:
            if parameter < len(member["params"]):
                member["params"][parameter] = value


def collect(classes, notes, event):
    seen = {}
    declared = {}
    for owner in superclasses(classes, event):
        if owner == EVENT:
            continue
        for method in classes[owner]["methods"]:
            if "static" in method["mods"] or "public" not in method["mods"]:
                continue
            key = (method["name"], tuple(method["params"]))
            if key not in seen:
                seen[key] = method
                declared[key] = owner
    result = {"getters": [], "setters": [], "indexed": [], "other": []}
    for key in sorted(seen):
        name, params = key
        if name in EXCLUDED:
            continue
        method = seen[key]
        note = notes.get((declared[key], name, params), {"deprecated": False, "returns": None, "params": [None] * len(params)})
        entry = {"name": name, "declared_in": declared[key]}
        if note["deprecated"]:
            entry["deprecated"] = True
        if GETTER.match(name) and not params:
            result["getters"].append({**entry, "type": method["ret"], "nullable": note["returns"]})
        elif GETTER.match(name):
            result["indexed"].append({**entry, "params": list(params), "type": method["ret"], "nullable": note["returns"]})
        elif MUTATOR.match(name):
            result["setters"].append({**entry, "params": list(params), "nullable": note["params"]})
        else:
            result["other"].append({**entry, "params": list(params), "type": method["ret"], "nullable": note["returns"]})
    assign_keys(result)
    return result


def assign_keys(found):
    stems = {}
    for entry in found["getters"]:
        stems.setdefault(stem(entry["name"]), []).append(entry)
    for name, entries in stems.items():
        for entry in entries:
            plain = len(entries) == 1 or entry["name"].startswith("get")
            entry["key"] = name if plain else entry["name"]
    for entry in found["setters"]:
        entry["key"] = stem(entry["name"])
    for entry in found["indexed"]:
        entry["key"] = entry["name"] + "[]"
    for entry in found["other"]:
        entry["key"] = entry["name"]


def leaves(type_name):
    type_name = type_name.strip()
    if type_name.endswith("[]"):
        return leaves(type_name[:-2])
    generic = re.match(r"^([\w.$]+)<(.*)>$", type_name)
    if generic and generic[1] in CONTAINERS:
        found = []
        for argument in split_top(generic[2]):
            argument = re.sub(r"^\?( (extends|super) )?", "", argument).strip()
            if argument:
                found.extend(leaves(argument))
        return found
    if generic:
        return [generic[1]]
    return [type_name]


def constants_of(classes, name):
    return [field["name"] for field in classes[name]["fields"] if field["type"] == name and "static" in field["mods"]]


def classify(classes, name):
    if name in PRIMITIVES:
        return "primitive"
    if name not in classes:
        return "external"
    entry = classes[name]
    if any(base_name(parent) == "java.lang.Enum" for parent in entry["extends"]):
        return "enum"
    if "org.bukkit.entity.Entity" in all_supertypes(classes, name):
        return "entity"
    if constants_of(classes, name):
        return "keyed"
    return "bukkit"


def type_record(classes, name):
    kind = classify(classes, name)
    record = {"kind": kind}
    if kind == "enum":
        constants = constants_of(classes, name)
        record["constants"] = constants if len(constants) <= MAX_LISTED_CONSTANTS else None
    return record


def stem(name):
    for prefix in ("get", "is", "has", "can", "should", "set", "add", "remove", "clear", "put"):
        if name.startswith(prefix) and len(name) > len(prefix) and name[len(prefix)].isupper():
            return name[len(prefix)].lower() + name[len(prefix) + 1 :]
    return name


def simple(name):
    return name.rsplit(".", 1)[1]


def number_events(names, deprecated, previous):
    known = {event["java"]: event["number"] for event in previous.get("events", []) if event.get("number")}
    numbers = {}
    for name in FIRST:
        if name in names and name not in known:
            known[name] = FIRST.index(name) + 1
    for name, number in known.items():
        if name in names and name not in deprecated:
            numbers[name] = number
    taken = set(known.values())
    following = max(taken, default=0) + 1
    for name in sorted(names, key=simple):
        if name in deprecated or name in numbers:
            continue
        while following in taken:
            following += 1
        numbers[name] = following
        taken.add(following)
    return numbers


def number_fields(found, previous):
    known = dict(previous or {})
    stems = sorted({entry["key"] for key in ("getters", "setters", "indexed", "other") for entry in found[key]})
    following = max(known.values(), default=0) + 1
    for name in stems:
        if name not in known:
            known[name] = following
            following += 1
    return dict(sorted(known.items(), key=lambda item: item[1]))


def render(value, depth=0):
    scalar = not isinstance(value, (dict, list)) or not value
    if scalar or depth >= 4 or (isinstance(value, list) and not any(isinstance(item, (dict, list)) for item in value)):
        return json.dumps(value, ensure_ascii=False)
    pad = " " * (depth + 1)
    if isinstance(value, list):
        inner = [pad + render(item, depth + 1) for item in value]
        return "[\n" + ",\n".join(inner) + "\n" + " " * depth + "]"
    inner = [f"{pad}{json.dumps(key, ensure_ascii=False)}: {render(value[key], depth + 1)}" for key in value]
    return "{\n" + ",\n".join(inner) + "\n" + " " * depth + "}"


def build(dump, jar, javap, previous):
    classes = parse_dump((dump / "spigot-api.javap.txt").read_text())
    names = listenable(classes)
    flagged = {row["event"] for row in json.loads((dump / "events.json").read_text()) if row["deprecated"]}
    unknown = sorted(name for name in flagged if name not in names)
    if unknown:
        raise SystemExit(f"events.json lists events the javap dump lacks: {unknown}")
    chain = {owner for name in names for owner in superclasses(classes, name)} - {EVENT}
    notes = annotations(javap, jar, chain)
    numbers = number_events(names, flagged, previous)
    old_fields = {event["java"]: event.get("fields") for event in previous.get("events", [])}
    events = []
    used = set()
    for name in names:
        found = collect(classes, notes, name)
        chain_of = superclasses(classes, name)
        registration = next((owner for owner in chain_of if declares_handler_list(classes, owner)), None)
        record = {
            "java": name,
            "name": simple(name).removesuffix("Event"),
            "package": name.split(".")[3],
            "cancellable": "org.bukkit.event.Cancellable" in all_supertypes(classes, name),
            "deprecated": name in flagged,
            "parents": [owner for owner in chain_of[1:] if owner != EVENT],
            "registration": registration,
        }
        if name not in flagged:
            record["number"] = numbers[name]
            record["fields"] = number_fields(found, old_fields.get(name))
            for key in ("getters", "setters", "indexed", "other"):
                for entry in found[key]:
                    for type_name in entry.get("params", []) + ([entry["type"]] if "type" in entry else []):
                        used.update(leaves(type_name))
        record.update(found)
        events.append(record)
    types = {leaf: type_record(classes, leaf) for leaf in sorted(used)}
    return {"source": JAR_NAME, "events": events, "types": types}


def main():
    parser = argparse.ArgumentParser(description="derive the plugin event catalog from the Spigot API dump")
    parser.add_argument("dump", type=Path, help="directory holding spigot-api.javap.txt and events.json")
    parser.add_argument("--jar", type=Path, help=f"the API jar, read with javap -v (default: <dump>/../libs/{JAR_NAME})")
    parser.add_argument("--javap", default=DEFAULT_JAVAP)
    parser.add_argument("--check", action="store_true", help="fail when the committed catalog differs")
    args = parser.parse_args()
    jar = args.jar or args.dump.parent / "libs" / JAR_NAME
    previous = json.loads(CATALOG.read_text()) if CATALOG.exists() else {}
    text = render(build(args.dump, jar, args.javap, previous)) + "\n"
    if args.check:
        if not CATALOG.exists() or CATALOG.read_text() != text:
            print(f"{CATALOG} is stale", file=sys.stderr)
            return 1
        return 0
    CATALOG.write_text(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
