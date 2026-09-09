-- Allow every player to declare their Flex role with ".flex role".
DELETE FROM `rbac_permissions` WHERE `id` = 946;
INSERT INTO `rbac_permissions` (`id`, `name`) VALUES
(946, 'Command: flex role');

DELETE FROM `rbac_linked_permissions` WHERE `linkedId` = 946;
INSERT INTO `rbac_linked_permissions` (`id`, `linkedId`) VALUES
(199, 946);
