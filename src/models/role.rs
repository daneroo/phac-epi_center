use std::fmt::Debug;

use chrono::{prelude::*};
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable, ExpressionMethods};
use diesel::{RunQueryDsl, QueryDsl};
use uuid::Uuid;
use async_graphql::*;

use crate::graphql::graphql_translate;
use crate::config_variables::DATE_FORMAT;

use crate::schema::*;
use crate::database::connection;

use super::{Person, Team};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = roles)]
/// Intermediary data structure between Person and team
/// Referenced by Person
pub struct Role {
    pub id: Uuid,
    pub person_id: Uuid,
    pub team_id: Uuid,
    pub title_en: String,
    pub title_fr: String,
    pub effort: f64,
    pub active: bool,
    pub start_datestamp: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[Object]
impl Role {
    pub async fn person(&self) -> Result<Person> {
        Person::get_by_id(&self.person_id)
    }

    pub async fn team(&self) -> Result<Team> {
        Team::get_by_id(&self.team_id)
    }

    pub async fn english_title(&self) -> Result<String> {
        Ok(self.title_en.to_owned())
    }

    pub async fn french_title(&self) -> Result<String> {
        Ok(self.title_fr.to_owned())
    }

    pub async fn effort(&self) -> Result<f64> {
        Ok(self.effort)
    }

    pub async fn active(&self) -> Result<String> {
        if self.active {
            Ok("Active".to_string())
        } else {
            Ok("INACTIVE".to_string())
        }
    }

    pub async fn start_date(&self) -> Result<String> {
        Ok(self.start_datestamp.format(DATE_FORMAT).to_string())
    }

    pub async fn end_date(&self) -> Result<String> {
        match self.end_date {
            Some(d) => Ok(d.format(DATE_FORMAT).to_string()),
            None => Ok("Still Active".to_string())
        }
    }

    pub async fn created_at(&self) -> Result<String> {
        Ok(self.created_at.format(DATE_FORMAT).to_string())
    }

    pub async fn updated_at(&self) -> Result<String> {
        Ok(self.updated_at.format(DATE_FORMAT).to_string())
    }
}


// Non Graphql
impl Role {
    pub fn create(role: &NewRole) -> Result<Role> {
        let mut conn = connection()?;

        let res = diesel::insert_into(roles::table)
        .values(role)
        .get_result(&mut conn);
        
        graphql_translate(res)
    }
    
    pub fn get_or_create(role: &NewRole) -> Result<Role> {
        let mut conn = connection()?;

        let res = roles::table
        .filter(roles::person_id.eq(&role.person_id))
        .distinct()
        .first(&mut conn);
        
        let role = match res {
            Ok(p) => p,
            Err(e) => {
                // Role not found
                println!("{:?}", e);
                let p = Role::create(role).expect("Unable to create role");
                p
            }
        };
        Ok(role)
    }

    pub fn find_all() -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let roles = roles::table.load::<Role>(&mut conn)?;
        Ok(roles)
    }

    pub fn get_by_id(id: Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let role = roles::table.filter(roles::id.eq(id)).first(&mut conn)?;
        Ok(role)
    }

    pub fn get_by_team_id(id: Uuid) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let res = roles::table
            .filter(roles::team_id.eq(id))
            .load::<Role>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_person_id(id: Uuid) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let res = roles::table
            .filter(roles::person_id.eq(id))
            .load::<Role>(&mut conn)?;

        Ok(res)
    }
    
    pub fn update(&self) -> Result<Self> {
        let mut conn = connection()?;

        let res = diesel::update(roles::table)
        .filter(roles::id.eq(&self.id))
        .set(self)
        .get_result(&mut conn)?;
        
        Ok(res)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, SimpleObject)]
#[diesel(table_name = roles)]
pub struct NewRole {
    pub person_id: Uuid,
    pub team_id: Uuid,
    pub title_en: String,
    pub title_fr: String,
    pub effort: f64,
    pub active: bool,
    pub start_datestamp: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
}

impl NewRole {

    pub fn new(
        person_id: Uuid,
        team_id: Uuid,
        title_en: String,
        title_fr: String,
        effort: f64,
        active: bool,
        start_datestamp: NaiveDateTime,
        end_date: Option<NaiveDateTime>,
    ) -> Self {
        NewRole {
            person_id,
            team_id,
            title_en,
            title_fr,
            effort,
            active,
            start_datestamp,
            end_date,
        }
    }
}
