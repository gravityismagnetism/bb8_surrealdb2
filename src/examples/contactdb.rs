use super::*;
use crate::errors::DatabaseConnectionErrors;

use std::error::Error;
use surrealdb::err::Error as DbError;
use surrealdb::Error as SurrealError;

#[allow(dead_code)]
static TABLE: &str = "Contacts";

#[allow(dead_code)]
pub struct ContactRepository;

#[allow(dead_code)]
impl ContactRepository {
    /// List all contacts in the database.
    pub async fn get_all(&self) -> Result<Vec<Contact>, Box<dyn Error>> {
        match get_pool().await {
            // Get a connection to the database from the pool each time to ensure that any connection
            // is still valid and operations can be performed in parallel.
            Ok(pool) => match pool.get().await {
                // Use a match case to ensure that the error is useful instead of panicking.
                Ok(connection) => Ok(connection.select(TABLE).await?),
                Err(e) => Err(SurrealError::Db(DbError::Thrown(e.to_string())).into()),
            },
            _ => Err(DatabaseConnectionErrors::PoolConnectionError.into()),
        }
    }

    /// Retrieve a contact by its unique identifier.
    pub async fn get_by_id(&self, id: String) -> Result<Contact, Box<dyn Error>> {
        let query = format!("SELECT * FROM type::thing({}, \"{}\")", TABLE, id);
        match get_pool().await {
            Ok(pool) => match pool.get().await {
                Ok(connection) => {
                    let mut response = connection.query(&query).await?;
                    if let Some(data) = response.take::<Option<Contact>>(0)? {
                        Ok(data)
                    } else {
                        Err(SurrealError::Db(DbError::NoRecordFound).into())
                    }
                }
                Err(e) => Err(SurrealError::Db(DbError::Thrown(e.to_string())).into()),
            },
            _ => Err(DatabaseConnectionErrors::PoolConnectionError.into()),
        }
    }

    /// Create a new contact in the database.
    pub async fn create_contact(&self, contact: Contact) -> Result<Record, Box<dyn Error>> {
        match get_pool().await {
            Ok(pool) => match pool.get().await {
                Ok(connection) => match connection.create(TABLE).content(contact.clone()).await {
                    Ok(Some(record)) => {
                        let record: Record = record;
                        Ok(record)
                    }
                    Err(e) => Err(e.into()),
                    _ => Err(SurrealError::Db(DbError::CreateStatement {
                        value: contact.to_string(),
                    })
                    .into()),
                },
                Err(e) => Err(SurrealError::Db(DbError::Thrown(e.to_string())).into()),
            },
            _ => Err(DatabaseConnectionErrors::PoolConnectionError.into()),
        }
    }

    /// Update an existing contact in the database.
    pub async fn update_contact(
        &self,
        id: String,
        contact: Contact,
    ) -> Result<Record, Box<dyn Error>> {
        match get_pool().await {
            Ok(pool) => match pool.get().await {
                Ok(connection) => {
                    match connection
                        .update((TABLE, id))
                        .content(contact.clone())
                        .await
                    {
                        Ok(Some(record)) => {
                            let record: Record = record;
                            Ok(record)
                        }
                        Err(e) => Err(e.into()),
                        _ => Err(SurrealError::Db(DbError::UpdateStatement {
                            value: contact.to_string(),
                        })
                        .into()),
                    }
                }
                Err(e) => Err(SurrealError::Db(DbError::Thrown(e.to_string())).into()),
            },
            _ => Err(DatabaseConnectionErrors::PoolConnectionError.into()),
        }
    }

    /// Delete a contact from the database.
    pub async fn delete_contact(&self, id: String) -> Result<Record, Box<dyn Error>> {
        match get_pool().await {
            Ok(pool) => match pool.get().await {
                Ok(connection) => match connection.delete((TABLE, id.clone())).await {
                    Ok(Some(record)) => {
                        let record: Record = record;
                        Ok(record)
                    }
                    Err(e) => Err(e.into()),
                    _ => {
                        Err(SurrealError::Db(DbError::DeleteStatement { value: id.clone() }).into())
                    }
                },
                Err(e) => Err(SurrealError::Db(DbError::Thrown(e.to_string())).into()),
            },
            _ => Err(DatabaseConnectionErrors::PoolConnectionError.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crud() {
        let repo = ContactRepository;
        let john = Contact {
            id: None,
            first: "John".to_string(),
            last: "Doe".to_string(),
            phone: Some("123-456-7890".to_string()),
            email: Some("john@abc.def".to_string()),
        };
        dbg!(&john);
        // Create a new contact
        let record: Record = repo.create_contact(john.clone()).await.unwrap();
        let id = record.id.clone().id;
        dbg!(&id);
        // Verify that it can be retrieved by its unique identifier
        let contact = repo.get_by_id(id.to_string()).await.unwrap();
        dbg!(&contact);
        assert_eq!(contact.first, john.first);
        assert_eq!(contact.last, john.last);
        assert_eq!(contact.phone, john.phone);
        assert_eq!(contact.email, john.email);

        // Update the contact
        // Note that all Contact properties need to be provided
        // To update only a subset of fields, a new patch function would need to be created.
        let jane = Contact {
            id: Some(record.id.clone()),
            first: "Jane".to_string(),
            last: "Doe".to_string(),
            phone: Some("234-567-8901".to_string()),
            email: Some("jane@bcd.efg".to_string()),
        };
        dbg!(&jane);
        let record: Record = repo
            .update_contact(id.to_string(), jane.clone())
            .await
            .unwrap();
        let id = record.id.id;
        // Verify that it can be retrieved
        let contact = repo.get_by_id(id.to_string()).await.unwrap();
        dbg!(&contact);
        assert_eq!(contact.first, jane.first);
        assert_eq!(contact.last, jane.last);
        assert_eq!(contact.phone, jane.phone);
        assert_eq!(contact.email, jane.email);

        // Delete the contact
        let record: Record = repo.delete_contact(id.to_string()).await.unwrap();
        dbg!(record);

        // Verify that it can no longer be retrieved
        let result = repo.get_by_id(id.to_string()).await;
        dbg!(&result);
        assert!(result.is_err());
    }
}
