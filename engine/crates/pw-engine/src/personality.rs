//! Bot personalities (GDD 07 and 09; decided on 2026-10-07 by delegation of the user, B2).
//!
//! A personality only configures a civilization without a human player: the Mandate its Governor
//! obeys and the T0 war policy. It is derived from the world seed, so the same world always has the
//! same personalities, and it never touches the Mandate of a human player. The engine keeps enforcing
//! the red lines of the Mandate carried by every Governor command.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    diplomacy::{WarAppetite, WarPolicy},
    governor::{Direction, Mandate, MandatePreset, RedLine, Stance},
    ids::CivId,
    rng::Rng,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotPersonality { Cautious, Mercantile, Expansionist, Belligerent }

impl BotPersonality {
    pub const ALL: [BotPersonality; 4] = [Self::Cautious, Self::Mercantile, Self::Expansionist, Self::Belligerent];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Cautious => "personality.cautious",
            Self::Mercantile => "personality.mercantile",
            Self::Expansionist => "personality.expansionist",
            Self::Belligerent => "personality.belligerent",
        }
    }

    /// The Mandate of the bot's Governor. Only the cautious personality keeps `NoStartWar`; every
    /// personality keeps `NoCedeCity`, and only the belligerent one may break a treaty. Every
    /// direction keeps `relations > 10`, otherwise no diplomatic proposal passes the minimum benefit.
    pub fn mandate(self, version: u32) -> Mandate {
        let mut mandate = match self {
            Self::Cautious => return Mandate::preset(MandatePreset::GrowCautiously, version),
            // The economic direction of the cautious preset: the balanced one let the Governor spend the
            // treasury new cities live on (measured in B2), and the difference is only the war policy.
            Self::Mercantile => Mandate::preset(MandatePreset::GrowCautiously, version),
            Self::Expansionist => Mandate { direction: Direction { security: 25, sustenance: 25, development: 35, relations: 15 }, stance: Stance::Deterrent, ..Mandate::preset(MandatePreset::Balanced, version) },
            Self::Belligerent => Mandate { direction: Direction { security: 40, sustenance: 25, development: 20, relations: 15 }, stance: Stance::Deterrent, ..Mandate::preset(MandatePreset::Balanced, version) },
        };
        mandate.red_lines = match self {
            Self::Belligerent => vec![RedLine::NoCedeCity],
            _ => vec![RedLine::NoBreakTreaty, RedLine::NoCedeCity],
        };
        mandate
    }

    /// T0 war policy: appetite, prudence (no war below a cohesion level, one front at a time) and whether
    /// the bot proposes binding treaties (expansionist and belligerent bots keep their borders open).
    pub const fn war_policy(self) -> WarPolicy {
        match self {
            Self::Cautious => WarPolicy { appetite: WarAppetite::LastResort, min_cohesion: 0, single_front: false, seeks_treaties: true },
            Self::Mercantile => WarPolicy { appetite: WarAppetite::LastResort, min_cohesion: 10, single_front: true, seeks_treaties: true },
            Self::Expansionist => WarPolicy { appetite: WarAppetite::BorderFirst, min_cohesion: 8, single_front: true, seeks_treaties: false },
            Self::Belligerent => WarPolicy { appetite: WarAppetite::Eager, min_cohesion: 5, single_front: true, seeks_treaties: false },
        }
    }

    /// Personalities of the given bot civilizations: a seeded shuffle of the ids (stream
    /// `bot-personality`) dealt the four personalities in turn, so every world with at least four bots
    /// has each personality and the assignment depends only on the seed and the ids.
    pub fn assign(seed: u64, civilizations: impl IntoIterator<Item = CivId>) -> BTreeMap<CivId, BotPersonality> {
        let mut order: Vec<CivId> = civilizations.into_iter().collect();
        order.sort_unstable();
        order.dedup();
        let mut rng = Rng::derive(seed, "bot-personality");
        for index in (1..order.len()).rev() { order.swap(index, rng.below(index as u32 + 1) as usize); }
        order.into_iter().enumerate().map(|(index, civ)| (civ, Self::ALL[index % Self::ALL.len()])).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{diplomacy::ProposalKind, ids::UnitId, world::CommandPayload};

    #[test]
    fn assignment_is_seeded_reproducible_and_balanced() {
        let civs = (0..8).map(CivId);
        let first = BotPersonality::assign(20_261_001, civs.clone());
        assert_eq!(first, BotPersonality::assign(20_261_001, civs.clone().rev()));
        for personality in BotPersonality::ALL { assert_eq!(first.values().filter(|value| **value == personality).count(), 2); }
        assert!((1..20).any(|seed| BotPersonality::assign(seed, civs.clone()) != first), "the seed changes the assignment");
    }

    #[test]
    fn only_cautious_bots_keep_the_war_red_line_and_every_mandate_is_valid() {
        let war = CommandPayload::DeclareWar { target: CivId(1), objective: "objective.recover_territory".into(), cause: 0 };
        let attack = CommandPayload::DeclareAttack { attacker: UnitId(1), target: UnitId(2) };
        let trade = CommandPayload::ProposeDiplomacy { recipient: CivId(1), kind: ProposalKind::Trade };
        for personality in BotPersonality::ALL {
            let mandate = personality.mandate(1);
            assert!(mandate.valid(), "{personality:?}");
            assert!(mandate.red_lines.contains(&RedLine::NoCedeCity));
            assert!(mandate.direction.relations > crate::governor::MINIMUM_BENEFIT);
            assert_eq!(mandate.allows_payload(&war), personality != BotPersonality::Cautious);
            assert_eq!(mandate.allows_payload(&attack), personality != BotPersonality::Cautious);
            assert!(mandate.allows_payload(&trade));
        }
        assert_eq!(BotPersonality::Cautious.mandate(3), Mandate::preset(MandatePreset::GrowCautiously, 3));
    }
}
