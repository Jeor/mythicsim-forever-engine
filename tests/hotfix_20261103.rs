//! The live hotfix cache of 3 November 2026 (client 1.60.1.70707) and Blizzard's development
//! notes of 2 November, which only Forever's rules prepare. The Go reference keeps the frozen
//! data (client 1.60.1.70425) and its numbers.

use forever_engine::{prepare_json_with, Rules};
use serde_json::Value;
use std::{fs, path::Path};

fn request(name: &str) -> Value {
    let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/mage/prepared-v2"
    ))
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

/// The ranged Attack Power of each stat aura combination that holds Aspect of the Hawk, minus
/// the same combination without it.
fn hawk_ranged_attack_power(prepared: &Value) -> f64 {
    let table = prepared["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| {
            effect["combos"].is_array()
                && effect["auras"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|aura| aura == "Aspect of the Hawk")
        })
        .expect("a stat aura table with Aspect of the Hawk");
    let bit = table["auras"]
        .as_array()
        .unwrap()
        .iter()
        .position(|aura| aura == "Aspect of the Hawk")
        .unwrap();
    let combos = table["combos"].as_array().unwrap();
    let rap = |i: usize| combos[i]["RangedAttackPower"].as_f64().unwrap();
    rap(1 << bit) - rap(0)
}

/// Aimed Shot, Arcane Shot, Sniper Shot and Serpent Sting at the ranks a level 60 Hunter casts,
/// with the notes' attack power shares for Arcane Shot (15%) and Serpent Sting (5% a tick).
#[test]
fn hunter_shots_take_the_hotfix() {
    let request = request("production-marksmanship-hunter");
    let forever = prepared(&request, Rules::Forever);
    let reference = prepared(&request, Rules::GoReference);

    assert_eq!(effect(&forever, "aimed_shot")["flat_bonus"], 350.0);
    assert_eq!(effect(&reference, "aimed_shot")["flat_bonus"], 166.0);
    assert_eq!(effect(&forever, "arcane_shot")["base_damage"], 280.0);
    assert_eq!(effect(&forever, "arcane_shot")["rap_coefficient"], 0.15);
    assert_eq!(effect(&reference, "arcane_shot")["base_damage"], 217.0);
    assert_eq!(effect(&reference, "arcane_shot")["rap_coefficient"], 0.1);
    assert_eq!(effect(&forever, "sniper_shot")["flat_bonus"], 590.0);
    assert_eq!(effect(&reference, "sniper_shot")["flat_bonus"], 295.0);
    // Serpent Sting rank 8 under Forever's rules (rank 9 is an Ahn'Qiraj 20 book rank).
    let sting = effect(&forever, "serpent_sting");
    assert_eq!(sting["spell_id"], 13555);
    assert_eq!(sting["tick_base"], 100.0);
    assert_eq!(sting["attack_power_share"], 0.05);
    assert_eq!(effect(&reference, "serpent_sting")["tick_base"], 111.0);
    assert_eq!(
        effect(&reference, "serpent_sting")["attack_power_share"],
        0.035
    );
}

/// Aspect of the Hawk rank 6 (the rank without the book) gives 110 ranged Attack Power, was 55;
/// the reference casts rank 7 at 120.
#[test]
fn aspect_of_the_hawk_rank_6_is_110() {
    let request = request("production-marksmanship-hunter");
    assert_eq!(
        effect(&prepared(&request, Rules::Forever), "aspect_of_the_hawk")["spell_id"],
        14322
    );
    assert_eq!(
        hawk_ranged_attack_power(&prepared(&request, Rules::Forever)),
        110.0
    );
    assert_eq!(
        hawk_ranged_attack_power(&prepared(&request, Rules::GoReference)),
        120.0
    );
}

/// Trueshot Aura rank 5 gives 100 ranged Attack Power (was 50); the reference reads rank 4 at
/// 75. Measured on a party that gains the aura against the same party without it.
#[test]
fn trueshot_aura_is_100() {
    let without = request("production-arms-warrior");
    let mut with = without.clone();
    with["raid"]["parties"][0]["buffs"]["trueshotAura"] = Value::Bool(true);
    let rap = |request: &Value, rules: Rules| {
        prepared(request, rules)["player"]["stats"]["RangedAttackPower"]
            .as_f64()
            .unwrap()
    };
    assert_eq!(
        rap(&with, Rules::Forever) - rap(&without, Rules::Forever),
        100.0
    );
    assert_eq!(
        rap(&with, Rules::GoReference) - rap(&without, Rules::GoReference),
        75.0
    );
}
