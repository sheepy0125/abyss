//! ORM types for the database

#[derive(Queryable, Selectable, Serialize, Clone)]
#[diesel(table_name = crate::schema::cartas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Carta {
    pub id: i32,
    pub user_id: Option<i32>,
    pub parent: Option<i32>,
}

impl Carta {}
