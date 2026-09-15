-- Flex Solo Tank CC Immunity aura spell
-- SPELL_AURA_MECHANIC_IMMUNITY_MASK (aura 147) with MiscValue=96 grants immunity to:
-- Snare, Root, Fear, Stun, Sleep, Charm, Sapped, Horror, Polymorph, Disoriented, Freeze, Turn.
-- Applied only when exactly 1 player has the Tank role in the instance.
-- MiscValue 96 matches the engine's hardcoded switch for broad CC immunity.

DELETE FROM `spell_dbc` WHERE `ID` = 100108;
INSERT INTO `spell_dbc` (
    `ID`, `Attributes`, `EquippedItemClass`, `Effect_1`, `EffectAura_1`, `ImplicitTargetA_1`,
    `EffectMiscValue_1`, `DurationIndex`, `CastingTimeIndex`,
    `RangeIndex`, `CumulativeAura`, `Name_Lang_enUS`
) VALUES
(100108, 64, -1, 6, 147, 1, 96, 0, 1, 1, 1, 'Flex Solo Tank Immunity');
