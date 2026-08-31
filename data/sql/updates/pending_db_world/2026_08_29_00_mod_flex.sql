-- Flex Stat primary-attribute aura spell
-- Flex Stat uses SPELL_AURA_MOD_TOTAL_STAT_PERCENTAGE (aura 137),
-- applied via CastCustomSpell with difficulty-scaled amounts.

DELETE FROM `spell_dbc` WHERE `ID` = 100103;
INSERT INTO `spell_dbc` (
    `ID`, `Attributes`, `EquippedItemClass`, `Effect_1`, `EffectAura_1`, `ImplicitTargetA_1`,
    `EffectMiscValue_1`, `DurationIndex`, `CastingTimeIndex`,
    `RangeIndex`, `Name_Lang_enUS`
) VALUES
(100103, 64, -1, 6, 137, 1, -1, 0, 1, 1, 'Flex Stat');
