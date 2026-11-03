//! The tree independent lines of Blizzard's development notes of 2 November 2026 that the first
//! pass (tests/hotfix_20261103.rs) left: Paladin and Mage. They are prepared under Forever's rules
//! only; the Go reference keeps the frozen data (client 1.60.1.70425) and its numbers.

use forever_engine::{
    contracts::prepared_v2::PreparedV2, prepare_json_with, simulate_prepared, Rules,
};
use serde_json::Value;
use std::{fs, path::Path};

fn request(dir: &str, name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(dir)
        .join(format!("{name}.request.json"));
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn prepared(request: &Value, rules: Rules) -> Value {
    prepare_json_with(request.to_string().as_bytes(), "rust", rules).unwrap()
}

fn effect<'a>(prepared: &'a Value, kind: &str) -> &'a Value {
    prepared["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["kind"] == kind)
        .unwrap_or_else(|| panic!("no {kind} effect"))
}

fn aura<'a>(prepared: &'a Value, label_prefix: &str) -> &'a Value {
    prepared["player"]["auras"]
        .as_array()
        .unwrap()
        .iter()
        .find(|aura| aura["label"].as_str().unwrap().starts_with(label_prefix))
        .unwrap_or_else(|| panic!("no aura {label_prefix}"))
}

/// The damage multipliers every prepared spell of this class spell carries.
fn damage_multipliers(prepared: &Value, class_spell: &str) -> Vec<f64> {
    prepared["player"]["spells"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|spell| spell["class_spell"] == class_spell)
        .map(|spell| spell["damage_multiplier"].as_f64().unwrap())
        .collect()
}

const FIXTURES: &str = "fixtures/mage/prepared-v2";
/// The Retribution fixture's talent string is in the Go reference's order; this one is the same
/// build in client 1.60.1.70707's, with Vengeance and Two-Handed Weapon Specialization at 3.
const RET_70707: &str = "050050002-503-05233301001301351";

fn retribution(talents: &str) -> Value {
    let mut request = request(FIXTURES, "production-retribution-paladin");
    request["raid"]["parties"][0]["players"][0]["talentsString"] = talents.into();
    request
}

/// Vengeance stacks up to 5 times under the notes (3 before); a stack is still 3% at 3 points.
#[test]
fn vengeance_stacks_five_times() {
    let request = retribution(RET_70707);
    let forever = prepared(&request, Rules::Forever);
    assert_eq!(forever["player"]["talents"]["vengeance"], 3);
    assert_eq!(aura(&forever, "Vengeance")["max_stacks"], 5);
    assert_eq!(effect(&forever, "vengeance")["per_stack"], 0.03);

    // The reference reads the same fixture in its own order, and keeps 3 stacks.
    let reference = prepared(
        &self::request(FIXTURES, "production-retribution-paladin"),
        Rules::GoReference,
    );
    assert_eq!(reference["player"]["talents"]["vengeance"], 3);
    assert_eq!(aura(&reference, "Vengeance")["max_stacks"], 3);
}

/// Two-Handed Weapon Specialization raises Holy damage as well as Physical, 6% at 3 points, but
/// not a Holy heal. The reference raises neither Holy spell.
#[test]
fn two_handed_specialization_raises_holy_damage() {
    let forever = prepared(&retribution(RET_70707), Rules::Forever);
    let reference = prepared(
        &request(FIXTURES, "production-retribution-paladin"),
        Rules::GoReference,
    );
    for spell in [
        "holy_strike",
        "consecration",
        "exorcism",
        "hammer_of_wrath",
        "judgement_of_command",
        "seal_of_command_proc",
        "seal_of_righteousness_proc",
        "judgement_of_wisdom",
    ] {
        assert!(
            damage_multipliers(&forever, spell)
                .iter()
                .all(|multiplier| (multiplier - 1.06).abs() < 1e-9),
            "{spell} under Forever's rules: {:?}",
            damage_multipliers(&forever, spell)
        );
        assert!(
            damage_multipliers(&reference, spell)
                .iter()
                .all(|multiplier| (multiplier - 1.0).abs() < 1e-9),
            "{spell} under the reference: {:?}",
            damage_multipliers(&reference, spell)
        );
    }
    // Seal of Light's proc heals, so the Holy damage bonus leaves it alone.
    for heal in ["seal_of_light_proc", "holy_light", "flash_of_light"] {
        assert!(
            damage_multipliers(&forever, heal)
                .iter()
                .all(|multiplier| *multiplier == 1.0),
            "{heal}: {:?}",
            damage_multipliers(&forever, heal)
        );
    }
}

const ARCANE_BOARD: &str = "fixtures/dev-notes-20261102";
const CLEARCASTING: &str = "SpellID: 12536";

/// Under Forever's rules the effect says the missiles roll; the reference's prepared input does
/// not carry the field at all.
#[test]
fn arcane_concentration_flag_is_forever_only() {
    let forever = prepared(&request(ARCANE_BOARD, "arcane-mage"), Rules::Forever);
    assert_eq!(effect(&forever, "arcane_concentration")["missiles"], true);

    // An Arcane fixture in the Go reference's order.
    let reference = prepared(
        &request(FIXTURES, "arcane-preset-p0-freezing-band"),
        Rules::GoReference,
    );
    assert!(effect(&reference, "arcane_concentration")
        .get("missiles")
        .is_none());
}

/// The first iteration's log lines of the Arcane board, a fight of five minutes.
fn arcane_log() -> Vec<String> {
    let mut request = request(ARCANE_BOARD, "arcane-mage");
    request["simOptions"]["debugFirstIteration"] = true.into();
    request["simOptions"]["iterations"] = 1.into();
    request["encounter"]["duration"] = 120.into();
    request["encounter"]["durationVariation"] = 0.into();
    let prepared = prepared(&request, Rules::Forever);
    let typed: PreparedV2 = serde_json::from_value(prepared).unwrap();
    let report = simulate_prepared(&typed).unwrap();
    let logs = report.result["logs"].as_str().unwrap().to_string();
    logs.lines().map(str::to_string).collect()
}

fn time(line: &str) -> f64 {
    line[1..line.find(']').unwrap()].parse().unwrap()
}

/// A missile of Arcane Missiles (rank 7's, 10274) that lands rolls for Clearcasting: the log
/// shows the aura gained or refreshed on the line after a missile's hit.
#[test]
fn arcane_missiles_proc_clearcasting() {
    let lines = arcane_log();
    let missile_hit = |line: &str| {
        line.contains("SpellID: 10274} Hit for") || line.contains("SpellID: 10274} Crit for")
    };
    let procs = lines
        .windows(2)
        .filter(|pair| {
            missile_hit(&pair[0])
                && pair[1].contains(CLEARCASTING)
                && (pair[1].contains("Aura gained") || pair[1].contains("Aura refreshed"))
        })
        .count();
    assert!(procs > 0, "no missile proc Clearcasting");
}

/// An Arcane Missiles cast that finds Clearcasting from an earlier moment and Missile Barrage up
/// does not spend the Clearcasting (it fades at its own end, 1.5 sec after the last gain).
#[test]
fn clearcasting_survives_missile_barrage_missiles() {
    const MISSILES_RANK_7: &str = "Casting {SpellID: 10212}";
    const MISSILE_BARRAGE: &str = "SpellID: 44404";
    let lines = arcane_log();
    let (mut clearcasting_since, mut clearcasting_until, mut barrage) = (None, 0.0, false);
    let mut kept = 0;
    for (i, line) in lines.iter().enumerate() {
        let now = time(line);
        if line.contains(CLEARCASTING) {
            if line.contains("Aura gained") {
                clearcasting_since = Some(now);
            }
            if line.contains("Aura gained") || line.contains("Aura refreshed") {
                clearcasting_until = now + 1.5;
            }
            if line.contains("Aura faded") {
                clearcasting_since = None;
            }
        }
        if line.contains(MISSILE_BARRAGE) {
            if line.contains("Aura gained") || line.contains("Aura refreshed") {
                barrage = true;
            }
            if line.contains("Aura faded") {
                barrage = false;
            }
        }
        if !line.contains(MISSILES_RANK_7) {
            continue;
        }
        let older = clearcasting_since.is_some_and(|since| since < now);
        if older && barrage && clearcasting_until > now + 1e-9 {
            let spent = lines[i + 1..]
                .iter()
                .take_while(|next| time(next) == now)
                .any(|next| next.contains(CLEARCASTING) && next.contains("Aura faded"));
            assert!(!spent, "Clearcasting spent by the cast at {now}");
            kept += 1;
        }
    }
    assert!(kept > 0, "no cast with Clearcasting and Missile Barrage up");
}
