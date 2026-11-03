//! Shank, from the development notes of 2 November 2026 (Forever's rules): a baseline strike
//! with the off hand weapon for half its weapon damage, which generates Energy. It cannot miss,
//! so it rolls like each hand of Mutilate: blocked or critical, never missed, dodged or parried.
//! It gives no combo point.

use crate::core::fight::{melee::PhysicalOutcome, Agent, Fight, Side, SpellId};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Shank {
    pub(crate) energy_gain: f64,
}

impl Shank {
    /// Shank's effect; Stealth has already broken. The spell's damage multiplier holds the
    /// weapon share, and the off hand's damage penalty is in the weapon roll.
    pub(crate) fn apply<A: Agent>(&self, fight: &mut Fight<A>, spell: SpellId, target: Side) {
        let attack_power = fight.melee_attack_power();
        let damage = fight.oh_weapon_damage(attack_power);
        let result = fight.calc_physical_damage(
            spell,
            target,
            damage,
            PhysicalOutcome::MeleeSpecialBlockAndCrit { count: true },
        );
        fight.deal_damage(spell, result, false);
        let (energy_metrics, _) = fight.spells[spell]
            .energy_metrics
            .expect("Shank costs energy");
        fight.add_energy(self.energy_gain, energy_metrics);
    }
}
