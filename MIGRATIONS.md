# 数据库迁移指南 (sqlx-cli)

本项目使用 [sqlx-cli](https://github.com/launchbadge/sqlx) 管理数据库迁移。迁移文件位于 `migrations/` 目录，命名格式为 `<timestamp>_<description>.sql`。

## 安装 sqlx-cli

```bash
cargo install sqlx-cli --no-default-features --features mysql,rustls
```

> 安装版本需与项目依赖的 sqlx 版本一致（当前 0.9.x）。验证：`sqlx --version`

## 环境变量

sqlx-cli 通过 `DATABASE_URL` 环境变量连接数据库。可直接命令行传入，或放在 `.env` 中（sqlx 会自动读取）：

```bash
# .env
DATABASE_URL=mysql://root:123456@localhost:3306/datasphere
```

以下命令假设 `DATABASE_URL` 已配置，否则需在每条命令前加 `DATABASE_URL=mysql://...`。

## 常用命令

### 执行迁移

跑所有未执行的迁移（幂等，已执行的会跳过）：

```bash
sqlx migrate run
```

### 查看迁移状态

```bash
sqlx migrate info
```

输出示例：
```
20260628000001/installed init
20260628000002/installed add run stats
20260628000003/installed add progress cancel
20260628000004/installed add funds
```

- `installed` = 已执行
- `pending`   = 未执行

### 新建迁移文件

```bash
# 单向迁移（只有 up）
sqlx migrate add <name>

# 可回滚迁移（生成 up + down 两个文件）
sqlx migrate add -r <name>
```

生成文件示例：
```
migrations/20260629000001_add_xxx.sql          # 单向
migrations/20260629000001_add_xxx.up.sql       # 可回滚 - 升级
migrations/20260629000001_add_xxx.down.sql     # 可回滚 - 回滚
```

### 回滚迁移（仅可回滚迁移）

```bash
sqlx migrate revert
```

每次回滚一个版本（按时间倒序）。需要迁移文件有对应的 `.down.sql`。

## 迁移文件规范

1. **命名**：`<14位时间戳>_<英文描述>.sql`，如 `20260629000001_add_fund_nav.sql`
2. **内容**：纯 SQL，每个文件就是一个事务
3. **幂等性**：建议用 `CREATE TABLE IF NOT EXISTS` / `DROP TABLE IF EXISTS`，但 ALTER 类需注意不要重复执行
4. **顺序**：按文件名排序执行，时间戳保证顺序

## 当前迁移清单

| 文件 | 说明 |
|------|------|
| `20260628000001_init.sql` | 初始建表：stocks / klines / tasks / task_runs |
| `20260628000002_add_run_stats.sql` | task_runs 加 success_count / failed_count / duration_ms |
| `20260628000003_add_progress_cancel.sql` | task_runs 加 total / processed / cancel_requested |
| `20260628000004_add_funds.sql` | 新增 funds 基金列表表 |
| `20260629000001_add_fund_holdings.sql` | 新增 fund_holdings 基金成分股表 |

## 从零部署

```bash
# 1. 创建空数据库
mysql -u root -p123456 -e "CREATE DATABASE datasphere DEFAULT CHARACTER SET utf8mb4;"

# 2. 执行所有迁移
sqlx migrate run

# 3. 验证
sqlx migrate info
```

## 追踪表说明

sqlx 在数据库中自动创建 `_sqlx_migrations` 表记录已执行的迁移：

| 字段 | 说明 |
|------|------|
| version | 迁移版本号（时间戳） |
| description | 描述 |
| installed_on | 执行时间 |
| success | 是否成功 |
| checksum | 文件校验和（防篡改） |

> **注意**：不要手动修改此表，否则会导致迁移状态不一致。如需重置，删库后重新 `sqlx migrate run`。

## 常见问题

**Q: 手动改了 migration 文件后执行报错？**
A: sqlx 用 checksum 检测文件篡改。已执行的迁移文件不可修改。如需修改，删库重来或新建一个迁移文件做调整。

**Q: 想跳过某个迁移？**
A: 不建议。如果确实需要，可手动向 `_sqlx_migrations` 插入对应记录（含正确 checksum），但这属于高级操作，需谨慎。

**Q: sqlx migrate run 无输出？**
A: 说明所有迁移都已执行，没有 pending 的。用 `sqlx migrate info` 确认。
