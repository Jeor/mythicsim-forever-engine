//! Soul Siphon (17804) under Forever's rules: Drain Life and Wrack deal 12% more per each of the
//! warlock's other Affliction effects active on the target, up to 36% (the talent's text in
//! client 1.60.1.70755). The talent is a dummy aura, so the server counts the effects; Forever's
//! damage over time reads the caster's current modifiers at each tick, so each tick counts the
//! effects up at that moment. Go counts every registered Affliction aura on the target, active
//! or not (`AfflictionCount`), which makes the bonus the full 36% for every tick of every build;
//! the Go reference keeps that.

use crate::{
    classes::warlock::agent::WarlockAgent,
    contracts::prepared_v2::SoulSiphonLive,
    core::fight::{AuraRef, Fight, Side},
};

use crate::classes::warlock::spells::find_spell;

#[derive(Clone, Debug)]
pub(crate) struct SoulSiphon {
    per_effect: f64,
    max_effects: u32,
    /// The auras of the warlock's Affliction damage over time effects, on the first target.
    dot_auras: Vec<AuraRef>,
}

pub(crate) fn bind(
    fight: &Fight<WarlockAgent>,
    live: &SoulSiphonLive,
) -> Result<SoulSiphon, String> {
    let mut dot_auras = Vec::new();
    for &spell_id in &live.dot_spells {
        let spell = find_spell(fight, spell_id)?;
        let dot = fight
            .spell_dot(spell)
            .ok_or_else(|| format!("Soul Siphon's spell {spell_id} has no dot"))?;
        dot_auras.push(fight.dots[dot].aura);
    }
    Ok(SoulSiphon {
        per_effect: live.per_effect,
        max_effects: live.max_effects,
        dot_auras,
    })
}

impl SoulSiphon {
    /// The warlock's Affliction effects active on `target` now: its damage over time effects
    /// and its own curse (Curse of the Elements or Curse of Recklessness, both Affliction spells
    /// in the client's skill line 355). The raid's curse is another aura and does not count.
    pub(crate) fn effects(&self, fight: &Fight<WarlockAgent>, target: Side) -> u32 {
        let active = |aura: AuraRef| fight.aura(fight.aura_on(aura, target)).active;
        let dots = self.dot_auras.iter().filter(|&&aura| active(aura)).count();
        let curses = [
            fight.agent.curse_of_the_elements_aura(),
            fight.agent.curse_of_recklessness_aura(),
        ]
        .into_iter()
        .flatten()
        .filter(|&aura| active(aura))
        .count();
        ((dots + curses) as u32).min(self.max_effects)
    }

    /// The multiplier on a tick at `target`, as Go's `1 + x*y`.
    pub(crate) fn multiplier(&self, fight: &Fight<WarlockAgent>, target: Side) -> f64 {
        self.per_effect
            .mul_add(f64::from(self.effects(fight, target)), 1.0)
    }
}
