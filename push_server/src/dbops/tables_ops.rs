use std::collections::{HashMap, HashSet};
use chrono::NaiveDateTime;
use sqlx::{self, MySql, Pool, Row};
use crate::data_struct::{Essay, EssayInfo};
use anyhow::{Ok, Result};

/// 得到数据库中所有文章的最后保存时间
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

/// 根据文章的 eid 得到该文章的内容
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

/// 根据文章的 eid 得到该文章的 tags
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

/// 根据文章的 eid 得到该文章的 categories
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

/// 得到所有文章的 info (eid, title, date, brief, tags, categories)
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

/// 根据 eid 得到该文章的 essay_info
pub async fn query_essay_info_from_eid(
    pool: &Pool<MySql>,
    eid: &str,
) -> Result<Option<EssayInfo>> {
    let row = sqlx::query!(
        r#"
SELECT title, date, brief FROM essays WHERE eid = ?
        "#,
        eid
    )
    .fetch_optional(pool)
    .await?;

    let info_data = match row {
        Some(data) => data,
        None => return Ok(None),
    };

    let title = info_data.title;
    let date = info_data.date.unwrap().format("%Y-%m-%d %H:%M:%S").to_string();;
    let brief = info_data.brief;
    let tags = query_essay_tags(pool, eid).await?;
    let categories = query_essay_categories(pool, eid).await?;
    Ok(Some(EssayInfo::new(
        eid.to_string(), title, date, categories, tags, brief
    )))
}

async fn query_eids_from_tag(
    pool: &Pool<MySql>,
    tag: &str,
) -> Result<Vec<String>> {
    let tag_id = get_tag_id_from_name(pool, tag).await?;
    let eids = sqlx::query!(
        r#"
SELECT eid FROM essay_tag WHERE tag_id = ?
        "#,
        tag_id
    )
    .fetch_all(pool)
    .await?;

    let essay_ids: Vec<String> = eids.iter().map(|row| row.eid.to_string()).collect();
    Ok(essay_ids)
}

async fn query_eids_from_category(
    pool: &Pool<MySql>,
    category: &str,
) -> Result<Vec<String>> {
    let category_id = get_category_id_from_name(pool, category).await?;
    let eids = sqlx::query!(
        r#"
SELECT eid FROM essay_category WHERE category_id = ?
        "#,
        category_id
    )
    .fetch_all(pool)
    .await?;
    let essay_ids: Vec<String> = eids.iter().map(|row| row.eid.to_string()).collect();
    Ok(essay_ids)
}

/// 根据 tag 得到拥有这个 tag 的 essay_info
pub async fn query_essays_info_from_tag(
    pool: &Pool<MySql>,
    tag: &str,
) -> Result<Vec<EssayInfo>> {
    let eids = query_eids_from_tag(pool, tag).await?;
    let mut essay_infos = Vec::new();
    for eid in eids {
        match query_essay_info_from_eid(pool, &eid).await? {
            Some(x) => {
                essay_infos.push(x)
            }
            None => ()
        }
    }
    Ok(essay_infos)
}

/// 根据 category 得到 拥有这个category 的 essay_info
pub async fn query_essays_from_category(
    pool: &Pool<MySql>,
    category: &str,
) -> Result<Vec<EssayInfo>> {
    let eids = query_eids_from_category(pool, category).await?;
    let mut essay_infos = Vec::new();
    for eid in eids {
        match query_essay_info_from_eid(pool, &eid).await? {
            Some(x) => {
                essay_infos.push(x)
            }
            None => ()
        }
    }
    Ok(essay_infos)
}


/// 向数据库中添加一篇文章
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

/// 根据 category_name 得到 category_id
async fn get_category_id_from_name(
    pool: &Pool<MySql>,
    category_name: &str,
) -> Result<u32> {
    Ok(
        sqlx::query_scalar!(
            r#"
SELECT id FROM category_set WHERE category_name = ?
            "#,
            category_name
        )
        .fetch_one(pool)
        .await?
    )
}

/// 根据 tag_name 得到 tag_id
async fn get_tag_id_from_name(
    pool: &Pool<MySql>,
    tag_name: &str,
) -> Result<u32> {
    Ok(
        sqlx::query_scalar!(
            r#"
SELECT id FROM tag_set WHERE tag_name = ?
            "#,
            tag_name
        )
        .fetch_one(pool)
        .await?
    )
}

async fn insert_eaasy_category(
    pool: &Pool<MySql>,
    eid: &str,
    category: &str,
) -> Result<()> {
    let category_id: u32 = get_category_id_from_name(pool, category).await?;

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