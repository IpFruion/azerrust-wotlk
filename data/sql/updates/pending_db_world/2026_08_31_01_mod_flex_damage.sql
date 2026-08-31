-- Flex SpellPower outgoing magic-damage percentage aura.
-- MiscValue 126 = SPELL_SCHOOL_MASK_MAGIC (excludes physical/melee).
DELETE FROM `spell_dbc` WHERE `ID` = 100104;
INSERT INTO `spell_dbc` (
    `ID`, `Attributes`, `EquippedItemClass`, `Effect_1`, `EffectAura_1`, `ImplicitTargetA_1`,
    `EffectMiscValue_1`, `DurationIndex`, `CastingTimeIndex`, `RangeIndex`, `Name_Lang_enUS`
) VALUES
(100104, 64, -1, 6, 79, 1, 126, 0, 1, 1, 'Flex SpellPower');
