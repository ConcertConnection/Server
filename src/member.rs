use datetime;
use anyhow;
use crate::concert_pass::ConcertPass;
struct Member<'a> {
    first_name: String,
    last_name: String,
    email: String,
    active: bool,
    paused: bool,
    user_uuid: String,
    sign_up_date: datetime::ZonedDateTime<'a>,
    concert_passes: Vec<ConcertPass<'a>>
}

impl Member {
    pub fn builder(user_name: String) -> Result<Self, anyhow::Error> {

    }
}