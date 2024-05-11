use scylla::macros::FromRow;
#[derive(Debug, FromRow)]
pub struct Venue {
    name: String
}