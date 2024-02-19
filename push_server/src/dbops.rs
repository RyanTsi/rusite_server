use std::collections::{HashMap, HashSet};

use sqlx::{self, MySql, Pool, Row};
use crate::Essay;
use anyhow::Result;

pub async fn query_essays_last_save_time(
    pool: &Pool<MySql>,
) -> Result<HashMap<String, f64>> {
    let mut res = HashMap::new();
    let rows = sqlx::query(
        "SELECT eid, last_save_time FROM essays"
    )
    .fetch_all(pool)
    .await?;
    for row in rows {
        let eid: String = row.get("eid");
        let last_save_time: f64= row.get("last_save_time");
        res.insert(eid, last_save_time);
    }
    Ok(res)
}

pub async fn insert_essay(
    pool: &Pool<MySql>,
    essay: &Essay,
    current_time: f64,
) -> Result<()> {

    insert_essay_info(pool, essay, current_time).await?;
    insert_essay_tags(pool, essay).await?;
    insert_essay_categories(pool, essay).await?;

    Ok(())
}

async fn insert_essay_info(
    pool: &Pool<MySql>,
    essay: &Essay,
    current_time: f64,
) -> Result<()>{
    sqlx::query!(
        r#"
INSERT INTO essays (eid, title, date, brief, content, last_save_time) 
VALUES (?, ?, ?, ?, ?, ?)
        "#,
        essay.eid,
        essay.title,
        essay.date,
        essay.brief,
        essay.content,
        current_time,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_essay_tags(
    pool: &Pool<MySql>,
    essay: &Essay
) -> Result<()> {
    let tag_set = query_tag_set(pool).await?;
    for tag in &essay.tags {
        if !tag_set.contains(tag) {
            insert_tag(pool, tag).await?;
        }
        insert_eaasy_tag(pool, &essay.eid, tag).await?;
    }
    Ok(())
}

async fn insert_essay_categories(
    pool: &Pool<MySql>,
    essay: &Essay
) -> Result<()> {
    let category_set = query_category_set(pool).await?;
    for category in &essay.categories {
        if !category_set.contains(category) {
            insert_category(pool, category).await?;
        }
        insert_eaasy_category(pool, &essay.eid, category).await?;
    }
    Ok(())
}

async fn insert_eaasy_tag(
    pool: &Pool<MySql>,
    eid: &str,
    tag: &str,
) -> Result<()> {
    let tag_id: u32 = sqlx::query_scalar!(
        "SELECT id FROM tag_set WHERE tag_name = ?",
        tag
    )
    .fetch_one(pool)
    .await?;

    sqlx::query!(
        "INSERT INTO essay_tag (eid, tag_id) VALUES (?, ?)",
        eid,
        tag_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_eaasy_category(
    pool: &Pool<MySql>,
    eid: &str,
    category: &str,
) -> Result<()> {
    let category_id: u32 = sqlx::query_scalar!(
        "SELECT id FROM category_set WHERE category_name = ?",
        category
    )
    .fetch_one(pool)
    .await?;

    sqlx::query!(
        "INSERT INTO essay_category (eid, category_id) VALUES (?, ?)",
        eid,
        category_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_tag(
    pool: &Pool<MySql>,
    tag: &str,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO tag_set (tag_name) VALUES (?)",
        tag
    )
    .execute(pool)
    .await?
    .last_insert_id();
    
    Ok(())
}

async fn insert_category(
    pool: &Pool<MySql>,
    category: &str,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO category_set (category_name) VALUES (?)",
        category
    )
    .execute(pool)
    .await?;
    
    Ok(())
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

pub async fn update_essay(
    pool: &Pool<MySql>,
    essay: &Essay,
    current_time: f64,
) -> Result<()> {
    delete_essay_tags(pool, essay).await?;
    delete_essay_categories(pool, essay).await?;
    update_essay_info(pool, essay, current_time).await?;
    insert_essay_categories(pool, essay).await?;
    insert_essay_tags(pool, essay).await?;

    Ok(())
}

async fn delete_essay_tags(
    pool: &Pool<MySql>,
    essay: &Essay
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM essay_tag WHERE eid = ?",
        essay.eid
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn delete_essay_categories(
    pool: &Pool<MySql>,
    essay: &Essay
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM essay_category WHERE eid = ?",
        essay.eid
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn update_essay_info(
    pool: &Pool<MySql>,
    essay: &Essay,
    current_time: f64,
) -> Result<()> {
    sqlx::query!(
        r#"
UPDATE essays 
SET title = ?, date = ?, brief = ?, content = ?, last_save_time = ?
WHERE eid = ?
        "#,
        essay.title,
        essay.date,
        essay.brief,
        essay.content,
        current_time,
        essay.eid,
    )
    .execute(pool)
    .await?;
    Ok(())
}