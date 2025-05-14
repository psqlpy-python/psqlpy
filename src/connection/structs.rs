use deadpool_postgres::Object;
use tokio_postgres::Client;

pub struct PoolConnection {
    pub connection: Object,
}

pub struct SingleConnection {
    pub connection: Client,
}

pub enum PSQLPyConnection {
    PoolConn(PoolConnection),
    SingleConnection(SingleConnection),
}
