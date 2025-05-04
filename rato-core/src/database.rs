use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait};

/// 简易使用数据库事务
#[async_trait]
pub trait DbPool {
    fn get_db_pool(&self) -> &DatabaseConnection;

    async fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        Ok(self.get_db_pool().begin().await?)
    }

    async fn commit(&self, tx: DatabaseTransaction) -> Result<(), DbErr> {
        Ok(tx.commit().await?)
    }

    async fn rollback(&self, tx: DatabaseTransaction) -> Result<(), DbErr> {
        Ok(tx.rollback().await?)
    }
}