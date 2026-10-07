pub mod db;
pub mod error;
pub mod repository;
pub mod server;
pub mod service;

pub mod prelude {
    pub use crate::db::{connect, migrate, Db};
    pub use crate::error::RepositoryError;
    pub use crate::repository::{CrudRepository, Entity};
    pub use crate::server::{ActixAnyhow, ActixAnyhowError};
    pub use crate::service::CrudService;
}
