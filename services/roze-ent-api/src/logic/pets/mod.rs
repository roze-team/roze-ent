include!("prelude.rs");

mod create_pet;
pub use create_pet::create_pet;
mod get_pet;
pub use get_pet::get_pet;
mod delete_pet;
pub use delete_pet::delete_pet;
