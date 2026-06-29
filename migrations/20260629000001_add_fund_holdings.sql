-- 基金成分股（持仓明细）
-- 每只基金定期披露的前十大持仓等

USE `datasphere`;

DROP TABLE IF EXISTS `fund_holdings`;
CREATE TABLE `fund_holdings` (
  `id`           BIGINT       NOT NULL AUTO_INCREMENT,
  `fund_code`    VARCHAR(16)  NOT NULL COMMENT '基金代码',
  `stock_code`   VARCHAR(16)  NOT NULL COMMENT '股票代码',
  `stock_name`   VARCHAR(64)  NOT NULL COMMENT '股票名称',
  `report_date`  DATE         NOT NULL COMMENT '报告期（季报披露日）',
  `weight`       DECIMAL(10,4) NOT NULL COMMENT '占净值比例(%)',
  `shares`       BIGINT       NULL COMMENT '持仓股数',
  `market_value` DECIMAL(18,4) NULL COMMENT '持仓市值(元)',
  `rank`         INT          NULL COMMENT '持仓排名（第几大）',
  `created_at`   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_fund_stock_date` (`fund_code`, `stock_code`, `report_date`),
  KEY `idx_fund_code` (`fund_code`),
  KEY `idx_stock_code` (`stock_code`),
  KEY `idx_report_date` (`report_date`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='基金成分股（持仓明细）';
