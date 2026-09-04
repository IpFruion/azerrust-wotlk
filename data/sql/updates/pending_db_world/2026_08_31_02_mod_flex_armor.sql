-- Flex Armor percentage resistance aura.
-- MiscValue 1 = SPELL_SCHOOL_MASK_NORMAL (physical/armor).
-- Aura type 101 = SPELL_AURA_MOD_RESISTANCE_PCT.
DELETE FROM `spell_dbc` WHERE `ID` = 100105;
INSERT INTO `spell_dbc` (
    `ID`, `Attributes`, `EquippedItemClass`, `Effect_1`, `EffectAura_1`, `ImplicitTargetA_1`,
    `EffectMiscValue_1`, `DurationIndex`, `CastingTimeIndex`, `RangeIndex`, `Name_Lang_enUS`
) VALUES
(100105, 64, -1, 6, 101, 1, 1, 0, 1, 1, 'Flex Armor');
