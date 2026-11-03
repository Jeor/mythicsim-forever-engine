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
