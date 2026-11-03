//! Live server hotfix pushes 113531 and 113554, which reached client 1.60.1.70707 on 30 October
//! 2026 through the DBCache rather than a client build (wago.tools' CDN tables do not carry
//! them). Only Forever's rules read them; the Go reference keeps the frozen data
//! (client 1.60.1.70425) and its goldens hold.
//!
//! The rows here are the frozen rows with the hotfixed columns changed, each change naming its
//! SpellEffect or Spell column. They come from wowsims/forever #769 (push 113531) and #770 (push
//! 113554), which regenerated the store from Raidbots' `forever` DBCache.bin, and agree with
//! Wowhead's Forever tooltips. Paladin and Priest rows are not here.
//!
//! The live hotfix cache of 3 November 2026 (still client 1.60.1.70707), which carries the Hunter
//! part of Blizzard's development notes of 2 November, is here too: wowsims/forever #824
//! regenerated the store from it (`--cdn --dbcache --carry-hotfixes`).

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::data::spells::{Effect, Spell};

use super::spelldata::Ladder;

use super::dbcenums;
use super::sim::Rules;
use super::spelldata::must_find;

/// The row a spell has under `rules`: the hotfixed row under Forever's rules where one exists,
/// the frozen row otherwise. Panics, as [`must_find`] does, for a spell the store lacks.
pub(crate) fn row(rules: Rules, id: i32) -> &'static Spell {
    match rules {
        Rules::Forever => hotfixed(id).unwrap_or_else(|| must_find(id)),
        Rules::GoReference => must_find(id),
    }
}

/// The hotfixed row, if the pushes changed one the engine reads.
pub(crate) fn hotfixed(id: i32) -> Option<&'static Spell> {
    static ROWS: OnceLock<HashMap<i32, &'static Spell>> = OnceLock::new();
    ROWS.get_or_init(build).get(&id).copied()
}

/// Ice Lance, every rank: the Frozen bonus and the cooldown below.
pub(crate) const ICE_LANCE_RANKS: [i32; 6] = [1312002, 400640, 1240044, 1240045, 1240046, 1240047];

/// Mage cooldown category 19, Fire Blast's, which Ice Lance joins (push 113531).
pub(crate) const FIRE_BLAST_CATEGORY: i16 = 19;

/// Hunter cooldown category 411, Explosive Trap's and Immolation Trap's, which Freezing Trap and
/// Frost Trap join from their own 2183 (push 113554).
pub(crate) const FIRE_TRAP_CATEGORY: i16 = 411;

/// The Hunter shot rows the live hotfix cache of 3 November raised, as (spell, SpellEffect, base
/// points). wowsims/forever #824's `spells_auto_gen.go` diff.
pub(crate) const HUNTER_SHOTS_20261103: [(i32, i32, f64); 32] = [
    // Aimed Shot ranks 1 to 6: 20, 34, 55, 89, 125, 166 before.
    (19434, 696631, 40.0),
    (20900, 697762, 70.0),
    (20901, 699653, 115.0),
    (20902, 697763, 185.0),
    (20903, 699670, 260.0),
    (20904, 697764, 350.0),
    // Arcane Shot ranks 1 to 8: 20, 26, 39, 65, 94, 134, 170, 217 before.
    (3044, 680797, 25.0),
    (14281, 691213, 35.0),
    (14282, 691984, 50.0),
    (14283, 692127, 85.0),
    (14284, 690859, 120.0),
    (14285, 690933, 175.0),
    (14286, 690976, 220.0),
    (14287, 691069, 280.0),
    // Serpent Sting ranks 1 to 9, a tick: 2, 6, 12, 22, 34, 48, 64, 83, 111 before.
    (1978, 679708, 3.0),
    (13549, 691588, 7.0),
    (13550, 691589, 14.0),
    (13551, 691570, 26.0),
    (13552, 690760, 40.0),
    (13553, 690132, 58.0),
    (13554, 691479, 75.0),
    (13555, 690655, 100.0),
    (25295, 702403, 135.0),
    // Sniper Shot ranks 1 to 3: 160, 225, 295 before.
    (1310687, 1340384, 320.0),
    (1310785, 1340617, 450.0),
    (1310786, 1340619, 590.0),
    // Aspect of the Hawk ranks 1 to 6, ranged Attack Power: 15, 22, 30, 38, 47, 55 before. Rank 6
    // (14322) is the one a level 60 Hunter knows under Forever's rules (rank 7, 25296, is an
    // Ahn'Qiraj 20 book rank, and stays 120).
    (13165, 690400, 20.0),
    (14318, 691884, 35.0),
    (14319, 691886, 50.0),
    (14320, 692212, 70.0),
    (14321, 692607, 90.0),
    (14322, 692138, 110.0),
];

/// Trueshot Aura's ranged Attack Power by rank under the live hotfix cache of 3 November: 20, 35,
/// 50, 75 and 100 (rank 5, 20906, SpellEffect 697683, was 50).
pub(crate) const TRUESHOT_AURA_20261103: [(i32, f64); 5] = [
    (1299346, 20.0),
    (1299348, 35.0),
    (19506, 50.0),
    (20905, 75.0),
    (20906, 100.0),
];

/// Every Starfire rank (1 to 7).
pub(crate) const STARFIRE_RANKS: [i32; 7] = [2912, 8949, 8950, 8951, 9875, 9876, 25298];

/// Starfire's damage under the development notes of 2 November 2026: "about 40%" more, which no
/// row states yet.
pub(crate) const STARFIRE_NOTES_SCALE: f64 = 1.4;

/// Talents whose per rank values Blizzard's development notes of 2 November 2026 change while
/// keeping the rank count, as the curve each takes under Forever's rules: one row per effect, in
/// effect order, as the client states curves. No client row states them yet.
const TALENT_CURVES_20261102: &[(i32, &[&[f64]])] = &[
    // Hunter: Barrage 5/10/15% (was 3/7/10%; the 3 November cache states it).
    (19461, &[&[5.0, 10.0, 15.0]]),
    // Warrior: Two-Handed Weapon Specialization 2/4/6% (was 1/2/3%), Defiance 7/13/20% threat
    // (was 5/10/15%), and Improved Slam's cooldown cut 2.5/5 sec (its third effect, was 3 sec at
    // either rank; the cast time and GCD curves keep their values).
    (12163, &[&[2.0, 4.0, 6.0]]),
    (12792, &[&[7.0, 13.0, 20.0]]),
    (
        12862,
        &[&[-250.0, -500.0], &[-250.0, -500.0], &[-2500.0, -5000.0]],
    ),
    // Warlock: Shadow and Flame's Shadow and Fire damage 4/8/12/16/20% (its third and fourth
    // effects, were 2/4/6/8/10%; Go reads the third). The second, Conflagrate's chance to keep
    // Immolate, and the first keep theirs.
    (
        426316,
        &[
            &[4.0, 8.0, 12.0, 16.0, 20.0],
            &[20.0, 40.0, 60.0, 80.0, 100.0],
            &[4.0, 8.0, 12.0, 16.0, 20.0],
            &[4.0, 8.0, 12.0, 16.0, 20.0],
        ],
    ),
    // Shaman: Lightning Overload 10/20/30% (was 3/7/10%), Mental Dexterity 50/100/150% of
    // Intellect (was 33/67/100%), Flurry 6/12/18/24/30% (was 5/10/15/20/25%).
    (408438, &[&[10.0, 20.0, 30.0]]),
    (415140, &[&[50.0, 100.0, 150.0]]),
    (16256, &[&[6.0, 12.0, 18.0, 24.0, 30.0]]),
    // Rogue: Aggression 5/10/15% (was 2/4/6%), on Sinister Strike and Eviscerate only; the mask
    // is in the Rogue's registration.
    (18427, &[&[5.0, 10.0, 15.0]]),
];

/// A talent's ladder under `rules`: the curve of [`TALENT_CURVES_20261102`] under Forever's rules
/// where one is listed, the frozen data's otherwise (Go `Talent`).
pub(crate) fn talent(rules: Rules, spell_id: i32, max_ranks: i32) -> Ladder {
    static LADDERS: OnceLock<HashMap<i32, Ladder>> = OnceLock::new();
    if rules == Rules::Forever {
        let ladders = LADDERS.get_or_init(|| {
            TALENT_CURVES_20261102
                .iter()
                .map(|(id, curve)| {
                    let rows: Vec<Vec<f64>> = curve.iter().map(|row| row.to_vec()).collect();
                    (*id, Ladder::talent_with_curve(*id, &rows))
                })
                .collect()
        });
        if let Some(ladder) = ladders.get(&spell_id) {
            assert_eq!(ladder.len(), max_ranks, "talent {spell_id} rank count");
            return ladder.clone();
        }
    }
    Ladder::talent(spell_id, max_ranks)
}

/// Every Mutilate hit spell, a main hand and an off hand strike for each of four ranks, whose
/// weapon damage effect the notes of 2 November raise from 75% to 90%. The flat damage a hit adds
/// is read through the same share, so it grows with it.
pub(crate) const MUTILATE_HIT_RANKS: [i32; 8] = [
    1310705, 1310706, 399960, 399961, 1241585, 1241588, 1241586, 1241590,
];

/// Mutilate's weapon share under the development notes of 2 November: 90% (was 75%).
pub(crate) const MUTILATE_WEAPON_SHARE_20261102: f64 = 90.0;

/// Flawless Execution's cost reduction under the notes of 2 November: 15 Energy (was 10), on
/// SpellEffect 1340441 of 1310711.
pub(crate) const FLAWLESS_EXECUTION_COST_20261102: f64 = -15.0;

/// Every rank of Slam, whose cooldown the notes of 2 November cut from 18 to 15 sec.
pub(crate) const SLAM_RANKS: [i32; 5] = [1240193, 1464, 8820, 11604, 11605];

fn build() -> HashMap<i32, &'static Spell> {
    let mut rows: HashMap<i32, Spell> = HashMap::new();
    let mut patch = |id: i32, change: &dyn Fn(&mut Spell)| {
        let row = rows.entry(id).or_insert_with(|| must_find(id).clone());
        change(row);
    };

    // Mage (push 113531).
    // Ice Lance: the bonus against Frozen targets (effect 2, a dummy) 200% to 300%, and an 8 sec
    // cooldown on category 19, which Fire Blast shares. Rank 1 (1312002) had the cooldown
    // already.
    for id in ICE_LANCE_RANKS {
        patch(id, &|row| {
            row.category = FIRE_BLAST_CATEGORY;
            row.category_cooldown_ms = 8000;
            row.effects[1].base_points = 300.0;
        });
    }
    // Winter's Chill: Cone of Cold damage taken a stack, SpellEffect 688253, 5% to 20%.
    patch(12579, &|row| effect(row, 688253).base_points = 20.0);
    // Arcane Blast's buff: 12% of Intellect as spell damage a stack (1045737, was 15), Arcane
    // Blast's cost 100% more a stack (1045738, was 175), and 18 sec (was 15).
    patch(400573, &|row| {
        row.duration_ms = 18000;
        effect(row, 1045737).base_points = 12.0;
        effect(row, 1045738).base_points = 100.0;
    });
    // Ice Barrier, every rank: Frost damage while shielded 3% to 5% (hit and periodic).
    for id in [11426, 13031, 13032, 13033] {
        patch(id, &|row| {
            for effect in &mut row.effects {
                if effect.aura == dbcenums::A_ADD_PCT_MODIFIER {
                    effect.base_points = 5.0;
                }
            }
        });
    }
    // Frigid Power: 3% to 15% (SpellEffect 1365007 and 1365008).
    patch(1326891, &|row| {
        effect(row, 1365007).base_points = 15.0;
        effect(row, 1365008).base_points = 15.0;
    });
    // Arctic Reach: the range effect (694455) covers Ice Lance too, the first mask word 160 to 131232.
    patch(16757, &|row| {
        effect(row, 694455).class_flags.mask[0] = 131_232
    });

    // Druid (push 113531).
    // Primal Bite: rank 4 (1238073) is gone, and rank 3 (1238070) is the level 60 rank with
    // rank 4's 77 flat damage (SpellEffect 1235073, was 59 at level 48).
    patch(1238070, &|row| {
        let level_60 = must_find(1238073);
        row.spell_level = level_60.spell_level;
        row.base_level = level_60.base_level;
        for (effect, top) in row.effects.iter_mut().zip(&level_60.effects) {
            effect.base_points = top.base_points;
            effect.spell_level = top.spell_level;
        }
    });
    // Shifting Power: 16 sec cooldown to 8.
    patch(1322605, &|row| row.cooldown_ms = 8000);
    // Wild Growth, every rank: instant to a 3 sec cast.
    for id in [408120, 1238214, 1238215] {
        patch(id, &|row| row.cast_time_ms = 3000);
    }
    // Naturalist: the cast time effect (695065) covers Healing Touch and Wild Growth, the second mask
    // word 33554432 to 67108864. Its damage curve doubles (spell_data in the druid's preparation).
    patch(17069, &|row| {
        effect(row, 695065).class_flags.mask[1] = 67_108_864
    });

    // Hunter (push 113554): Freezing Trap, every rank, and Frost Trap leave category 2183.
    for id in [1499, 14310, 14311, 13809] {
        patch(id, &|row| row.category = FIRE_TRAP_CATEGORY);
    }

    // Hunter (live hotfix cache of 3 November, development notes of 2 November): every rank of
    // Aimed Shot (about 110% more bonus damage), Arcane Shot (about 30% more), Serpent Sting
    // (about 20% more a tick), Sniper Shot (twice the bonus damage) and Aspect of the Hawk ranks
    // 1 to 6. The old values are 70707's, which the frozen data shares for these rows.
    for (spell, id, points) in HUNTER_SHOTS_20261103 {
        patch(spell, &|row| effect(row, id).base_points = points);
    }

    // Druid (development notes of 2 November): Starfire's base damage "increased by about 40%".
    // No row states the new values yet, so every rank's damage (base points and points per
    // level) is scaled by [`STARFIRE_NOTES_SCALE`], a placeholder until the client carries them.
    for id in STARFIRE_RANKS {
        patch(id, &|row| {
            let damage = &mut row.effects[0];
            damage.base_points *= STARFIRE_NOTES_SCALE;
            damage.ppl *= STARFIRE_NOTES_SCALE;
        });
    }

    // Warrior (notes of 2 November): Defensive Stance's threat bonus 30% to 80% (Defensive Stance
    // Passive, SpellEffect 686475), and Slam's 18 sec cooldown to 15 on every rank.
    patch(7376, &|row| effect(row, 686475).base_points = 80.0);
    for id in SLAM_RANKS {
        patch(id, &|row| row.category_cooldown_ms = 15000);
    }
    // Warlock (notes of 2 November): Wrack's spell damage coefficient doubles (0.143 to 0.286
    // a tick, "0.286 per second of cast time") and its base damage a second grows 2.5 a level,
    // from 36 at its level 40 to 86 at 60 (was 36 at every level).
    patch(1316697, &|row| {
        let tick = effect(row, 1349556);
        tick.ppl = 2.5;
        tick.sp_coef *= 2.0;
    });
    // Shaman (notes of 2 November): Stormstrike's cooldown 8 sec to 6.
    patch(17364, &|row| row.cooldown_ms = 6000);
    // Rogue (notes of 2 November): Mutilate deals 90% weapon damage with each hand (was 75%), and
    // Flawless Execution takes 15 Energy off Eviscerate (was 10). The client has no rows for
    // either yet.
    for id in MUTILATE_HIT_RANKS {
        patch(id, &|row| {
            for effect in &mut row.effects {
                if effect.effect_type == dbcenums::E_WEAPON_PERCENT_DAMAGE {
                    effect.base_points = MUTILATE_WEAPON_SHARE_20261102;
                }
            }
        });
    }
    patch(1310711, &|row| {
        effect(row, 1340441).base_points = FLAWLESS_EXECUTION_COST_20261102
    });

    rows.into_iter()
        .map(|(id, row)| (id, &*Box::leak(Box::new(row))))
        .collect()
}

/// The effect with this SpellEffect ID. Panics when the row has none: the patch names a row the
/// frozen data does not have.
fn effect(row: &mut Spell, id: i32) -> &mut Effect {
    let spell = row.id;
    row.effects
        .iter_mut()
        .find(|effect| effect.id == id)
        .unwrap_or_else(|| panic!("spell {spell} has no SpellEffect {id}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_patch_finds_its_row() {
        for id in ICE_LANCE_RANKS {
            let row = hotfixed(id).unwrap();
            assert_eq!(row.effects[1].base_points, 300.0);
            assert_eq!((row.category, row.category_cooldown_ms), (19, 8000));
        }
        assert_eq!(hotfixed(12579).unwrap().effects[0].base_points, 20.0);
        let primal_bite = hotfixed(1238070).unwrap();
        assert_eq!(primal_bite.effects[0].base_points, 77.0);
        assert_eq!(primal_bite.spell_level, 60);
        assert_eq!(hotfixed(1322605).unwrap().cooldown_ms, 8000);
        assert!(hotfixed(25306).is_none());
        for (spell, id, points) in HUNTER_SHOTS_20261103 {
            let row = hotfixed(spell).unwrap();
            let effect = row.effects.iter().find(|effect| effect.id == id).unwrap();
            assert_eq!(effect.base_points, points, "spell {spell}");
        }
        let wrack = row(Rules::Forever, 1316697);
        assert_eq!(wrack.effects[0].average(60), 86.0);
        assert_eq!(must_find(1316697).effects[0].average(60), 36.0);
        assert!((wrack.effects[0].sp_coef - 0.286).abs() < 1e-6);
        let stance = hotfixed(7376).unwrap();
        let threat = stance
            .effects
            .iter()
            .find(|effect| effect.id == 686475)
            .unwrap();
        assert_eq!(threat.base_points, 80.0);
        for id in SLAM_RANKS {
            assert_eq!(hotfixed(id).unwrap().category_cooldown_ms, 15000);
        }
        assert_eq!(hotfixed(17364).unwrap().cooldown_ms, 6000);
        for id in MUTILATE_HIT_RANKS {
            let weapon = hotfixed(id).unwrap().effect_n(2);
            assert_eq!(
                weapon.effect_type,
                dbcenums::E_WEAPON_PERCENT_DAMAGE,
                "{id}"
            );
            assert_eq!(weapon.base_points, 90.0, "{id}");
            assert_eq!(must_find(id).effect_n(2).base_points, 75.0, "{id}");
            // The flat damage the hit adds is the frozen row's, scaled by the share.
            assert_eq!(
                hotfixed(id).unwrap().effect_n(1).base_points,
                must_find(id).effect_n(1).base_points,
                "{id}"
            );
        }
        let flawless = hotfixed(1310711).unwrap();
        assert_eq!(flawless.effects[0].base_points, -15.0);
        assert_eq!(must_find(1310711).effects[0].base_points, -10.0);
        let starfire = hotfixed(9876).unwrap();
        assert!((starfire.effects[0].base_points - 337.0 * 1.4).abs() < 1e-9);
    }

    #[test]
    fn the_reference_reads_the_frozen_rows() {
        assert!(std::ptr::eq(
            row(Rules::GoReference, 12579),
            must_find(12579)
        ));
        assert!(!std::ptr::eq(row(Rules::Forever, 12579), must_find(12579)));
        assert!(std::ptr::eq(row(Rules::Forever, 25306), must_find(25306)));
    }
}
