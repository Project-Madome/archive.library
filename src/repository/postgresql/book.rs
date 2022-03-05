use chrono::{DateTime, Utc};
use itertools::Itertools;
use sai::{Component, ComponentLifecycle, Injected};
use sea_orm::{
    ConnectionTrait, DbErr, EntityTrait, IdenStatic, QueryResult, Statement, TransactionError,
    TransactionTrait, Value,
};

use crate::{
    constant::postgresql,
    database::{
        postgresql::entity::{book, book_tag},
        DatabaseSet,
    },
    entity::{Book, BookGroupByTag, BookKind, BookSortBy, BookTag, Sort},
    repository::r#trait::BookRepository,
};

#[derive(Component)]
#[lifecycle]
pub struct PostgresqlBookRepository {
    #[injected]
    database: Injected<DatabaseSet>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for PostgresqlBookRepository {
    async fn start(&mut self) {
        book_tag::create_table(self.database.postgresql()).await;
        book::create_table(self.database.postgresql()).await;
        book::tag_ref::create_table(self.database.postgresql()).await;
    }
}

#[async_trait::async_trait]
impl BookRepository for PostgresqlBookRepository {
    async fn get_one(&self, book_id: u32) -> crate::Result<Option<Book>> {
        let (sql, values) = select_books_sql(SelectBy::Id(book_id));

        let db = self.database.postgresql();
        let psql = db.get_database_backend();
        let stmt = Statement::from_sql_and_values(psql, &sql, values);

        let query_results = db.query_all(stmt).await?;

        let mut books = into_books(query_results)?;

        Ok(books.next())
    }

    async fn get_many(
        &self,
        kind: Option<BookKind>,
        per_page: usize,
        page: usize,
        sort_by: BookSortBy,
    ) -> crate::Result<Vec<Book>> {
        let (query, values) = select_books_sql(SelectBy::Many(kind, per_page, page, sort_by));

        let db = self.database.postgresql();
        let psql = db.get_database_backend();
        let stmt = Statement::from_sql_and_values(psql, &query, values);

        let query_results = db.query_all(stmt).await?;

        let books = into_books(query_results)?;

        Ok(books.collect())
    }

    async fn get_many_by_ids(&self, book_ids: Vec<u32>) -> crate::Result<Vec<Book>> {
        let (query, values) = select_books_sql(SelectBy::Ids(book_ids));

        let db = self.database.postgresql();
        let psql = db.get_database_backend();
        let stmt = Statement::from_sql_and_values(psql, &query, values);

        let query_results = db.query_all(stmt).await?;

        let books = into_books(query_results)?;

        Ok(books.collect())
    }

    async fn get_many_by_tags(
        &self,
        book_tags: Vec<BookTag>,
    ) -> crate::Result<Vec<BookGroupByTag>> {
        let sql = r#"
            SELECT

        "#;

        unimplemented!()
    }

    async fn get_many_by_tag(&self, book_tag: BookTag) -> crate::Result<Vec<Book>> {
        unimplemented!()
    }

    async fn add(&self, book: Book) -> crate::Result<bool> {
        let db = self.database.postgresql();

        let r = db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let book_id = book.id;

                    let book_tag_ids = insert_book_tags(book.tags, txn).await?;

                    book::Entity::insert::<book::ActiveModel>(
                        Book {
                            tags: Vec::new(),
                            ..book
                        }
                        .into(),
                    )
                    .exec(txn)
                    .await?;

                    book::tag_ref::Entity::insert_many(
                        book_tag_ids.into_iter().map(|tag_id| {
                            book::tag_ref::ActiveModel::insert(book_id as i64, tag_id)
                        }),
                    )
                    .exec(txn)
                    .await?;

                    Ok(())
                })
            })
            .await;

        match r {
            Ok(_) => Ok(true),
            Err(TransactionError::Connection(err) | TransactionError::Transaction(err))
                if err.to_string().contains(postgresql::DUPLICATE_KEY_VALUE) =>
            {
                Ok(false)
            }
            Err(err) => Err(err.into()),
        }
    }
}

fn into_books(query_results: Vec<QueryResult>) -> Result<impl Iterator<Item = Book>, DbErr> {
    let mut xs = Vec::new();

    for res in query_results {
        let book = book::Model {
            id: res.try_get::<i64>("", "A_id")?,
            title: res.try_get::<String>("", "A_title")?,
            page: res.try_get::<i32>("", "A_page")?,
            language: res.try_get::<String>("", "A_language")?,
            kind: res.try_get::<String>("", "A_kind")?,
            created_at: res.try_get::<DateTime<Utc>>("", "A_created_at")?,
        };

        // 작품에 태그가 하나도 없으면 None임
        let book_tag = match res.try_get::<Option<i64>>("", "B_id")? {
            Some(id) => Some(book_tag::Model {
                id,
                kind: res.try_get::<String>("", "B_kind")?,
                name: res.try_get::<String>("", "B_name")?,
            }),
            None => None,
        };

        if xs.is_empty() {
            xs.push((book, book_tag.map(|x| vec![x]).unwrap_or_default()));
        } else {
            let (left, right) = xs.last_mut().unwrap();

            // 태그가 없는 작품은 하나씩밖에 없는데,
            // left.id와 book.id가 같다면 책에는 무조건 두개 이상의 태그가 있다는 소리임
            // book_tag는 None일 수가 없음
            if left.id == book.id {
                right.push(book_tag.unwrap());
            } else {
                xs.push((book, book_tag.map(|x| vec![x]).unwrap_or_default()))
            }
        }
    }
    log::debug!("rows.into_book = {xs:?}");

    Ok(xs.into_iter().map_into())
}

enum SelectBy {
    Ids(Vec<u32>),
    Id(u32),
    /// kind, per_page, page, sort_by
    Many(Option<BookKind>, usize, usize, BookSortBy),
}

fn select_books_sql(select_by: SelectBy) -> (String, Vec<Value>) {
    let books = book::Entity.as_str();
    let book_tags = book_tag::Entity.as_str();
    let books_tag_ref = book::tag_ref::Entity.as_str();

    let (mut where_, mut offset, mut limit, mut order_by, mut last_order_by, mut values) =
        Default::default();

    match select_by {
        SelectBy::Many(kind, per_page, page, sort_by) => {
            let sort_by = match sort_by {
                BookSortBy::Id(Sort::Desc) => {
                    format!(r#""{books}"."id" DESC"#)
                }
                BookSortBy::Id(Sort::Asc) => {
                    format!(r#""{books}"."id" ASC"#)
                }
                BookSortBy::Random => "RANDOM()".to_string(),
            };

            order_by = format!("ORDER BY {sort_by}");

            offset = "OFFSET $1";
            limit = "LIMIT $2";

            where_ = if kind.is_some() {
                format!(r#"WHERE "{books}"."kind" = $3"#,)
            } else {
                where_
            };

            values = match kind {
                Some(kind) => {
                    vec![
                        ((per_page * (page - 1)) as u64).into(),
                        (per_page as u64).into(),
                        kind.as_str().into(),
                    ]
                }
                None => {
                    vec![
                        ((per_page * (page - 1)) as u64).into(),
                        (per_page as u64).into(),
                    ]
                }
            };
        }
        SelectBy::Ids(book_ids) => {
            let (vars, vals): (Vec<_>, Vec<_>) = book_ids
                .into_iter()
                .enumerate()
                .map(|(i, x)| (i + 1, x))
                .map(|(i, x)| (format!("${}", i), x))
                .unzip();

            where_ = format!(r#"WHERE "{books}"."id" IN ({vars})"#, vars = vars.join(","));

            last_order_by = format!(
                r#"ORDER BY {vars}"#,
                vars = vars
                    .iter()
                    .map(|var| format!(r#""{books}"."id" = {var} DESC"#))
                    // .rev()
                    .join(",")
            );

            values = vals.into_iter().map_into().collect();
        }
        SelectBy::Id(book_id) => {
            where_ = format!(r#"WHERE "{books}"."id" = $1"#,);

            values = vec![book_id.into()];
        }
    };

    let query = format!(
        r#"
        SELECT
            "{books}"."id" AS "A_id",
            "{books}"."title" AS "A_title",
            "{books}"."page" AS "A_page",
            "{books}"."language" AS "A_language",
            "{books}"."kind" AS "A_kind",
            "{books}"."created_at" AS "A_created_at",
            "{book_tags}"."id" AS "B_id",
            "{book_tags}"."kind" AS "B_kind",
            "{book_tags}"."name" AS "B_name"
        FROM
            (SELECT * FROM "{books}" {where_} {order_by} {offset} {limit}) AS "{books}"
        LEFT JOIN "{books_tag_ref}"
            ON "{books_tag_ref}"."book_id" = "{books}"."id"
        LEFT JOIN "{book_tags}"
            ON "{book_tags}"."id" = "{books_tag_ref}"."book_tag_id"
        {last_order_by}
        "#
    );

    log::debug!("values = {values:?}");

    (query, values)
}

async fn insert_book_tags(
    book_tags: Vec<BookTag>,
    db: &impl ConnectionTrait,
) -> Result<Vec<i64>, DbErr> {
    let vars = (1..=book_tags.len())
        .map(|i| i * 2) // i * column count
        .fold(Vec::new(), |mut acc, i| {
            // "($2, $3)"
            acc.push(format!("(${}, ${})", i - 1, i));
            acc
        })
        .join(",");

    let values = book_tags
        .into_iter()
        .flat_map(|x| vec![x.kind().into(), x.name().into()]);

    let insert_book_tags_sql = format!(
        r#"
        INSERT INTO
            book_tag(kind, name)
        VALUES
            {}
        ON CONFLICT (kind, name)
            DO NOTHING
        RETURNING id
        "#,
        vars
    );

    let psql = db.get_database_backend();
    let res = db
        .query_all(Statement::from_sql_and_values(
            psql,
            &insert_book_tags_sql,
            values,
        ))
        .await?;

    res.into_iter()
        .map(|x| x.try_get("", book_tag::Column::Id.as_str()))
        .collect::<Result<_, _>>()
}
