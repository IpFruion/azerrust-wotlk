-- Flex Tank Stat aura spell (separate from base Flex Stat to avoid overwrite bug).
-- Same aura type as Flex Stat (SPELL_AURA_MOD_TOTAL_STAT_PERCENTAGE = 137),
-- but used exclusively for the Tank role's stat bonus.

DELETE FROM `spell_dbc` WHERE `ID` = 100109;
INSERT INTO `spell_dbc` (
    `ID`, `Attributes`, `EquippedItemClass`, `Effect_1`, `EffectAura_1`, `ImplicitTargetA_1`,
    `EffectMiscValue_1`, `DurationIndex`, `CastingTimeIndex`,
    `RangeIndex`, `CumulativeAura`, `Name_Lang_enUS`
) VALUES
(100109, 64, -1, 6, 137, 1, -1, 0, 1, 1, 40, 'Flex Tank Stat');
