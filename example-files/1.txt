-- phpMyAdmin SQL Dump
-- version 2.6.0-pl2

CREATE TABLE `hello` (
  `id` mediumint(8) NOT NULL default '0',
  `forum_id` smallint(5) unsigned NOT NULL default '0',
  `view` tinyint(1) NOT NULL default '0',
  `read` tinyint(1) NOT NULL default '0',
  `post` tinyint(1) NOT NULL default '0',
  `reply` tinyint(1) NOT NULL default '0',
  `edit` tinyint(1) NOT NULL default '0',
  `delete` tinyint(1) NOT NULL default '0',
  `sticky` tinyint(1) NOT NULL default '0',
  `announce` tinyint(1) NOT NULL default '0',
  `vote` tinyint(1) NOT NULL default '0',
  `poll_create` tinyint(1) NOT NULL default '0',
  `attachments` tinyint(1) NOT NULL default '0',
  `mod` tinyint(1) NOT NULL default '0',
  KEY `group_id` (`group_id`),
  KEY `forum_id` (`forum_id`)
) ENGINE=MyISAM DEFAULT CHARSET=latin1;

INSERT INTO `hello` VALUES (1, 34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1);
INSERT INTO `hello` VALUES (2, 35, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1);

SET FOREIGN_KEY_CHECKS=0;

CREATE TABLE IF NOT EXISTS `access_tokens` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int(10) unsigned NOT NULL,
  `access_token` varchar(191) COLLATE utf8mb4_unicode_ci NOT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `access_tokens_user_id_foreign` (`user_id`),
  CONSTRAINT `access_tokens_user_id_foreign` FOREIGN KEY (`user_id`) REFERENCES `site_users` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=332 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO `tokens` (`id`, `user_id`, `access_token`, `created_at`, `updated_at`) VALUES
	(1, 1, ' token=\'gg("")\' ', ''),(2, 1, '', '', '');