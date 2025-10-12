use mongodb::{Database, bson::{doc, oid::ObjectId, DateTime as BsonDateTime}};
use crate::types::{Post, PostStatus, CreatePost, UpdatePost};
use futures::stream::TryStreamExt;

/// Create a new post
pub async fn create_post(
    db: &Database,
    author_id: ObjectId,
    data: CreatePost,
) -> Result<Post, mongodb::error::Error> {
    let collection = db.collection::<Post>("posts");
    
    let slug = slugify(&data.title);
    let now = BsonDateTime::now();
    let published_at = if data.status == PostStatus::Published {
        Some(now)
    } else {
        None
    };
    
    let post = Post {
        id: None,
        title: data.title,
        slug,
        content: data.content,
        excerpt: data.excerpt,
        status: data.status,
        author_id,
        created_at: now,
        updated_at: now,
        published_at,
        tags: data.tags.unwrap_or_default(),
        views: 0,
    };
    
    let result = collection.insert_one(&post).await?;
    let inserted_id = result.inserted_id.as_object_id()
        .ok_or_else(|| mongodb::error::Error::custom("Failed to get inserted ID"))?;
    
    Ok(Post {
        id: Some(inserted_id),
        ..post
    })
}

/// Get post by ID
pub async fn get_post_by_id(
    db: &Database,
    id: ObjectId,
) -> Result<Option<Post>, mongodb::error::Error> {
    let collection = db.collection::<Post>("posts");
    collection.find_one(doc! { "_id": id }).await
}

/// Get post by slug
pub async fn get_post_by_slug(
    db: &Database,
    slug: &str,
) -> Result<Option<Post>, mongodb::error::Error> {
    let collection = db.collection::<Post>("posts");
    
    let post = collection
        .find_one(doc! { "slug": slug })
        .await?;
    
    // Increment view count if post exists and is published
    if let Some(ref p) = post {
        if p.status == PostStatus::Published {
            if let Some(post_id) = p.id {
                collection.update_one(
                    doc! { "_id": post_id },
                    doc! { "$inc": { "views": 1 } },
                ).await?;
            }
        }
    }
    
    Ok(post)
}

/// Update a post
pub async fn update_post(
    db: &Database,
    id: ObjectId,
    data: UpdatePost,
) -> Result<Option<Post>, mongodb::error::Error> {
    let collection = db.collection::<Post>("posts");
    
    // Get current post
    let current = match get_post_by_id(db, id).await? {
        Some(post) => post,
        None => return Ok(None),
    };
    
    let mut update_doc = doc! { "updated_at": BsonDateTime::now() };
    
    if let Some(title) = data.title {
        update_doc.insert("title", title.clone());
        update_doc.insert("slug", slugify(&title));
    }
    
    if let Some(content) = data.content {
        update_doc.insert("content", content);
    }
    
    if let Some(excerpt) = data.excerpt {
        update_doc.insert("excerpt", excerpt);
    }
    
    if let Some(status) = &data.status {
        update_doc.insert("status", status.as_str());
        
        // Set published_at if status changed to published
        if *status == PostStatus::Published && current.status != PostStatus::Published {
            update_doc.insert("published_at", BsonDateTime::now());
        }
    }
    
    if let Some(tags) = data.tags {
        update_doc.insert("tags", tags);
    }
    
    collection.update_one(
        doc! { "_id": id },
        doc! { "$set": update_doc },
    ).await?;
    
    // Return updated post
    get_post_by_id(db, id).await
}

/// Delete a post
pub async fn delete_post(
    db: &Database,
    id: ObjectId,
) -> Result<bool, mongodb::error::Error> {
    let collection = db.collection::<Post>("posts");
    
    let result = collection.delete_one(doc! { "_id": id }).await?;
    Ok(result.deleted_count > 0)
}

/// List posts with filtering and pagination
pub async fn list_posts(
    db: &Database,
    status: Option<PostStatus>,
    limit: i64,
    skip: u64,
) -> Result<Vec<Post>, mongodb::error::Error> {
    let collection = db.collection::<Post>("posts");
    
    let filter = match status {
        Some(s) => doc! { "status": s.as_str() },
        None => doc! {},
    };
    
    let options = mongodb::options::FindOptions::builder()
        .sort(doc! { "created_at": -1 })
        .limit(limit)
        .skip(skip)
        .build();
    
    let cursor = collection.find(filter).with_options(options).await?;
    cursor.try_collect().await
}

/// Helper function to create URL-friendly slugs
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Rust is Awesome!"), "rust-is-awesome");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(slugify("Special@#$Characters"), "specialcharacters");
    }
}