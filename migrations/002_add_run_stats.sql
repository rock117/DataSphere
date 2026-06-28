-- DataSphere 002: task_runs 增加运行统计字段
-- 增量 migration，不影响 001 已有结构

USE `datasphere`;

ALTER TABLE `task_runs`
  ADD COLUMN `success_count` BIGINT NOT NULL DEFAULT 0 COMMENT '成功记录数' AFTER `records_affected`,
  ADD COLUMN `failed_count`  BIGINT NOT NULL DEFAULT 0 COMMENT '失败记录数' AFTER `success_count`,
  ADD COLUMN `duration_ms`   BIGINT NOT NULL DEFAULT 0 COMMENT '运行耗时(毫秒)' AFTER `failed_count`;
