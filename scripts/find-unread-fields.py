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


def snake(struct: str) -> str:
    return re.sub(r'(?<!^)(?=[A-Z])', '_', struct).lower()


def claimed_by(struct: str, field: str, source: str) -> bool:
    """Whether some read of `.field` plausibly goes through `struct`.

    A heuristic, and the honest reason it is one: telling `Rule.visible` from
    `Guideline.visible` needs types, and this script does not have them. What
    it has is that Rust code names things after their types -- `rule.visible`,
    `for guideline in`, `current_rules` -- so it looks for a read qualified by
    some fragment of the struct's own name.

    Being wrong in the permissive direction is the safer failure: a field
    wrongly called claimed stays in the last section, which says it still
    wants a look. A field wrongly called unclaimed would send someone hunting
    for a bug that is not there.
    """
    words = [word for word in snake(struct).split('_') if len(word) > 2]
    if not words:
        return True
    # `rule.visible`, `own_rule.visible`, `self.rule.visible`, `r.visible`
    for word in words:
        if re.search(r'\b\w*' + word + r'\w*\s*(?:\.\s*)?\.' + field + r'\b', source):
            return True
        # `for rule in ...` / `|rule|` followed anywhere by `.field` is weaker,
        # so require the binding and the access to share a line.
        if re.search(r'\b' + word + r's?\b[^\n]*\.' + field + r'\b', source):
            return True
    return False


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

    unread, unclaimed, shared = [], [], []
    for relative, struct, field in declared:
        hits = len(re.findall(r'\.' + field + r'\b', everything))
        if hits == 0:
            unread.append((relative, struct, field))
        elif len(owners[field]) > 1:
            if claimed_by(struct, field, everything):
                shared.append((field, hits, owners[field]))
            else:
                unclaimed.append((relative, struct, field, hits))

    print(f'=== no reader at all ({len(unread)}) ===')
    for relative, struct, field in unread:
        print(f'   {relative:<30} {struct:<26} {field}')

    print(f'\n=== shared name, read through some other struct ({len(unclaimed)}) ===')
    for relative, struct, field, hits in unclaimed:
        print(f'   {relative:<30} {struct:<26} {field:<22} ({hits} hit(s) elsewhere)')

    seen = set()
    unique_shared = [
        entry for entry in sorted(shared, key=lambda e: e[0])
        if not (entry[0] in seen or seen.add(entry[0]))
    ]
    print(f'\n=== shared name, at least one read looks right ({len(unique_shared)}) ===')
    for field, hits, structs in unique_shared:
        print(f'   {field:<26} {hits} hit(s) on: {", ".join(sorted(structs))}')

    return 0


if __name__ == '__main__':
    sys.exit(main())
