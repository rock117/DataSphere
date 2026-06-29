-- 行业分类 + 概念板块
-- 1. stocks 加 industry 字段（一对一，每只股票一个行业）
-- 2. concepts 概念表
-- 3. stock_concepts 关联表（多对多）

USE `datasphere`;

-- stocks 加行业字段
ALTER TABLE `stocks`
  ADD COLUMN `industry` VARCHAR(64) NULL COMMENT '行业分类（申万一级）' AFTER `exchange`,
  ADD INDEX `idx_industry` (`industry`);

-- 概念板块
DROP TABLE IF EXISTS `concepts`;
CREATE TABLE `concepts` (
  `id`          BIGINT       NOT NULL AUTO_INCREMENT,
  `name`        VARCHAR(64)  NOT NULL COMMENT '概念名称，如 人工智能、新能源',
  `description` VARCHAR(256) NULL COMMENT '概念描述',
  `created_at`  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_name` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='概念板块';

-- 股票-概念关联表（多对多）
DROP TABLE IF EXISTS `stock_concepts`;
CREATE TABLE `stock_concepts` (
  `id`         BIGINT      NOT NULL AUTO_INCREMENT,
  `stock_code` VARCHAR(16) NOT NULL COMMENT '股票代码',
  `concept_id` BIGINT      NOT NULL COMMENT '概念ID',
  `created_at` DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_stock_concept` (`stock_code`, `concept_id`),
  KEY `idx_concept_id` (`concept_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='股票-概念关联表';
