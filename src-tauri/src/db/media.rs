use rusqlite::{params, Transaction};
use crate::core::error::AppResult;

pub fn insert_media_row(trans: &Transaction, media_type: &str, now: i64) -> AppResult<i64> {
    trans.execute(
        r#"
            INSERT INTO media (
                content_hash,
                media_type,
                width,
                height,
                duration_ms,
                hash_status,
                metadata_status,
                thumbnail_status,
                created_at,
                updated_at
            ) VALUES (
                NULL,
                ?1,
                NULL,
                NULL,
                NULL,
                'pending',
                'pending',
                'pending',
                ?2,
                ?2
            )
        "#,
        params![media_type, now],
    )?;

    Ok(trans.last_insert_rowid())
}