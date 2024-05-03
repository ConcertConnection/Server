use datetime;

pub struct ConcertPass<'a> {
    venue: String,
    artist: String,
    date: datetime::ZonedDateTime<'a>,
    standby: bool
}