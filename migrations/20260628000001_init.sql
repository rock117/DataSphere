-- DataSphere 初始化脚本
-- 数据库：datasphere
-- 字符集：utf8mb4

CREATE DATABASE IF NOT EXISTS `datasphere`
  DEFAULT CHARACTER SET utf8mb4
  DEFAULT COLLATE utf8mb4_unicode_ci;

USE `datasphere`;

-- ----------------------------------------------------------------------------
-- stocks: A股股票列表
-- ----------------------------------------------------------------------------
DROP TABLE IF EXISTS `stocks`;
CREATE TABLE `stocks` (
  `id`         BIGINT       NOT NULL AUTO_INCREMENT,
  `code`       VARCHAR(16)  NOT NULL COMMENT '统一代码，如 600000',
  `symbol`     VARCHAR(16)  NOT NULL COMMENT '带交易所前缀，如 sh600000',
  `name`       VARCHAR(64)  NOT NULL COMMENT '证券名称',
  `market`     VARCHAR(8)   NOT NULL COMMENT '市场：SH/SZ/BJ',
  `exchange`   VARCHAR(16)  NOT NULL COMMENT '交易所全称',
  `list_date`  DATE         NULL COMMENT '上市日期',
  `delist_date` DATE        NULL COMMENT '退市日期',
  `created_at` DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_code` (`code`),
  KEY `idx_market` (`market`),
  KEY `idx_name` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='A股股票列表';

-- ----------------------------------------------------------------------------
-- klines: 日K历史行情
-- ----------------------------------------------------------------------------
DROP TABLE IF EXISTS `klines`;
CREATE TABLE `klines` (
  `id`          BIGINT        NOT NULL AUTO_INCREMENT,
  `code`        VARCHAR(16)   NOT NULL COMMENT '股票代码',
  `date`        DATE          NOT NULL COMMENT '交易日',
  `open`        DECIMAL(12,4) NOT NULL COMMENT '开盘价',
  `close`       DECIMAL(12,4) NOT NULL COMMENT '收盘价',
  `high`        DECIMAL(12,4) NOT NULL COMMENT '最高价',
  `low`         DECIMAL(12,4) NOT NULL COMMENT '最低价',
  `volume`      BIGINT        NOT NULL COMMENT '成交量(股)',
  `amount`      DECIMAL(18,4) NOT NULL COMMENT '成交额(元)',
  `turnover`    DECIMAL(10,4) NULL COMMENT '换手率(%)',
  `pct_change`  DECIMAL(10,4) NULL COMMENT '涨跌幅(%)',
  `created_at`  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_code_date` (`code`, `date`),
  KEY `idx_date` (`date`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='日K历史行情';

-- ----------------------------------------------------------------------------
-- tasks: 任务定义
--   task_type: FetchStockList | FetchKline
--   provider : mock | (后续真实数据源名)
--   cron     : 可空，纯手动任务为 NULL
--   params   : JSON，结构随 task_type 变化
--     FetchStockList: {} 或 {"market":"SH"}
--     FetchKline   : {"start":"2024-01-01","end":"2024-06-01","codes":["600000"]} (codes 空表示全市场)
-- ----------------------------------------------------------------------------
DROP TABLE IF EXISTS `tasks`;
CREATE TABLE `tasks` (
  `id`         BIGINT       NOT NULL AUTO_INCREMENT,
  `name`       VARCHAR(128) NOT NULL COMMENT '任务名称',
  `task_type`  VARCHAR(32)  NOT NULL COMMENT '任务类型',
  `provider`   VARCHAR(32)  NOT NULL DEFAULT 'mock' COMMENT '数据源 provider',
  `cron`       VARCHAR(64)  NULL COMMENT 'cron 表达式，NULL 表示纯手动',
  `params`     JSON         NULL COMMENT '任务参数',
  `enabled`    TINYINT(1)   NOT NULL DEFAULT 1 COMMENT '是否启用',
  `created_at` DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `idx_enabled` (`enabled`),
  KEY `idx_task_type` (`task_type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='任务定义';

-- ----------------------------------------------------------------------------
-- task_runs: 任务执行历史
--   status      : Pending | Running | Success | Failed
--   trigger_type: Cron | Manual
-- ----------------------------------------------------------------------------
DROP TABLE IF EXISTS `task_runs`;
CREATE TABLE `task_runs` (
  `id`               BIGINT       NOT NULL AUTO_INCREMENT,
  `task_id`          BIGINT       NOT NULL,
  `status`           VARCHAR(16)  NOT NULL DEFAULT 'Pending',
  `trigger_type`     VARCHAR(16)  NOT NULL DEFAULT 'Manual',
  `started_at`       DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `finished_at`      DATETIME     NULL,
  `records_affected` BIGINT       NOT NULL DEFAULT 0,
  `error`            TEXT         NULL,
  PRIMARY KEY (`id`),
  KEY `idx_task_started` (`task_id`, `started_at`),
  KEY `idx_status` (`status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='任务执行历史';
