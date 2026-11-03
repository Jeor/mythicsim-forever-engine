#!/usr/bin/env python3
"""Check the numbers Forever's rules take from Blizzard's development notes of 2 November 2026
against a client spell store, so each change is confirmed once the live data carries it.

    python3 tools/dev_notes_check.py STORE [STORE ...]

STORE is a `spell_store_inputs.json` (gzip or plain), the file wowsims/forever regenerates from
the live client and its hotfix cache: `assets/db_inputs/spell_store_inputs.json` at any of its
commits, most usefully each daily "[DB] Update items to ... with hotfixes as of ..." commit.

Each check prints one of:

- `live`: the store states the value Forever's rules use.
- `old`: the store still states the frozen value, so the engine runs ahead of the live data.
- `DIFFERS`: the store states a third value. The live game disagrees with the engine, which needs
  the store's number (`prepare::hotfix_113554` or the reader named in the check).

A change no client row can state (an attack power share, a proc rate, a form's threat) is listed
as `not in the data`, with what it takes to check it in game. The exit status is 1 when any check
reads DIFFERS.

Uses only Python's standard library.
"""

import argparse
import gzip
import json
import sys
from pathlib import Path

# (label, spell id, effect id or None for a cooldown, field, Forever's value, frozen value)
ROWS = [
    ("Aimed Shot rank 6", 20904, 697764, "BasePoints", 350, 166),
    ("Arcane Shot rank 8", 14287, 691069, "BasePoints", 280, 217),
    ("Serpent Sting rank 8", 13555, 690655, "BasePoints", 100, 83),
    ("Sniper Shot rank 3", 1310786, 1340619, "BasePoints", 590, 295),
    ("Aspect of the Hawk rank 6", 14322, 692138, "BasePoints", 110, 55),
    ("Trueshot Aura rank 5", 20906, 697683, "BasePoints", 100, 50),
    ("Starfire rank 6 (placeholder: notes say about 40%)", 9876, 688325, "BasePoints",
     337 * 1.4, 337),
    ("Shred weapon damage %", 9830, 688179, "BasePoints", 200, 155),
    ("Claw weapon damage %", 9850, 687219, "BasePoints", 140, 110),
    ("Ravage weapon damage %", 9867, 687812, "BasePoints", 435, 350),
    ("Defensive Stance threat %", 7376, 686475, "BasePoints", 80, 30),
    ("Slam cooldown (ms)", 11605, None, "CategoryCooldownMs", 15000, 18000),
    ("Wrack damage a level", 1316697, 1349556, "PPL", 2.5, 0),
    ("Wrack spell power coefficient", 1316697, 1349556, "SPCoef", 0.286, 0.143),
    ("Stormstrike cooldown (ms)", 17364, None, "CooldownMs", 6000, 8000),
    ("Improved Slam cooldown cut (ms), rank 2", 12862, 1339521, "BasePoints", -5000, -3000),
    ("Lightning Bolt rank 10 base", 15208, 691780, "BasePoints", 355.5, 196),
    ("Lightning Bolt rank 5 base", 943, 679378, "BasePoints", 109, 82),
    ("Lightning Bolt overload rank 10 base", 408477, 1072630, "BasePoints", 178, 98),
    ("Chain Lightning rank 4 base", 10605, 688920, "BasePoints", 519, 123),
    ("Chain Lightning rank 4 spell power coefficient", 10605, 688920, "SPCoef", 0.714, 0.571),
    ("Earth Shock rank 7", 10414, 688559, "BasePoints", 531, 301),
    ("Flame Shock rank 6, direct", 29228, 704945, "BasePoints", 292, 166),
    ("Flame Shock rank 6, a tick", 29228, 704946, "BasePoints", 80, 44),
    ("Frost Shock rank 4", 10473, 688022, "BasePoints", 506, 283),
    ("Flawless Execution Energy", 1310711, 1340441, "BasePoints", -15, -10),
    ("Mutilate rank 4 main hand weapon %", 1241586, 1240654, "BasePoints", 90, 75),
    ("Mutilate rank 4 off hand weapon %", 1241590, 1240661, "BasePoints", 90, 75),
    ("Mutilate rank 1 weapon %", 1310705, 1340423, "BasePoints", 90, 75),
    ("Riptide rank 3 cooldown (ms)", 1239243, None, "CategoryCooldownMs", 12000, 6000),
]

# (label, spell id, power cost field, Forever's value, frozen value): the mana cost a row states.
POWERS = [
    ("Riptide rank 1 mana (about 40% less)", 408521, "Cost", 147, 245),
    ("Riptide rank 2 mana (about 40% less)", 1239242, "Cost", 180, 300),
    ("Riptide rank 3 mana (about 40% less)", 1239243, "Cost", 231, 385),
]

# (label, spell id, Forever's value, frozen value): the stacks an aura row states.
STACKS = [
    ("Vengeance stacks (Paladin, 5 in the notes)", 20050, 5, 3),
]

# (label, spell id, effect id, class flag bit, whether Forever's rules have it set)
CLASS_FLAGS = [
    ("Hemorrhage's debuff covers Garrote", 16511, 693309, 256, True),
    ("Aggression leaves Backstab out", 18427, 695769, 4, False),
]

# (label, talent spell id, curve row, Forever's values, frozen values)
TALENTS = [
    ("Barrage", 19461, 0, [5, 10, 15], [3, 7, 10]),
    ("Two-Handed Weapon Specialization", 12163, 0, [2, 4, 6], [1, 2, 3]),
    ("Defiance", 12792, 0, [7, 13, 20], [5, 10, 15]),
    ("Shadow and Flame damage", 426316, 2, [4, 8, 12, 16, 20], [2, 4, 6, 8, 10]),
    ("Lightning Overload", 408438, 0, [10, 20, 30], [3, 7, 10]),
    ("Mental Dexterity", 415140, 0, [50, 100, 150], [33, 67, 100]),
    ("Flurry (Shaman)", 16256, 0, [6, 12, 18, 24, 30], [5, 10, 15, 20, 25]),
    ("Aggression", 18427, 0, [5, 10, 15], [2, 4, 6]),
    ("Rend and Tear", 1223246, 0, [3, 6, 9, 12, 15], [2, 4, 6, 8, 10]),
]

NOT_IN_DATA = [
    ("Riptide has no initial heal, only the heal over time",
     "the tooltip names a heal and a heal over time; the first effect leaves the row as a heal"),
    ("Poised for Druids (level 32, 10 Energy off the next Energy ability out of combat)",
     "no client row yet: Forever's rules use the provisional id 1399003; replace it once a client "
     "build carries the spell"),
    ("Instant Poison 1% and Deadly Poison 0.9% a tick of attack power",
     "poison hits at two attack power values: each grows 0.01 and 0.009 a point (a stack for Deadly)"),
    ("Efficient Strikes takes Energy off Hemorrhage too",
     "Hemorrhage's cost with the talent: 35 less 3 or 5 (the talent's class flags state it once "
     "the client does)"),
    ("Shank (level 24, 5 Energy, 10 sec, 50% weapon damage, 15 or 25 Energy) and Poised",
     "no client row yet: Forever's rules use the provisional ids 1399001 and 1399002; replace "
     "them once a client build carries the spells"),
    ("Arcane Shot 15% of ranged attack power",
     "Arcane Shot on a training dummy at two ranged attack power values: the damage grows 0.15 a point"),
    ("Serpent Sting 5% of ranged attack power a tick",
     "Serpent Sting ticks at two ranged attack power values: each tick grows 0.05 a point"),
    ("Rip, Rake, Ferocious Bite and Pounce about 25% (placeholder 1.25)",
     "tooltips or ticks at a known attack power, then replace the placeholder with the rows"),
    ("Bear Form threat 215%", "threat meter on a dummy with a known damage hit"),
    ("Maelstrom Weapon chance doubled (20 PPM)",
     "count stacks over a few minutes of melee with a known weapon speed"),
    ("Stormstrike 150% weapon damage", "Stormstrike hits against a dummy with a known weapon"),
    ("Chain Lightning cast time 2.5 sec and mana 248/304/428/495 (225% of Lightning Bolt)",
     "the spell tooltips: the store keeps cast time and power cost outside the effect rows"),
    ("Call of Thunder, +5% crit for Lightning Bolt and Chain Lightning, in Lava Burst's slot",
     "the talent's tooltip once it has a row; Lava Burst (1326047) leaves the store with it"),
]


def load(path):
    raw = Path(path).read_bytes()
    if raw[:2] == b"\x1f\x8b":
        raw = gzip.decompress(raw)
    return json.loads(raw)


def close(a, b):
    return abs(float(a) - float(b)) < 1e-3


def verdict(value, forever, frozen):
    if value is None:
        return "missing"
    if close(value, forever):
        return "live"
    if close(value, frozen):
        return "old"
    return "DIFFERS"


def row_value(store, spell, effect_id, field):
    if effect_id is None:
        return (store["Cooldowns"].get(str(spell)) or {}).get(field)
    for effect in store["Effects"].get(str(spell), []):
        if effect["ID"] == effect_id:
            return effect.get(field)
    return None


def talent_curve(store, spell, row):
    nodes = {node["SpellID"]: node for node in store["TraitNodes"]}
    node = nodes.get(spell)
    if node is None:
        return None
    points = store["TraitPoints"].get(str(node["DefinitionID"]), {}).get(str(row))
    if not points:
        return None
    return [points[str(rank)] for rank in range(1, node["MaxRanks"] + 1)]


def curve_verdict(values, forever, frozen):
    if values is None:
        return "missing"
    if len(values) == len(forever) and all(map(close, values, forever)):
        return "live"
    if len(values) == len(frozen) and all(map(close, values, frozen)):
        return "old"
    return "DIFFERS"


def check(path):
    store = load(path)
    builds = sorted(set(store.get("Builds", {}).values()))
    hotfixes = sorted(set(store.get("Hotfixes", {}).values()))
    print(f"{path}: client {', '.join(builds)}, hotfix push {', '.join(map(str, hotfixes))}")
    differs = 0
    for label, spell, effect_id, field, forever, frozen in ROWS:
        value = row_value(store, spell, effect_id, field)
        state = verdict(value, forever, frozen)
        differs += state == "DIFFERS"
        print(f"  {state:8} {label}: store {value}, Forever {forever:g}, frozen {frozen:g}")
    for label, spell, field, forever, frozen in POWERS:
        powers = store.get("Powers", {}).get(str(spell)) or [{}]
        value = powers[0].get(field)
        state = verdict(value, forever, frozen)
        differs += state == "DIFFERS"
        print(f"  {state:8} {label}: store {value}, Forever {forever:g}, frozen {frozen:g}")
    for label, spell, forever, frozen in STACKS:
        options = store.get("AuraOptions", {}).get(str(spell)) or {}
        value = options.get("MaxStack")
        state = verdict(value, forever, frozen)
        differs += state == "DIFFERS"
        print(f"  {state:8} {label}: store {value}, Forever {forever:g}, frozen {frozen:g}")
    for label, spell, effect_id, bit, forever in CLASS_FLAGS:
        mask = None
        for effect in store["Effects"].get(str(spell), []):
            if effect["ID"] == effect_id:
                mask = effect["ClassFlags"]["Mask"][0]
        state = "missing" if mask is None else "live" if bool(mask & bit) == forever else "old"
        print(f"  {state:8} {label}: store mask {mask}, bit {bit}")
    for label, spell, row, forever, frozen in TALENTS:
        values = talent_curve(store, spell, row)
        state = curve_verdict(values, forever, frozen)
        differs += state == "DIFFERS"
        print(f"  {state:8} {label}: store {values}, Forever {forever}, frozen {frozen}")
    return differs


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("stores", nargs="+")
    args = parser.parse_args()
    differs = sum(check(path) for path in args.stores)
    print("not in the data, to check in game:")
    for label, how in NOT_IN_DATA:
        print(f"  {label}: {how}")
    return 1 if differs else 0


if __name__ == "__main__":
    sys.exit(main())
