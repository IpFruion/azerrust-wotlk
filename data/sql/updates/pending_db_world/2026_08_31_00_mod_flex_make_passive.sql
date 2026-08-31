-- Make Flex Stat permanent by marking the aura spell as passive.
UPDATE `spell_dbc` SET `Attributes` = 64, `EquippedItemClass` = -1 WHERE `ID` = 100103;
