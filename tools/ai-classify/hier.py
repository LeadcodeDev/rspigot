import hashlib
import json
import os
import re
import subprocess
import sys

PACKAGES = ("net.minecraft.world.level.block", "net.minecraft.world.level.block.piston")
METHOD = "isPathfindable(net.minecraft.world.level.block.state.BlockState, net.minecraft.world.level.pathfinder.PathComputationType);"
INSTRUCTION = re.compile(r"^\s*\d+: (\w+)\s*(.*)$")
CONSTANT = re.compile(r"#\d+(,\s*\d+)?")


def fail(message):
    sys.exit("ai-classify hier: " + message)


class Classes:
    def __init__(self, root, javap):
        self.root = root
        self.javap = javap
        self.cache = {}

    def load(self, name):
        if name in self.cache:
            return self.cache[name]
        path = os.path.join(self.root, name.replace(".", "/") + ".class")
        if not os.path.exists(path):
            self.cache[name] = None
            return None
        result = subprocess.run([self.javap, "-c", "-p", path], capture_output=True, text=True)
        if result.returncode != 0:
            fail("javap failed on " + path + ": " + result.stderr.strip())
        lines = result.stdout.splitlines()
        parent = None
        for line in lines:
            if " class " in line and " extends " in line:
                parent = line.split(" extends ")[1].split()[0].split("<")[0]
                break
        self.cache[name] = (parent, body(lines))
        return self.cache[name]

    def find(self, simple):
        for package in PACKAGES:
            name = package + "." + simple
            if self.load(name) is not None:
                return name
        fail("class not found for " + simple)


def body(lines):
    for index, line in enumerate(lines):
        if line.strip().endswith(METHOD):
            code = []
            for entry in lines[index + 1:]:
                if not entry.strip():
                    break
                match = INSTRUCTION.match(entry)
                if not match:
                    continue
                opcode, operand = match.groups()
                if "//" in operand:
                    operand = operand.split("//", 1)[1].strip()
                code.append((opcode + " " + CONSTANT.sub("", operand)).strip())
            return code
    return None


def fingerprint(code):
    if code == ["iconst_0", "ireturn"]:
        return "false"
    if code == ["iconst_1", "ireturn"]:
        return "true"
    return hashlib.sha256("\n".join(code).encode("utf-8")).hexdigest()[:16]


def main():
    if len(sys.argv) != 5:
        fail("usage: hier.py CLASSES_ROOT BLOCK_PROPERTIES JAVAP OUT")
    root, dump, javap, out = sys.argv[1:5]
    with open(dump, encoding="utf-8") as handle:
        blocks = json.load(handle)["blocks"]
    classes = Classes(root, javap)
    rows = []
    seen = set()
    for block in blocks:
        simple = block["class"]
        if simple in seen:
            continue
        seen.add(simple)
        chain = []
        owner = None
        current = classes.find(simple)
        while current and current.startswith("net.minecraft."):
            loaded = classes.load(current)
            if loaded is None:
                break
            parent, code = loaded
            short = current.split(".")[-1]
            chain.append(short)
            if owner is None and code is not None:
                owner = (short, fingerprint(code))
            current = parent
        if owner is None:
            fail("no isPathfindable owner for " + simple)
        rows.append("|".join([simple, ">".join(chain), owner[0], owner[1]]))
    with open(out, "w", encoding="utf-8") as handle:
        handle.write("\n".join(rows) + "\n")


main()
