//! Faerie Fire (9907), from Go sim/druid/faerie_fire.go: a magic hit roll that activates the
//! target's Faerie Fire on a landed outcome. The debuff's armor reduction, an exclusive effect,
//! applies while it is up when it holds its category alone, and never when another aura holds
//! the category for the whole fight; the gate admits only those two readings. Go keeps one
//! debuff per target (`FaerieFireAuras` by unit index), so a cast at another target puts that
//! target's copy up and changes that target's armor.

use crate::core::fight::{Agent, AuraRef, Fight, Outcome, Side, SpellId};

#[derive(Clone, Copy, Debug)]
pub(crate) struct FaerieFire {
    pub(crate) aura: AuraRef,
    /// The armor change while the debuff holds its category, or none.
    pub(crate) armor: Option<f64>,
}

impl FaerieFire {
    /// The exclusive effect's gain on the target that holds this copy, before the aura's.
    pub(crate) fn on_exclusive_gain<A: Agent>(&self, fight: &mut Fight<A>, aura: AuraRef) {
        if let Some(armor) = self.armor {
            fight.add_target_armor(aura.side, armor);
        }
    }

    /// The exclusive effect's expiry on the target that holds this copy.
    pub(crate) fn on_expire<A: Agent>(&self, fight: &mut Fight<A>, aura: AuraRef) {
        if let Some(armor) = self.armor {
            fight.add_target_armor(aura.side, -armor);
        }
    }

    pub(crate) fn apply<A: Agent>(&self, fight: &mut Fight<A>, spell: SpellId, target: Side) {
        let result = fight.calc_outcome(spell, target, Outcome::MagicHit);
        fight.deal_damage(spell, result, false);
        if result.landed() {
            let debuff = fight.aura_on(self.aura, target);
            fight.activate_aura(debuff);
        }
    }
}
