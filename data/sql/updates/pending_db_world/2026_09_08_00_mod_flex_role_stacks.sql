-- Flex auras stack once per missing player (CumulativeAura = StackAmount).
-- 100106: physical damage percent. 100107: healing done percent.
UPDATE `spell_dbc` SET `CumulativeAura` = 40 WHERE `ID` IN (100103, 100104, 100105);

DELETE FROM `spell_dbc` WHERE `ID` IN (100106, 100107);
INSERT INTO `spell_dbc` (
    `ID`, `Attributes`, `EquippedItemClass`, `Effect_1`, `EffectAura_1`, `ImplicitTargetA_1`,
    `EffectMiscValue_1`, `DurationIndex`, `CastingTimeIndex`, `RangeIndex`, `CumulativeAura`, `Name_Lang_enUS`
) VALUES
(100106, 64, -1, 6, 79, 1, 1, 0, 1, 1, 40, 'Flex Physical'),
(100107, 64, -1, 6, 136, 1, 0, 0, 1, 1, 40, 'Flex Healing');
