//! portunex-db — 数据访问层(entities + repositories),基于 sqlx/Postgres。
//! entities 由原库 information_schema 精确生成;repositories 逐步实现业务查询。
pub mod entities;
pub mod repositories;

pub type Db = sqlx::PgPool;
