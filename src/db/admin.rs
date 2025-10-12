use mongodb::{Database, bson::{doc, oid::ObjectId, DateTime as BsonDateTime}};
use crate::types::User;
use bcrypt::{verify, hash, DEFAULT_COST};

/// Get the admin user (the first user in the system)
pub async fn get_admin_user(db: &Database) -> Result<Option<User>, mongodb::error::Error> {
    let collection = db.collection::<User>("users");
    collection.find_one(doc! {}).await
}

/// Verify admin password
pub async fn verify_admin_password(
    db: &Database,
    username: &str,
    password: &str,
) -> Result<Option<User>, Box<dyn std::error::Error>> {
    let collection = db.collection::<User>("users");
    
    let user = collection
        .find_one(doc! { "username": username })
        .await?;

    match user {
        Some(user) => {
            if verify(password, &user.password_hash)? {
                Ok(Some(user))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

/// Update last login time
pub async fn update_last_login(
    db: &Database,
    user_id: ObjectId,
) -> Result<(), mongodb::error::Error> {
    let collection = db.collection::<User>("users");
    
    collection.update_one(
        doc! { "_id": user_id },
        doc! { "$set": { "last_login": BsonDateTime::now() } },
    ).await?;
    
    Ok(())
}

/// Initialize admin user (run this once during setup)
pub async fn initialize_admin(
    db: &Database,
    username: &str,
    password: &str,
    email: &str,
) -> Result<User, Box<dyn std::error::Error>> {
    let password_hash = hash(password, DEFAULT_COST)?;
    let collection = db.collection::<User>("users");
    
    let user = User {
        id: None, // MongoDB will generate this
        username: username.to_string(),
        password_hash,
        email: email.to_string(),
        created_at: BsonDateTime::now(),
        last_login: None,
    };
    
    let result = collection.insert_one(&user).await?;
    let inserted_id = result.inserted_id.as_object_id()
        .ok_or("Failed to get inserted ID")?;
    
    Ok(User {
        id: Some(inserted_id),
        ..user
    })
}

/// Check if any admin exists
pub async fn admin_exists(db: &Database) -> Result<bool, mongodb::error::Error> {
    let collection = db.collection::<User>("users");
    let count = collection.count_documents(doc! {}).await?;
    Ok(count > 0)
}