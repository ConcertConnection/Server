extern crate proc_macro;
mod user;
mod claimed_pass;
mod unclaimed_pass;
mod concert;
mod venue;

pub use user::User;
pub trait Nameable {
    fn get_name(&self) -> String;
}