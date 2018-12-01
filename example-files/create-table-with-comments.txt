-- phpMyAdmin SQL Dump
-- version 2.6.0-pl2

-- 
-- Database: `test`
-- 

-- --------------------------------------------------------

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