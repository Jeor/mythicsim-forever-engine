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

/// Blizzard's development notes of 2 November 2026, Shaman: Lightning Bolt's base damage on every
/// rank (each "roughly 78%" of the Classic Era rank, except the maximum rank, which the notes
/// "increased by 80%": exactly 1.8 times its old damage at level 60, 200 to 360, so both ends of
/// its roll scale alike), Chain Lightning's primary target damage (145% of the comparable rank
/// Lightning Bolt's level 60 damage) and the shocks "reverted to match Classic Era values", as (spell, SpellEffect, base
/// points). No client row states them yet, so the numbers are derived from the notes' ratios and
/// Wowhead's Classic Era tooltips, the way wowsims/forever #821 (a preview on a patched spell
/// store) derives them; the tests rebuild each from its source. Lightning Overload's copies deal
/// half their rank's damage, as before (the second half of the table).
pub(crate) const SHAMAN_BASE_POINTS_20261102: [(i32, i32, f64); 50] = [
    // Lightning Bolt ranks 1 to 10: 15, 27, 45, 61, 82, 111, 147, 162, 178, 196 before. Rank 10's
    // 355.5 is 360 at level 60, 1.8 times the 200 it was.
    (403, 678918, 12.0),
    (529, 679369, 24.0),
    (548, 679030, 41.0),
    (915, 679313, 73.0),
    (943, 679378, 109.0),
    (6041, 684438, 149.0),
    (10391, 688309, 195.0),
    (10392, 688310, 241.0),
    (15207, 691075, 295.0),
    (15208, 691780, 355.5),
    // Chain Lightning ranks 1 to 4: 88, 100, 112, 123 before.
    (421, 679158, 219.0),
    (930, 679273, 286.0),
    (2860, 681324, 431.5),
    (10605, 688920, 519.0),
    // Earth Shock ranks 1 to 7: 18, 33, 49, 81, 132, 205, 301 before.
    (8042, 686790, 20.5),
    (8044, 685548, 36.5),
    (8045, 686227, 67.0),
    (8046, 686672, 130.0),
    (10412, 687897, 242.0),
    (10413, 688149, 383.0),
    (10414, 688559, 531.0),
    // Flame Shock ranks 1 to 6, the direct hit: 20, 33, 43, 82, 128, 166 before.
    (8050, 686823, 25.0),
    (8052, 685531, 51.0),
    (8053, 686555, 95.0),
    (10447, 688330, 164.0),
    (10448, 688934, 245.0),
    (29228, 704945, 292.0),
    // Flame Shock ranks 2 to 6, a tick: 8, 14, 21, 34, 44 before (rank 1's 7 stays).
    (8052, 685532, 12.0),
    (8053, 686556, 24.0),
    (10447, 688331, 42.0),
    (10448, 688935, 64.0),
    (29228, 704946, 80.0),
    // Frost Shock ranks 1 to 4: 66, 124, 188, 283 before.
    (8056, 686558, 98.0),
    (8058, 687459, 222.5),
    (10472, 688813, 355.5),
    (10473, 688022, 506.0),
    // Lightning Overload's Lightning Bolt, ranks 1 to 10: 7, 14.5, 22.5, 28, 36, 56, 74, 81, 89,
    // 98 before.
    (408439, 1072563, 6.0),
    (408440, 1072564, 12.0),
    (408441, 1072565, 20.5),
    (408442, 1072567, 36.5),
    (408443, 1072569, 54.5),
    (408472, 1072625, 74.5),
    (408473, 1072626, 97.5),
    (408474, 1072627, 120.5),
    (408475, 1072628, 147.5),
    (408477, 1072630, 178.0),
    // Lightning Overload's Chain Lightning, ranks 1 to 4: 44, 50, 56, 62 before.
    (408479, 1072633, 107.5),
    (408481, 1072635, 140.5),
    (408482, 1072636, 213.0),
    (408484, 1072638, 256.5),
];

/// Chain Lightning's spell power coefficient under the notes of 2 November: 2.5 sec of cast at
/// the standard 3.5 sec rate (0.714, was 0.571 and 0.517), and half of it on the Lightning
/// Overload copy, as before. (spell, SpellEffect, coefficient).
pub(crate) const CHAIN_LIGHTNING_COEFFICIENTS_20261102: [(i32, i32, f64); 8] = [
    (421, 679158, 0.714),
    (930, 679273, 0.714),
    (2860, 681324, 0.714),
    (10605, 688920, 0.714),
    (408479, 1072633, 0.357),
    (408481, 1072635, 0.357),
    (408482, 1072636, 0.357),
    (408484, 1072638, 0.357),
];

/// Chain Lightning's mana cost ("approximately 225%" of the
/// comparable Lightning Bolt rank's) under the notes of 2 November, as (spell, mana). The
/// comparable Lightning Bolt ranks are 6, 7, 9 and 10, the nearest by the level each is learnt.
pub(crate) const CHAIN_LIGHTNING_COSTS_20261102: [(i32, i32); 4] =
    [(421, 248), (930, 304), (2860, 428), (10605, 495)];

/// Chain Lightning's cast time under the notes of 2 November.
pub(crate) const CHAIN_LIGHTNING_CAST_MS_20261102: i32 = 2500;
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

/// Riptide's three ranks and their mana cost under the notes of 2 November: "reduced by about
/// 40%" (245, 300 and 385 before, each times 0.6 rounded down), as (spell, mana). The notes also
/// give it a 12 sec cooldown (was 6) and drop its initial heal, leaving the heal over time.
pub(crate) const RIPTIDE_COSTS_20261102: [(i32, i32); 3] =
    [(408521, 147), (1239242, 180), (1239243, 231)];

/// Riptide's cooldown under the notes of 2 November.
pub(crate) const RIPTIDE_COOLDOWN_MS_20261102: i32 = 12000;

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

    // Shaman (notes of 2 November): Lightning Bolt, Chain Lightning, the shocks and Lightning
    // Overload's copies, then Chain Lightning's cast time, coefficient and mana.
    for (spell, id, points) in SHAMAN_BASE_POINTS_20261102 {
        patch(spell, &|row| effect(row, id).base_points = points);
    }
    for (spell, id, coefficient) in CHAIN_LIGHTNING_COEFFICIENTS_20261102 {
        patch(spell, &|row| effect(row, id).sp_coef = coefficient);
    }
    for (spell, cost) in CHAIN_LIGHTNING_COSTS_20261102 {
        patch(spell, &|row| {
            row.cast_time_ms = CHAIN_LIGHTNING_CAST_MS_20261102;
            row.powers[0].cost = cost;
        });
    }

    // Shaman (notes of 2 November): Riptide costs about 40% less mana, has a 12 sec cooldown (was
    // 6) and no initial heal, only the heal over time. The direct heal (effect 1, a heal) turns
    // into an inert dummy so the effects keep their places: the Chain Heal bonus reads the third.
    for (id, cost) in RIPTIDE_COSTS_20261102 {
        patch(id, &|row| {
            row.powers[0].cost = cost;
            row.category_cooldown_ms = RIPTIDE_COOLDOWN_MS_20261102;
            row.effects[0].effect_type = dbcenums::E_DUMMY;
        });
    }

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
        for (id, mana) in RIPTIDE_COSTS_20261102 {
            let riptide = hotfixed(id).unwrap();
            assert_eq!(riptide.powers[0].cost, mana, "{id}");
            assert_eq!(riptide.category_cooldown_ms, 12000, "{id}");
            // The heal effect is gone and the heal over time and the Chain Heal bonus keep their
            // places.
            assert!(riptide.heal_effect().is_nil(), "{id}");
            assert_eq!(riptide.effects[1].aura, dbcenums::A_PERIODIC_HEAL, "{id}");
            assert_eq!(riptide.effects[2].base_points, 25.0, "{id}");
            assert_eq!(must_find(id).category_cooldown_ms, 6000, "{id}");
            assert!(!must_find(id).heal_effect().is_nil(), "{id}");
        }
        assert_eq!(must_find(1239243).powers[0].cost, 385);
        let starfire = hotfixed(9876).unwrap();
        assert!((starfire.effects[0].base_points - 337.0 * 1.4).abs() < 1e-9);
    }

    /// The Shaman rows rebuild from the notes of 2 November 2026 and their sources, so a number
    /// that drifts from its derivation fails here.
    #[test]
    fn shaman_rows_rebuild_from_their_sources() {
        let base = |spell: i32, effect_id: i32| -> f64 {
            row(Rules::Forever, spell)
                .effects
                .iter()
                .find(|effect| effect.id == effect_id)
                .unwrap()
                .base_points
        };
        let table = |spell: i32, effect_id: i32| -> f64 {
            SHAMAN_BASE_POINTS_20261102
                .iter()
                .find(|row| row.0 == spell && row.1 == effect_id)
                .unwrap()
                .2
        };
        for (spell, effect_id, points) in SHAMAN_BASE_POINTS_20261102 {
            assert_eq!(base(spell, effect_id), points, "spell {spell}");
            assert_ne!(
                must_find(spell)
                    .effects
                    .iter()
                    .find(|effect| effect.id == effect_id)
                    .unwrap()
                    .base_points,
                points,
                "spell {spell} already states it"
            );
        }

        // The level 60 damage of a row with the base points a table states.
        let at_60 = |spell: i32, effect_id: i32| -> f64 {
            let effect = must_find(spell)
                .effects
                .iter()
                .find(|effect| effect.id == effect_id)
                .unwrap();
            let mut patched = effect.clone();
            patched.base_points = table(spell, effect_id);
            patched.average(60)
        };
        let frozen_at_60 = |spell: i32| must_find(spell).effects[0].average(60);

        // Lightning Bolt: "roughly 78%" of the Classic Era rank, whose damage at level 60
        // Wowhead's Classic tooltips average (15 to 17, 28 to 33, ... 428 to 477), on ranks 1 to
        // 9 ...
        let classic = [
            16.0, 30.5, 52.5, 94.0, 140.0, 190.5, 249.5, 308.5, 378.5, 452.5,
        ];
        let bolts = [
            (403, 678918),
            (529, 679369),
            (548, 679030),
            (915, 679313),
            (943, 679378),
            (6041, 684438),
            (10391, 688309),
            (10392, 688310),
            (15207, 691075),
            (15208, 691780),
        ];
        for (rank, (spell, effect_id)) in bolts.into_iter().enumerate().take(9) {
            let expected = (classic[rank] * 0.78_f64).round();
            assert_eq!(
                table(spell, effect_id),
                expected,
                "Lightning Bolt rank {}",
                rank + 1
            );
        }
        // ... and the maximum rank is 1.8 times what it was at level 60 (200 to 360), on both
        // ends of its roll: the row's variance is a share of the average, so the minimum and
        // maximum scale with it. The Classic Era's 78% of 452.5 is 353 base, within 1% of it.
        let old = frozen_at_60(15208);
        let new = at_60(15208, 691780);
        assert_eq!((old, new), (200.0, 360.0));
        let variance = row(Rules::Forever, 15208).effects[0].variance;
        assert_eq!(variance, must_find(15208).effects[0].variance);
        for end in [-0.5, 0.5] {
            let scale = 1.0 + end * variance;
            assert!((new * scale - 1.8 * old * scale).abs() < 1e-9);
        }
        assert!((new - 0.78 * classic[9]).abs() / new < 0.03);

        // Chain Lightning: 145% of the comparable Lightning Bolt rank's level 60 damage, ranks
        // 6, 7, 9 and 10, the nearest by the level each is learnt (32, 40, 48 and 56).
        let chain = [
            (421, 679158, 5),
            (930, 679273, 6),
            (2860, 681324, 8),
            (10605, 688920, 9),
        ];
        for (spell, effect_id, bolt) in chain {
            let expected = (at_60(bolts[bolt].0, bolts[bolt].1) * 1.45).round();
            assert_eq!(at_60(spell, effect_id), expected, "Chain Lightning {spell}");
        }
        // Its mana is about 225% of the same ranks' (110, 135, 190 and 220 mana).
        for ((spell, mana), bolt) in CHAIN_LIGHTNING_COSTS_20261102.into_iter().zip([5, 6, 8, 9]) {
            let bolt_cost = f64::from(must_find(bolts[bolt].0).powers[0].cost);
            assert_eq!(f64::from(mana), (bolt_cost * 2.25).round(), "spell {spell}");
            let chain = row(Rules::Forever, spell);
            assert_eq!(chain.powers[0].cost, mana);
            assert_eq!(chain.cast_time_ms, 2500);
            assert_eq!(must_find(spell).cast_time_ms, 2000);
        }
        // 2.5 sec of cast at the standard 3.5 sec rate.
        for (_, _, coefficient) in CHAIN_LIGHTNING_COEFFICIENTS_20261102.iter().take(4) {
            assert!((coefficient - 2.5 / 3.5).abs() < 1e-3);
        }

        // Lightning Overload's copies deal half their rank's damage and coefficient.
        let overloads = [
            (403, 408439, 1072563),
            (529, 408440, 1072564),
            (548, 408441, 1072565),
            (915, 408442, 1072567),
            (943, 408443, 1072569),
            (6041, 408472, 1072625),
            (10391, 408473, 1072626),
            (10392, 408474, 1072627),
            (15207, 408475, 1072628),
            (15208, 408477, 1072630),
            (421, 408479, 1072633),
            (930, 408481, 1072635),
            (2860, 408482, 1072636),
            (10605, 408484, 1072638),
        ];
        for (parent, copy, effect_id) in overloads {
            let parent_effect = row(Rules::Forever, parent).effects[0].id;
            let half = at_60(parent, parent_effect) / 2.0;
            assert!(
                (at_60(copy, effect_id) - half).abs() <= 1.0,
                "overload of {parent}: {} against {half}",
                at_60(copy, effect_id)
            );
            let parent_coefficient = row(Rules::Forever, parent).effects[0].sp_coef;
            let copy_coefficient = row(Rules::Forever, copy).effects[0].sp_coef;
            assert!(
                (copy_coefficient - parent_coefficient / 2.0).abs() < 1e-3,
                "{parent}"
            );
        }

        // The shocks at Classic Era values, Wowhead's Classic tooltips at level 60.
        let earth = [20.5, 36.5, 67.0, 130.0, 242.0, 383.0, 531.0];
        for ((spell, effect_id), expected) in [
            (8042, 686790),
            (8044, 685548),
            (8045, 686227),
            (8046, 686672),
            (10412, 687897),
            (10413, 688149),
            (10414, 688559),
        ]
        .into_iter()
        .zip(earth)
        {
            assert_eq!(table(spell, effect_id), expected, "Earth Shock {spell}");
        }
        // 292 and 320 over 12 sec, which is 80 on each of four ticks.
        assert_eq!(table(29228, 704945), 292.0);
        assert_eq!(table(29228, 704946) * 4.0, 320.0);
        assert_eq!(table(10473, 688022), 506.0);
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
