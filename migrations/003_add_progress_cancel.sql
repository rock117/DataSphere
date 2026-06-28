-- DataSphere 003: task_runs 增加进度与取消支持
-- 增量 migration

USE `datasphere`;

ALTER TABLE `task_runs`
  ADD COLUMN `total`             BIGINT NOT NULL DEFAULT 0 COMMENT '待处理总数' AFTER `duration_ms`,
  ADD COLUMN `processed`         BIGINT NOT NULL DEFAULT 0 COMMENT '已处理数' AFTER `total`,
  ADD COLUMN `cancel_requested`  TINYINT(1) NOT NULL DEFAULT 0 COMMENT '是否请求取消' AFTER `processed`;
