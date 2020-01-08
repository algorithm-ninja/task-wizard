use super::*;

use crate::api::root::ApiContext;
use diesel::{QueryResult, RunQueryDsl, SqliteConnection};
use schema::announcements;

#[derive(Clone, Debug, Queryable)]
pub struct Announcement {
    id: i32,
    text: String,
}

#[juniper_ext::graphql(Context = ApiContext)]
impl Announcement {
    /// Text of the announcement
    fn text(&self) -> &String {
        &self.text
    }
}

/// Get announcements from the database
pub fn query_all(conn: &SqliteConnection) -> QueryResult<Vec<Announcement>> {
    announcements::table.load(conn)
}
