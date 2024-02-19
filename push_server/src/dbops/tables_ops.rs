use std::collections::{HashMap, HashSet};
use chrono::NaiveDateTime;
use sqlx::{self, MySql, Pool, Row};
use crate::data_struct::{Essay, EssayInfo};
use anyhow::Result;

pub async fn query_essays_last_save_time(
    pool: &Pool<MySql>,
) -> Result<HashMap<String, f64>> {
    let mut res = HashMap::new();
    let rows = sqlx::query(
        r#"
SELECT eid, last_save_time FROM essays
        "#
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

pub async fn query_essay_content(
    pool: &Pool<MySql>,
    eid: &str,
) -> Result<Option<String>> {
    Ok(
        sqlx::query_scalar!(
        r#"
SELECT content
FROM essays
WHERE eid = ?
        "#,
        eid,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn query_essay_tags(
    pool: &Pool<MySql>,
    eid: &str,
) -> Result<Vec<String>> {
    Ok(
        sqlx::query_scalar!(
        r#"
SELECT ts.tag_name
FROM essay_tag et
JOIN tag_set ts ON et.tag_id = ts.id
WHERE et.eid = ?
        "#,
        eid,
    )
    .fetch_all(pool)
    .await?)
    
}

pub async fn query_essay_categories(
    pool: &Pool<MySql>,
    eid: &str
) -> Result<Vec<String>> {
    let res = sqlx::query_scalar!(
        r#"
SELECT cs.category_name
FROM essay_category ec
JOIN category_set cs ON ec.category_id = cs.id
WHERE ec.eid = ?
        "#,
        eid,
    )
    .fetch_all(pool)
    .await?;
    
    Ok(res)
}

pub async fn query_essay_info(
    pool: &Pool<MySql>,
) -> Result<Vec<EssayInfo>> {
    let rows = sqlx::query(
        r#"
SELECT eid, title, date, brief FROM essays
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut res = Vec::new();
    for row in rows {
        let eid: String = row.get("eid");
        let title: String = row.get("title");
        let date: NaiveDateTime = row.get("date");
        let date = date.format("%Y-%m-%d %H:%M:%S").to_string();
        let brief: String = row.get("brief");
        let tags: Vec<String> = query_essay_tags(pool, &eid).await?;
        let categories: Vec<String> = query_essay_categories(pool, &eid).await?;
        res.push( EssayInfo::new( eid, title, date, categories, tags, brief) );
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
    let tag_id = sqlx::query_scalar!(
        r#"
SELECT id FROM tag_set WHERE tag_name = ?
        "#,
        tag
    )
    .fetch_one(pool)
    .await?;

    sqlx::query!(
        r#"
INSERT INTO essay_tag (eid, tag_id) VALUES (?, ?)
        "#,
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
        r#"
SELECT id FROM category_set WHERE category_name = ?
        "#,
        category
    )
    .fetch_one(pool)
    .await?;

    sqlx::query!(
        r#"
INSERT INTO essay_category (eid, category_id) VALUES (?, ?)
        "#,
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
        r#"
INSERT INTO tag_set (tag_name) VALUES (?)
        "#,
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
        r#"
INSERT INTO category_set (category_name) VALUES (?)
        "#,
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
        r#"
SELECT tag_name FROM tag_set
        "#
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
        r#"
SELECT category_name FROM category_set
        "#
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
    delete_essay_tags(pool, &essay.eid).await?;
    delete_essay_categories(pool, &essay.eid).await?;
    update_essay_info(pool, essay, current_time).await?;
    insert_essay_categories(pool, essay).await?;
    insert_essay_tags(pool, essay).await?;

    Ok(())
}

async fn delete_essay_tags(
    pool: &Pool<MySql>,
    eid: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
DELETE FROM essay_tag WHERE eid = ?
        "#,
        eid
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn delete_essay_categories(
    pool: &Pool<MySql>,
    eid: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
DELETE FROM essay_category WHERE eid = ?
        "#,
        eid
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

pub async fn delete_essay(
    pool: &Pool<MySql>,
    eid: &str,
) -> Result<()> {
    delete_essay_categories(pool, eid).await?;
    delete_essay_tags(pool, eid).await?;
    sqlx::query!(
        r#"
DELETE FROM essays WHERE eid = ?
        "#,
        eid
    )
    .execute(pool)
    .await?;
    Ok(())
}