use std::collections::HashSet;

use sqlx::{self, MySql, Pool, Row};
use crate::Essay;
use anyhow::Result;

pub async fn insert_eaasy(
    pool: &Pool<MySql>,
    essay: Essay
) -> Result<u64> {

    let id = sqlx::query!(
        r#"
INSERT INTO essays (eid, title, date, brief, content) 
VALUES (?, ?, ?, ?, ?)
        "#,
        essay.eid,
        essay.title,
        essay.date,
        essay.brief,
        essay.content
    )
    .execute(pool)
    .await?
    .last_insert_id();

    let tag_set = query_tag_set(pool).await?;
    let category_set = query_category_set(pool).await?;

    for tag in essay.tags {
        if !tag_set.contains(&tag) {
            insert_tag(pool, &tag).await?;
        }
        insert_eaasy_tag(pool, &essay.eid, &tag).await?;
    }
    for category in essay.categories {
        if !category_set.contains(&category) {
            insert_category(pool, &category).await?;
        }
        insert_eaasy_category(pool, &essay.eid, &category).await?;
    }

    
    Ok(id)
}

async fn insert_eaasy_tag(
    pool: &Pool<MySql>,
    eid: &str,
    tag: &str,
) -> Result<u64> {
    let tag_id: u32 = sqlx::query_scalar!(
        "SELECT id FROM tag_set WHERE tag_name = ?",
        tag
    )
    .fetch_one(pool)
    .await?;

    let x = sqlx::query!(
        "INSERT INTO essay_tag (eid, tag_id) VALUES (?, ?)",
        eid,
        tag_id
    )
    .execute(pool)
    .await?
    .last_insert_id();
    Ok(x)
}

async fn insert_eaasy_category(
    pool: &Pool<MySql>,
    eid: &str,
    category: &str,
) -> Result<u64> {
    let category_id: u32 = sqlx::query_scalar!(
        "SELECT id FROM category_set WHERE category_name = ?",
        category
    )
    .fetch_one(pool)
    .await?;

    let x = sqlx::query!(
        "INSERT INTO essay_category (eid, category_id) VALUES (?, ?)",
        eid,
        category_id
    )
    .execute(pool)
    .await?
    .last_insert_id();
    Ok(x)
}

async fn insert_tag(
    pool: &Pool<MySql>,
    tag: &str,
) -> Result<u64> {
    let x = sqlx::query!(
        "INSERT INTO tag_set (tag_name) VALUES (?)",
        tag
    )
    .execute(pool)
    .await?
    .last_insert_id();
    
    Ok(x)
}

async fn insert_category(
    pool: &Pool<MySql>,
    category: &str,
) -> Result<u64> {
    let x = sqlx::query!(
        "INSERT INTO category_set (category_name) VALUES (?)",
        category
    )
    .execute(pool)
    .await?
    .last_insert_id();
    
    Ok(x)
}

async fn query_tag_set(
    pool: &Pool<MySql>,
) -> Result<HashSet<String>> {
    let rows = sqlx::query(
        "SELECT tag_name FROM tag_set"
    )
    .fetch_all(pool)
    .await?;
    let mut res = HashSet::new();
    for row in rows {
        res.insert(row.get("tag_name"));
    }
    Ok(res)
}

async fn query_category_set(
    pool: &Pool<MySql>,
) -> Result<HashSet<String>> {
    let rows = sqlx::query(
        "SELECT category_name FROM category_set"
    )
    .fetch_all(pool)
    .await?;
    let mut res = HashSet::new();
    for row in rows {
        res.insert(row.get("category_name"));
    }
    Ok(res)
}