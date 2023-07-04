use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Query<T> {
    type Filter;
    async fn query(&self, filter: &Self::Filter) -> Result<Vec<T>>;
}

#[async_trait]
pub trait Insert<T> {
    async fn insert(&self, item: T) -> Result<T>;
}

#[async_trait]
pub trait Update<T> {
    async fn update(&self, item: T) -> Result<T>;
}

#[async_trait]
pub trait Retrieve<T> {
    type Key;
    async fn retrieve(&self, key: Self::Key) -> Result<T>;
}


#[async_trait]
pub trait Delete<T> {
    async fn delete(&self, item: T) -> Result<()>;
}

