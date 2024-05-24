use std::any::Any;
use scylla::frame::value::CqlTimestamp;
use uuid::Uuid;
use crate::routes::Concert as ConcertWeb;
use scylla::{FromRow, SerializeRow};
use scylla::_macro_internal::{RowSerializationContext, SerializeCql, SerializeRow};
use scylla::serialize::{RowWriter, SerializationError};
use struct_iterable::Iterable;
use crate::database::{Nameable, SelectQueries, SelectQueryChange};

#[derive(SerializeRow, FromRow, Debug, PartialEq)]
pub struct Concert {
    pub(crate) concert_uuid: Uuid,
    venue_uuid: Uuid,
    concert_date: CqlTimestamp,
    artist_name: String,
    pub(crate) artist_image: String,
    pub(crate) artist_bio: String,
    pub(crate) artist_video: String,
    standby_passes: i32,
    secured_passes: i32
}

#[derive(Debug, Iterable)]
pub struct ConcertConditions {
    concert_uuid: Option<Uuid>,
    concert_date: Option<CqlTimestamp>
}

impl ConcertConditions {
    pub fn new(
        concert_uuid: Option<Uuid>,
        concert_date: Option<CqlTimestamp>
    ) -> Self {
        ConcertConditions {
            concert_uuid,
            concert_date
        }
    }

}

impl SelectQueryChange for ConcertConditions {
    fn get_enum(&self) -> SelectQueries {
        let inner: ConcertSelectQueries = self.into();
        SelectQueries::Concert(inner)
    }
}


impl SerializeRow for ConcertConditions {
    fn serialize(
        &self,
        ctx: &RowSerializationContext<'_>,
        writer: &mut RowWriter<'_>
    ) -> Result<(), SerializationError> {

        if self.concert_uuid.is_some() {
            let cell_writer = writer.make_cell_writer();
            self.concert_uuid.serialize(&cell_writer)
        }
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.field1.is_none() && self.field2.is_none() && self.field3.is_none()
    }
}

impl ConcertSelectQueries {
    fn iter(&self) -> impl Iterator<Item = &'static str> {
        vec![
            self.concert_uuid.then(|| "concert_uuid"),
            self.concert_date.then(|| "concert_date"),
        ]
            .into_iter()
            .flatten()
    }
}

impl Nameable for Concert {
    fn get_name(&self) -> String {
        "concert".to_string()
    }
}

impl Concert {
    pub fn new(concert_uuid: Uuid,
               venue_uuid: Uuid,
               concert_date: CqlTimestamp,
               artist_name: String,
               artist_image: String,
               artist_bio: String,
               artist_video: String,
               standby_passes: i32,
               secured_passes: i32) -> Self {
        Concert {
            concert_uuid,
            venue_uuid,
            concert_date,
            artist_name,
            artist_image,
            artist_bio,
            artist_video,
            standby_passes,
            secured_passes
        }

    }
    pub fn get_id(&self) -> &Uuid {
        &self.concert_uuid
    }
    pub fn get_img_path(&self) -> &String {
        &self.artist_image
    }

    pub fn get_video_path(&self) -> &String {
        &self.artist_video
    }

    pub fn increment_pass(&mut self, kind: String) -> Result<(), String> {
        match kind.to_lowercase().as_str() {
            "secured" => Ok(self.secured_passes += 1),
            "standby" => Ok(self.standby_passes += 1),
            _ => Err(format!("{kind} is not supported"))
        }
    }
    pub fn decrement_pass(&mut self, kind:  String) -> Result<(), String> {
        match kind.to_lowercase().as_str() {
            "secured" => Ok(self.secured_passes -= 1),
            "standby" => Ok(self.standby_passes -= 1),
            _ => Err(format!("{kind} is not supported"))
        }
    }
}


impl From<ConcertWeb> for Concert {

    fn from(value: ConcertWeb) -> Self {
        let concert_uuid = value.concert_uuid.unwrap_or_else(Uuid::new_v4);

        let artist_image = value.artist_image.unwrap_or_else(|| "None".to_string());

        let artist_video = value.artist_video.unwrap_or_else(|| "None".to_string());

        let artist_bio = value.artist_bio.unwrap_or_else(|| "".to_string());
        Concert {
            concert_uuid,
            venue_uuid: value.venue_uuid,
            concert_date: CqlTimestamp(value.concert_date.timestamp_millis()),
            artist_name: value.artist_name,
            artist_image,
            artist_video,
            artist_bio,
            secured_passes: value.secured_passes,
            standby_passes: value.standby_passes,

        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Iterable, Clone, Copy)]
pub struct ConcertSelectQueries {
    pub concert_uuid: bool,
    pub concert_date: bool
}


impl From<&ConcertConditions> for ConcertSelectQueries {
    fn from(value: &ConcertConditions) -> Self {
        Self {
            concert_uuid: value.concert_uuid.is_some(),
            concert_date: value.concert_date.is_some()
        }
    }
}


