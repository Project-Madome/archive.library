use itertools::Itertools;
use sea_orm::{
    prelude::*,
    sea_query::{ColumnDef, Table},
    ConnectionTrait,
};

use crate::entity::Book;

use super::book_tag;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "books")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    pub page: i32,
    pub language: String,
    pub kind: String, // TODO: to enum
    // pub tags: Vec<super::book_tag::Model>,
    pub created_at: DateTimeUtc,
}

#[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "tag_ref::Entity")]
    BookTag,
}

impl Related<super::book_tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BookTag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Book> for ActiveModel {
    fn from(
        Book {
            id,
            title,
            page,
            language,
            kind,
            created_at,
            ..
        }: Book,
    ) -> Self {
        use sea_orm::ActiveValue::*;

        Self {
            id: Set(id as i64),
            title: Set(title),
            page: Set(page as i32),
            language: Set(language),
            kind: Set(kind.as_str().to_string()),
            created_at: Set(created_at),
        }
    }
}

impl From<(Model, Vec<book_tag::Model>)> for Book {
    fn from(
        (
            Model {
                id,
                title,
                page,
                language,
                kind,
                created_at,
            },
            book_tags,
        ): (Model, Vec<book_tag::Model>),
    ) -> Self {
        Self {
            id: id as u32,
            title,
            page: page as usize,
            language,
            kind: kind.into(),
            created_at,
            tags: book_tags.into_iter().map_into().collect(),
        }
    }
}

pub async fn create_table(db: &DatabaseConnection) {
    let stmt = Table::create()
        .table(Entity)
        .if_not_exists()
        .col(ColumnDef::new(Column::Id).big_integer().primary_key())
        .col(ColumnDef::new(Column::Title).string().not_null())
        .col(ColumnDef::new(Column::Kind).string().not_null())
        .col(ColumnDef::new(Column::Page).integer().not_null())
        .col(ColumnDef::new(Column::Language).string().not_null())
        .col(
            ColumnDef::new(Column::CreatedAt)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .to_owned();

    let builder = db.get_database_backend();
    db.execute(builder.build(&stmt))
        .await
        .expect("create table entity::book");
}

#[allow(clippy::enum_variant_names)]
pub mod tag_ref {
    use sea_orm::{
        prelude::*,
        sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, Table},
        ConnectionTrait, Statement,
    };

    use crate::database::postgresql::entity;

    #[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "books_tag_ref")]
    pub struct Model {
        /// book_id + book_tag_id
        #[sea_orm(primary_key)]
        pub id: i64,
        pub book_id: i64,
        pub book_tag_id: i64,
    }

    #[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "entity::book::Entity",
            from = "Column::BookId",
            to = "entity::book::Column::Id"
        )]
        Book,
        #[sea_orm(
            belongs_to = "entity::book_tag::Entity",
            from = "Column::BookTagId",
            to = "entity::book_tag::Column::Id"
        )]
        BookTag,
    }

    impl Related<entity::book::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Book.def()
        }
    }

    impl Related<entity::book_tag::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::BookTag.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}

    impl ActiveModel {
        pub fn insert(book_id: i64, tag_id: i64) -> Self {
            use sea_orm::ActiveValue::*;

            Self {
                id: NotSet,
                book_id: Set(book_id),
                book_tag_id: Set(tag_id),
            }
        }
    }

    pub async fn create_table(db: &DatabaseConnection) {
        let stmt = Table::create()
            .table(Entity)
            .if_not_exists()
            .col(
                ColumnDef::new(Column::Id)
                    .big_integer()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Column::BookId).big_integer().not_null())
            .col(ColumnDef::new(Column::BookTagId).big_integer().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name(Column::BookId.as_str())
                    .from(Entity, Column::BookId)
                    .to(entity::book::Entity, entity::book::Column::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .name(Column::BookTagId.as_str())
                    .from(Entity, Column::BookTagId)
                    .to(entity::book_tag::Entity, entity::book_tag::Column::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned();

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt))
            .await
            .expect("create table entity::book::tag_ref");

        // TODO: https://github.com/SeaQL/sea-query/issues/232
        // sea-orm create index issue

        /* let idx_book_id = Index::create()
        .table(Entity)
        .name("idx-book-id")
        .col(Column::BookId)
        .to_owned(); */
        let idx_book_id = Statement::from_string(
            builder,
            format!(
                r#"
                CREATE INDEX IF NOT EXISTS "idx-book-id"
                    ON "{}" ("{}")
                "#,
                Entity.as_str(),
                Column::BookId.as_str()
            ),
        );

        db.execute(idx_book_id)
            .await
            .expect("create index entity::book::tag_ref idx-book-id");

        /* let idx_book_tag_id = Index::create()
        .table(Entity)
        .name("idx-book-tag-id")
        .col(Column::BookTagId)
        .to_owned(); */

        let idx_book_tag_id = Statement::from_string(
            builder,
            format!(
                r#"
                CREATE INDEX IF NOT EXISTS "idx-book-tag-id"
                    ON "{}" ("{}")
                "#,
                Entity.as_str(),
                Column::BookTagId.as_str()
            ),
        );

        db.execute(idx_book_tag_id)
            .await
            .expect("create index entity::book::tag_ref idx-book-tag-id");
    }
}
