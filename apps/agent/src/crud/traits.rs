// internal crates
use crate::crud::errors::CrudErr;

#[allow(async_fn_in_trait)]
pub trait Read<K, V> {
    async fn read(&self, key: K) -> Result<V, CrudErr>;
    async fn read_optional(&self, key: K) -> Result<Option<V>, CrudErr>;
}

#[allow(async_fn_in_trait)]
pub trait Find<K, V> {
    async fn find_where<F>(&self, filter: F) -> Result<Vec<V>, CrudErr>
    where
        F: Fn(&V) -> bool + Send + Sync + 'static;

    async fn find_one_optional<F>(
        &self,
        filter_name: &'static str,
        filter: F,
    ) -> Result<Option<V>, CrudErr>
    where
        F: Fn(&V) -> bool + Send + Sync + 'static;

    async fn find_one<F>(
        &self,
        filter_name: &'static str,
        filter: F,
    ) -> Result<V, CrudErr>
    where
        F: Fn(&V) -> bool + Send + Sync + 'static;
}
