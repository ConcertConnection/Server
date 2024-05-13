extern crate proc_macro;
mod user;
mod claimed_pass;
mod unclaimed_pass;
mod concert;
mod venue;

use scylla::_macro_internal::SerializeRow;
pub use user::User;
pub use claimed_pass::ClaimedPass;
pub trait Nameable {
    fn get_name(&self) -> String;
}

