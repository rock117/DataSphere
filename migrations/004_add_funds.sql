-- DataSphere 004: funds 基金列表
-- 增量 migration

USE `datasphere`;

DROP TABLE IF EXISTS `funds`;
CREATE TABLE `funds` (
  `id`           BIGINT       NOT NULL AUTO_INCREMENT,
  `code`         VARCHAR(16)  NOT NULL COMMENT '基金代码，如 000001',
  `name`         VARCHAR(128) NOT NULL COMMENT '基金名称',
  `fund_type`    VARCHAR(32)  NOT NULL COMMENT '基金类型：股票型/混合型/债券型/货币型/QDII/指数型/FOF等',
  `management`   VARCHAR(128) NOT NULL COMMENT '基金管理人',
  `custodian`    VARCHAR(128) NOT NULL COMMENT '基金托管人',
  `setup_date`   DATE         NULL COMMENT '成立日期',
  `latest_scale` DECIMAL(18,4) NULL COMMENT '最新规模(亿元)',
  `created_at`   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_code` (`code`),
  KEY `idx_fund_type` (`fund_type`),
  KEY `idx_management` (`management`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='基金列表';
