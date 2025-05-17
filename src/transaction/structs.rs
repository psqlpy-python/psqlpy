use deadpool_postgres::Transaction as dp_Transaction;
use tokio_postgres::Transaction as tp_Transaction;

pub enum PSQLPyTransaction {
    PoolTransaction(dp_Transaction<'static>),
    SingleTransaction(tp_Transaction<'static>),
}
