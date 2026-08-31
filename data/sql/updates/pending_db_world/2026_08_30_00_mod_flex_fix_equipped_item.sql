-- Remove the accidental item-class requirement from the already-imported Flex Stat spell.
UPDATE `spell_dbc` SET `Attributes` = 64, `EquippedItemClass` = -1 WHERE `ID` = 100103;
