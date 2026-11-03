//! The Rogue changes of Blizzard's development notes of 2 November 2026 that do not depend on
//! the new talent trees, which only Forever's rules prepare. The Go reference keeps the frozen
//! data (client 1.60.1.70425) and its numbers.

use forever_engine::{
    check_prepared, contracts::prepared_v2::PreparedV2, prepare_json_with, simulate_prepared, Rules,
};
use serde_json::{json, Value};
use std::{fs, path::Path};

/// The provisional spell ids of Shank and Poised (`prepare::spells`), which no client row states.
const SHANK: i64 = 1_399_001;
const POISED: i64 = 1_399_002;

fn request(name: &str) -> Value {
    let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/mage/prepared-v2"
    ))
    .join(format!("{name}.request.json"));
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn player_mut(request: &mut Value) -> &mut Value {
    &mut request["raid"]["parties"][0]["players"][0]
}

fn with_talents(mut request: Value, talents: &str) -> Value {
    player_mut(&mut request)["talentsString"] = Value::from(talents);
    request
}

fn prepared(request: &Value, rules: Rules) -> Value {
    prepare_json_with(request.to_string().as_bytes(), "rust", rules).unwrap()
}

fn effects<'a>(prepared: &'a Value, kind: &str) -> Vec<&'a Value> {
    prepared["effects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|effect| effect["kind"] == kind)
        .collect()
}

fn spells<'a>(prepared: &'a Value, class_spell: &str) -> Vec<&'a Value> {
    prepared["player"]["spells"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|spell| spell["class_spell"] == class_spell)
        .collect()
}

fn spell<'a>(prepared: &'a Value, class_spell: &str) -> &'a Value {
    let found = spells(prepared, class_spell);
    assert_eq!(found.len(), 1, "{class_spell}");
    found[0]
}

fn flat_modifier(prepared: &Value, class_spell: &str) -> i64 {
    spell(prepared, class_spell)["cost"]["flat_modifier"]
        .as_i64()
        .unwrap()
}

fn additive(prepared: &Value, class_spell: &str) -> f64 {
    spell(prepared, class_spell)["damage_multiplier_additive"]
        .as_f64()
        .unwrap()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// Instant Poison's attack power share 0.01 (was 0.005) and Deadly Poison's 0.009 a tick (was
/// 0.0045).
#[test]
fn poison_attack_power_shares_double() {
    let request = request("production-assassination-rogue");
    let forever = prepared(&request, Rules::Forever);
    let reference = prepared(&request, Rules::GoReference);
    let share = |prepared: &Value, kind: &str| {
        effects(prepared, kind)[0]["attack_power_share"]
            .as_f64()
            .unwrap()
    };
    assert_eq!(share(&forever, "instant_poison"), 0.01);
    assert_eq!(share(&reference, "instant_poison"), 0.005);
    assert_eq!(share(&forever, "deadly_poison"), 0.009);
    assert_eq!(share(&reference, "deadly_poison"), 0.0045);
}

/// Mutilate deals 90% of weapon damage with each hand (was 75%); the flat damage it adds is
/// multiplied by the same share, so the additional amount grows with it.
#[test]
fn mutilate_deals_ninety_percent_weapon_damage() {
    let request = request("production-assassination-rogue");
    let forever = prepared(&request, Rules::Forever);
    let reference = prepared(&request, Rules::GoReference);
    for (prepared, share) in [(&forever, 0.9), (&reference, 0.75)] {
        let hits = spells(prepared, "mutilate_hit");
        assert_eq!(hits.len(), 2);
        assert!(hits
            .iter()
            .all(|hit| close(hit["damage_multiplier"].as_f64().unwrap(), share)));
        let mutilate = effects(prepared, "mutilate")[0];
        assert!(close(mutilate["weapon_share"].as_f64().unwrap(), share));
        // The flat damage per hand is the client's, 67 at rank 4.
        assert_eq!(mutilate["flat_damage"], 67.0);
    }
}

/// Hemorrhage's debuff raises Garrote's ticks as well as Rupture's, by the same 15%.
#[test]
fn hemorrhage_raises_garrote_and_rupture() {
    let request = request("production-subtlety-rogue");
    let forever = prepared(&request, Rules::Forever);
    let reference = prepared(&request, Rules::GoReference);
    let rupture = effects(&forever, "rupture")[0];
    assert_eq!(rupture["hemorrhage_garrote"], true);
    assert!(close(
        rupture["hemorrhage_multiplier"].as_f64().unwrap(),
        1.15
    ));
    let rupture = effects(&reference, "rupture")[0];
    assert!(rupture.get("hemorrhage_garrote").is_none());
    assert!(close(
        rupture["hemorrhage_multiplier"].as_f64().unwrap(),
        1.15
    ));
}

/// The ticks of a Garrote under Hemorrhage take the 15% in a fight, and Rupture's stay as they
/// were: only Garrote's damage changes when the flag is cleared.
#[test]
fn garrote_ticks_take_the_hemorrhage_debuff_in_a_fight() {
    // Hemorrhage is the builder, so its debuff is up on the target while Garrote ticks.
    let mut request = with_stealth_opener(request("production-subtlety-rogue"));
    let list = player_mut(&mut request)["rotation"]["priorityList"]
        .as_array_mut()
        .unwrap();
    let builder = list
        .iter()
        .rposition(|item| item["action"].get("castSpell").is_some())
        .unwrap();
    list[builder]["action"]["castSpell"]["spellId"]["spellId"] = json!(16511);
    let with = prepared(&request, Rules::Forever);
    let mut without = with.clone();
    for effect in without["effects"].as_array_mut().unwrap() {
        if effect["kind"] == "rupture" {
            effect.as_object_mut().unwrap().remove("hemorrhage_garrote");
        }
    }
    let damage = |value: &Value, spell_id: i64| -> f64 {
        let prepared: PreparedV2 = serde_json::from_value(value.clone()).unwrap();
        check_prepared(&prepared).unwrap();
        let result = simulate_prepared(&prepared).unwrap().result;
        let player = &result["raidMetrics"]["parties"][0]["players"][0];
        player["actions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|action| {
                action["id"]["spellId"].as_i64() == Some(spell_id)
                    && action["id"]["tag"].as_i64().unwrap_or(0) == 0
            })
            .flat_map(|action| action["targets"].as_array().unwrap())
            .map(|target| target["damage"].as_f64().unwrap_or(0.0))
            .sum()
    };
    let (garrote_with, garrote_without) = (damage(&with, 11290), damage(&without, 11290));
    assert!(garrote_without > 0.0);
    let ratio = garrote_with / garrote_without;
    assert!(
        ratio > 1.01 && ratio <= 1.15 + 1e-9,
        "Garrote ratio {ratio}"
    );
    assert_eq!(damage(&with, 11275), damage(&without, 11275));
}

/// Flawless Execution takes 15 Energy off Eviscerate (was 10).
#[test]
fn flawless_execution_is_fifteen() {
    let request = request("production-assassination-rogue");
    assert_eq!(
        flat_modifier(&prepared(&request, Rules::Forever), "eviscerate"),
        -15
    );
    assert_eq!(
        flat_modifier(&prepared(&request, Rules::GoReference), "eviscerate"),
        -10
    );
}

/// Aggression: Sinister Strike and Eviscerate by 5/10/15% (was Sinister Strike, Eviscerate and
/// Backstab by 2/4/6%). The fixture takes it at 2 points.
#[test]
fn aggression_is_five_ten_fifteen_without_backstab() {
    // Aggression is the sixteenth Combat talent; the rest of the tree stays as the production
    // request has it.
    let base = "005303105-32030320201515221-002";
    let combat = |rank: u8| {
        // The Combat tree is 17 talents; Aggression is the sixteenth.
        let mut tree: Vec<char> = "32030320201515221".chars().collect();
        tree[15] = char::from_digit(u32::from(rank), 10).unwrap();
        format!("005303105-{}-002", tree.iter().collect::<String>())
    };
    assert_eq!(combat(2), base);
    let request = request("production-combat-rogue");
    for (rank, forever, reference) in [(1u8, 0.05, 0.02), (2, 0.10, 0.04), (3, 0.15, 0.06)] {
        let request = with_talents(request.clone(), &combat(rank));
        let f = prepared(&request, Rules::Forever);
        let r = prepared(&request, Rules::GoReference);
        // Improved Eviscerate (Coup de Grace) adds 20% to Eviscerate in both; Opportunity none.
        assert!(close(additive(&f, "sinister_strike"), 1.0 + forever));
        assert!(close(additive(&r, "sinister_strike"), 1.0 + reference));
        assert!(close(additive(&f, "eviscerate"), 1.2 + forever));
        assert!(close(additive(&r, "eviscerate"), 1.2 + reference));
        // Backstab loses it under Forever's rules: only what else the tree gives remains.
        let backstab_f = additive(&f, "backstab");
        let backstab_r = additive(&r, "backstab");
        assert!(close(backstab_r - backstab_f, reference), "rank {rank}");
    }
}

/// Improved Eviscerate, renamed Coup de Grace, is 7/13/20% in the client already and in the
/// engine under either rules.
#[test]
fn coup_de_grace_is_seven_thirteen_twenty() {
    let request = request("production-combat-rogue");
    for (rank, bonus) in [('1', 0.07), ('2', 0.13), ('3', 0.20)] {
        let mut tree: Vec<char> = "32030320201515221".chars().collect();
        tree[0] = rank;
        // Remove Aggression so only Coup de Grace is left on Eviscerate.
        tree[15] = '0';
        let talents = format!("005303105-{}-002", tree.iter().collect::<String>());
        let request = with_talents(request.clone(), &talents);
        for rules in [Rules::Forever, Rules::GoReference] {
            let prepared = prepared(&request, rules);
            assert!(
                close(additive(&prepared, "eviscerate"), 1.0 + bonus),
                "rank {rank}"
            );
        }
    }
}

/// Efficient Strikes (Improved Sinister Strike) takes 3/5 Energy off Sinister Strike and
/// Hemorrhage (Sinister Strike only before).
#[test]
fn efficient_strikes_discounts_hemorrhage() {
    let request = request("production-subtlety-rogue");
    for (rank, discount) in [('1', -3), ('2', -5)] {
        let request = with_talents(
            request.clone(),
            &format!("0053021-3{rank}2301-5023203300013111051"),
        );
        let forever = prepared(&request, Rules::Forever);
        let reference = prepared(&request, Rules::GoReference);
        assert_eq!(flat_modifier(&forever, "sinister_strike"), discount);
        assert_eq!(flat_modifier(&reference, "sinister_strike"), discount);
        assert_eq!(flat_modifier(&forever, "hemorrhage"), discount);
        assert_eq!(flat_modifier(&reference, "hemorrhage"), 0);
    }
}

/// Shank is learned by every rogue holding an off hand weapon under Forever's rules: 5 Energy,
/// a 10 sec cooldown, melee range, an off hand strike for 50% weapon damage that generates 25
/// Energy at rank 2. The reference has no Shank.
#[test]
fn shank_is_a_baseline_off_hand_strike() {
    for name in [
        "production-combat-rogue",
        "production-assassination-rogue",
        "production-subtlety-rogue",
    ] {
        let request = request(name);
        let forever = prepared(&request, Rules::Forever);
        let shank = spell(&forever, "shank");
        assert_eq!(shank["action_id"]["spell_id"], SHANK, "{name}");
        assert_eq!(shank["cost"]["resource"], "energy");
        assert_eq!(shank["cost"]["base_cost"], 5);
        assert_eq!(shank["cd"]["duration_ns"], 10_000_000_000i64);
        assert_eq!(shank["damage_multiplier"], 0.5);
        assert_eq!(shank["proc_mask"], json!(["ProcMaskMeleeOHSpecial"]));
        assert_eq!(effects(&forever, "shank")[0]["energy_gain"], 25.0);
        let reference = prepared(&request, Rules::GoReference);
        assert!(spells(&reference, "shank").is_empty(), "{name}");
        assert!(effects(&reference, "shank").is_empty(), "{name}");
    }
}

/// A rogue with no off hand weapon has no Shank.
#[test]
fn shank_needs_an_off_hand_weapon() {
    let mut request = request("production-combat-rogue");
    no_off_hand_items(&mut request);
    let forever = prepared(&request, Rules::Forever);
    assert!(spells(&forever, "shank").is_empty());
    assert!(effects(&forever, "shank").is_empty());
}

/// Poised: the aura the fight starts with, taking 10 Energy off the first Energy ability.
#[test]
fn poised_is_prepared_under_forever_rules_only() {
    let request = request("production-combat-rogue");
    let forever = prepared(&request, Rules::Forever);
    let poised = effects(&forever, "poised")[0];
    assert_eq!(poised["aura"], "Poised");
    assert_eq!(poised["cost_reduction"], 10);
    let classes: Vec<&str> = poised["class_spells"]
        .as_array()
        .unwrap()
        .iter()
        .map(|class| class.as_str().unwrap())
        .collect();
    for energy in [
        "sinister_strike",
        "eviscerate",
        "garrote",
        "shank",
        "rupture",
    ] {
        assert!(classes.contains(&energy), "{energy}");
    }
    assert!(!classes.contains(&"stealth"));
    let reference = prepared(&request, Rules::GoReference);
    assert!(effects(&reference, "poised").is_empty());
}

/// Empties the off hand slot of the production combat rogue, whose equipment lists the main hand
/// (12590) and then the off hand (13368).
fn no_off_hand_items(request: &mut Value) {
    let items = player_mut(request)["equipment"]["items"]
        .as_array_mut()
        .unwrap();
    let off_hand = items
        .iter()
        .position(|item| item["id"] == 13368)
        .expect("the off hand dagger");
    items[off_hand] = json!({});
}

/// Adds a Stealth before the pull and a Garrote opener to a rogue's rotation, the way the
/// application's presets open.
fn with_stealth_opener(mut request: Value) -> Value {
    let rotation = &mut player_mut(&mut request)["rotation"];
    rotation["prepullActions"] = json!([{
        "action": {"castSpell": {"spellId": {"spellId": 1784}}},
        "doAtValue": {"const": {"val": "-1s"}},
    }]);
    let garrote = json!({"action": {
        "castSpell": {"spellId": {"spellId": 11290}},
        "condition": {"auraIsActive": {"auraId": {"spellId": 1784}}},
    }});
    rotation["priorityList"]
        .as_array_mut()
        .unwrap()
        .insert(0, garrote);
    request
}

/// Shank off the cooldown whenever Energy is below the threshold, ahead of the last cast line
/// (the builder).
fn with_shank(mut request: Value, threshold: u32) -> Value {
    let list = player_mut(&mut request)["rotation"]["priorityList"]
        .as_array_mut()
        .unwrap();
    let builder = list
        .iter()
        .rposition(|item| item["action"].get("castSpell").is_some())
        .unwrap();
    list.insert(
        builder,
        json!({"action": {
            "castSpell": {"spellId": {"spellId": SHANK}},
            "condition": {"cmp": {
                "lhs": {"currentEnergy": {}},
                "op": "OpLt",
                "rhs": {"const": {"val": threshold.to_string()}},
            }},
        }}),
    );
    request
}

/// The log of the first fight of a prepared request.
fn first_fight_log(request: &Value, rules: Rules) -> String {
    let mut value = prepared(request, rules);
    value["sim"]["iterations"] = json!(1);
    value["sim"]["debug_first_iteration"] = json!(true);
    let prepared: PreparedV2 = serde_json::from_value(value).unwrap();
    check_prepared(&prepared).unwrap();
    let report = simulate_prepared(&prepared).unwrap();
    report.result["logs"].as_str().unwrap().to_string()
}

fn time_of(line: &str) -> f64 {
    line[1..line.find(']').unwrap()].parse().unwrap()
}

/// The first Energy ability costs 10 less, once: Garrote from Stealth is 40 under Forever's
/// rules (Poised is up) and 50 under the reference.
#[test]
fn poised_discounts_the_opener_once() {
    let request = with_stealth_opener(request("production-combat-rogue"));
    let forever = first_fight_log(&request, Rules::Forever);
    let reference = first_fight_log(&request, Rules::GoReference);
    let garrote = |log: &str| {
        log.lines()
            .find(|line| line.contains("Casting {SpellID: 11290}"))
            .unwrap()
            .to_string()
    };
    assert!(
        garrote(&forever).contains("Cost = 40.000"),
        "{}",
        garrote(&forever)
    );
    assert!(garrote(&reference).contains("Cost = 50.000"));
    assert!(forever.contains(&format!("Aura gained: {{SpellID: {POISED}}}")));
    assert!(forever.contains(&format!("Aura faded: {{SpellID: {POISED}}}")));
    assert!(!reference.contains(&POISED.to_string()));
    // Gone after the opener: the next Energy ability pays in full.
    let faded = forever
        .lines()
        .position(|line| line.contains(&format!("Aura faded: {{SpellID: {POISED}}}")))
        .unwrap();
    assert!(forever
        .lines()
        .skip(faded)
        .filter(|line| line.contains("Casting {SpellID:") && line.contains("Cost = "))
        .filter(|line| !line.contains("Cost = 0.000"))
        .all(|line| !line.contains("Cost = 40.000") || line.contains("{SpellID: 11290}")));
}

/// Shank in a rotation: it casts every 10 sec or so at the rotation's call, costs 5 Energy and
/// gives 25.
#[test]
fn shank_casts_and_generates_energy() {
    let request = with_shank(request("production-combat-rogue"), 60);
    let log = first_fight_log(&request, Rules::Forever);
    let casts: Vec<&str> = log
        .lines()
        .filter(|line| line.contains(&format!("Casting {{SpellID: {SHANK}}}")))
        .collect();
    assert!(casts.len() >= 4, "{} Shank casts", casts.len());
    assert!(casts
        .iter()
        .all(|line| line.contains("Cost = 5.000") || line.contains("Cost = 0.000")));
    // The cooldown holds it to one cast in ten seconds.
    for pair in casts.windows(2) {
        assert!(
            time_of(pair[1]) - time_of(pair[0]) >= 10.0 - 1e-9,
            "{pair:?}"
        );
    }
    let gains = log
        .lines()
        .filter(|line| line.contains(&format!("Gained 25.000 energy from {{SpellID: {SHANK}}}")))
        .count();
    assert!(gains >= 4, "{gains} Shank Energy gains");
    // It strikes with the off hand: the log names Shank's damage on the target.
    assert!(log
        .lines()
        .any(|line| line.contains(&format!("{{SpellID: {SHANK}}}")) && line.contains("damage")));
}

/// A rotation line naming Shank is skipped where there is no Shank to cast (the reference rules,
/// or a rogue without an off hand weapon), as any rotation line naming a spell the player does
/// not have.
#[test]
fn a_rotation_line_naming_a_missing_shank_never_casts() {
    let request = with_shank(request("production-combat-rogue"), 60);
    let reference = first_fight_log(&request, Rules::GoReference);
    assert!(!reference.contains(&SHANK.to_string()));
    let mut no_off_hand = request;
    no_off_hand_items(&mut no_off_hand);
    let log = first_fight_log(&no_off_hand, Rules::Forever);
    assert!(!log.contains(&SHANK.to_string()));
}

/// Hemorrhage with a Dagger deals the client's 145% in place of its plain 100%, never on top of
/// it, under either rules: the notes' dagger fix needs nothing of the engine.
#[test]
fn a_daggers_hemorrhage_share_replaces_the_plain_share() {
    let request = request("production-subtlety-rogue");
    for rules in [Rules::Forever, Rules::GoReference] {
        let prepared = prepared(&request, rules);
        assert_eq!(spell(&prepared, "hemorrhage")["damage_multiplier"], 1.45);
    }
}
