#!/usr/bin/env python3
"""Report `pub` fields in data/ and state/ that no Rust code reads.

Most of the disconnects found in this project were the same shape: a field
authored in JSON, deserialized into a struct, and read by nothing. The game
looked complete because the data was there. This finds those.

    python scripts/find-unread-fields.py

Two sections come out:

  * "no reader at all" -- nothing anywhere accesses `.field`. Each of these is
    either a system that was never wired or a leftover from the JavaScript
    version this was ported from. Deciding which is the point of the report.

  * "shared name" -- the field name is declared on more than one struct, so a
    hit on `.field` may belong to the other one. These need a by-hand check.
    This section exists because its absence hid SCORING.SURVIVAL_BONUS behind
    GAME_CONSTANTS.SURVIVAL_BONUS: same name, same value, only one of them
    read, and the naive sweep called both live for a week.

Deliberately crude. It matches text rather than parsing Rust, so it cannot see
a field read through a destructuring pattern or a `..` struct update, and it
will not notice a field that is written but never meaningfully consumed. Treat
a hit as a question, not a verdict -- several entries here are construction
inputs kept on a record on purpose.
"""
import collections
import pathlib
import re
import sys

FIELD = re.compile(r'^\s*pub (\w+):', re.M)
STRUCT = re.compile(r'pub struct (\w+)\s*\{(.*?)\n\}', re.S)
# Where authored data lands. Engine and UI structs are excluded: a field
# nothing reads there is ordinary dead code and clippy already objects.
SEARCHED = ('src/data', 'src/state')


def main() -> int:
    root = pathlib.Path(__file__).resolve().parent.parent
    files = {
        path: path.read_text(encoding='utf-8')
        for path in (root / 'src').rglob('*.rs')
    }
    if not files:
        print('no sources found under', root / 'src')
        return 2
    everything = '\n'.join(files.values())

    declared = []
    for path, text in files.items():
        relative = path.relative_to(root).as_posix()
        if not relative.startswith(SEARCHED):
            continue
        for match in STRUCT.finditer(text):
            struct, body = match.group(1), match.group(2)
            for field in FIELD.findall(body):
                declared.append((relative, struct, field))

    owners = collections.defaultdict(set)
    for _, struct, field in declared:
        owners[field].add(struct)

    unread, shared = [], []
    for relative, struct, field in declared:
        hits = len(re.findall(r'\.' + field + r'\b', everything))
        if hits == 0:
            unread.append((relative, struct, field))
        elif len(owners[field]) > 1:
            shared.append((field, hits, owners[field]))

    print(f'=== no reader at all ({len(unread)}) ===')
    for relative, struct, field in unread:
        print(f'   {relative:<30} {struct:<26} {field}')

    seen = set()
    unique_shared = [
        entry for entry in sorted(shared, key=lambda e: e[0])
        if not (entry[0] in seen or seen.add(entry[0]))
    ]
    print(f'\n=== shared name, check by hand ({len(unique_shared)}) ===')
    for field, hits, structs in unique_shared:
        print(f'   {field:<26} {hits} hit(s) on: {", ".join(sorted(structs))}')

    return 0


if __name__ == '__main__':
    sys.exit(main())
