import json
import sys

DEFAULT_REPORT = "/Users/baptiste.parmantier/Development/rspigot/re/datagen/generated/reports/packets.json"
STATES = ["handshake", "status", "login", "configuration", "play"]


def const_name(key):
    return key.split(":", 1)[1].replace("/", "_").upper()


def main():
    report = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_REPORT
    output = sys.argv[2] if len(sys.argv) > 2 else "crates/rspigot-protocol/src/ids.rs"
    with open(report) as f:
        packets = json.load(f)
    lines = []
    lookup = []
    for state in STATES:
        lines.append(f"pub mod {state} {{")
        for direction in ["clientbound", "serverbound"]:
            entries = sorted(packets.get(state, {}).get(direction, {}).items(), key=lambda kv: kv[1]["protocol_id"])
            if not entries:
                continue
            lines.append(f"    pub mod {direction} {{")
            for key, value in entries:
                lines.append(f"        pub const {const_name(key)}: i32 = {value['protocol_id']};")
            lines.append("    }")
            names = ", ".join(f'"{k}"' for k, _ in entries)
            lookup.append((state, direction, names))
        lines.append("}")
        lines.append("")
    lines.append("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]")
    lines.append("pub enum State {")
    for state in STATES:
        lines.append(f"    {state.capitalize()},")
    lines.append("}")
    lines.append("")
    lines.append("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]")
    lines.append("pub enum Direction {")
    lines.append("    Clientbound,")
    lines.append("    Serverbound,")
    lines.append("}")
    lines.append("")
    lines.append("pub fn packet_name(state: State, direction: Direction, id: i32) -> Option<&'static str> {")
    lines.append("    let names: &[&str] = match (state, direction) {")
    for state, direction, names in lookup:
        lines.append(f"        (State::{state.capitalize()}, Direction::{direction.capitalize()}) => &[{names}],")
    lines.append("        _ => &[],")
    lines.append("    };")
    lines.append("    usize::try_from(id).ok().and_then(|i| names.get(i).copied())")
    lines.append("}")
    with open(output, "w") as f:
        f.write("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
