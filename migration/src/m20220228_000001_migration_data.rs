use futures::{stream, StreamExt};
use itertools::Itertools;
use sea_schema::migration::{
    sea_orm::Statement,
    sea_query::{self, *},
    *,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220228_000001_migrate_data"
    }
}

fn to_new_tag(kind: &mut String, name: &mut String) {
    if kind == "tag" {
        let (new_kind, new_name) = if name.starts_with("female") {
            ("female", name.replacen("female", "", 1).trim().to_owned())
        } else if name.starts_with("male") {
            ("male", name.replacen("male", "", 1).trim().to_owned())
        } else {
            ("misc", name.to_owned())
        };

        *kind = new_kind.to_string();
        *name = new_name;
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // RENAME TABLE book -> old_book
        /* manager
        .rename_table(
            Table::rename()
                .table(book::Table, old_book::Table)
                .to_owned(),
        )
        .await?; */

        // CREATE TABLE book
        manager
            .create_table(
                Table::create()
                    .table(book::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(book::Column::Id).big_integer().primary_key())
                    .col(ColumnDef::new(book::Column::Title).string().not_null())
                    .col(ColumnDef::new(book::Column::Kind).string().not_null())
                    .col(ColumnDef::new(book::Column::Page).integer().not_null())
                    .col(ColumnDef::new(book::Column::Language).string().not_null())
                    .col(
                        ColumnDef::new(book::Column::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
            .unwrap();

        // CREATE TABLE book_tag
        manager
            .create_table(
                Table::create()
                    .table(book_tag::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(book_tag::Column::Id)
                            .big_integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(book_tag::Column::Kind).string().not_null())
                    .col(ColumnDef::new(book_tag::Column::Name).string().not_null())
                    .to_owned(),
            )
            .await
            .unwrap();

        // CREATE TABLE book_tag_ref
        manager
            .create_table(
                Table::create()
                    .table(book_tag_ref::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(book_tag_ref::Column::Id)
                            .big_integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(book_tag_ref::Column::BookId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(book_tag_ref::Column::BookTagId)
                            .big_integer()
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            
                            .name("idx-book-id")
                            .col(Column::BookId),
                    )
                    .index(
                        Index::create()
                            
                            .name("idx-book-tag-id")
                            .col(Column::BookTagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(book_tag_ref::Column::BookId.as_str())
                            .from(book_tag_ref::Table, book_tag_ref::Column::BookId)
                            .to(book::Table, book::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(book_tag_ref::Column::BookTagId.as_str())
                            .from(book_tag_ref::Table, book_tag_ref::Column::BookTagId)
                            .to(book_tag::Table, book_tag::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
            .unwrap();

        // TODO: kind에서 artist cg, game cg 등 snake case가 아닌 것들을 snake case로 변경해야함
        let migrate_books = format!(
            r#"
        INSERT INTO
            {}(id, title, kind, page, language, created_at)
        SELECT
            id, title, type AS kind, page_count AS page, language, created_at
        FROM
            {}
        ON CONFLICT (id)
            DO NOTHING
        "#,
            book::Table.as_str(),
            old_book::Table.as_str()
        );

        // $1 = 기존 kind
        // $2 = 바뀐 kind
        let update_book_kind = |x: &str| {
            format!(
                r#"
                UPDATE
                    {}
                SET
                    {x} = $2
                WHERE
                    {x} = $1
                "#,
                book::Table.as_str()
            )
        };

        let select_distinct_old_book_tags = format!(
            r#"
            SELECT DISTINCT
                type, name
            FROM
                {}
        "#,
            old_book_tag::Table.as_str()
        );

        let select_old_book_tags = format!(
            r#"
                SELECT
                    *
                FROM
                    {}
            "#,
            old_book_tag::Table.as_str()
        );

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                migrate_books,
            ))
            .await
            .unwrap();

        for (col, old, new) in [
            ("kind", "game cg", "game_cg"),
            ("kind", "artist cg", "artist_cg"),
            ("language", "한국어", "korean"),
        ] {
            manager
                .get_connection()
                .execute(Statement::from_sql_and_values(
                    manager.get_database_backend(),
                    &update_book_kind(col),
                    [old.into(), new.into()],
                ))
                .await
                .unwrap();
        }

        let old_book_tags = manager
            .get_connection()
            .query_all(Statement::from_string(
                manager.get_database_backend(),
                select_distinct_old_book_tags,
            ))
            .await
            .unwrap();

        let mut new_book_tags = Vec::new();

        for (i, old_book_tag) in old_book_tags.into_iter().enumerate() {
            let mut kind = old_book_tag.try_get::<String>("", "type").unwrap();
            let mut name = old_book_tag.try_get::<String>("", "name").unwrap();

            to_new_tag(&mut kind, &mut name);

            new_book_tags.push(((i + 1) as i64, kind, name));
        }

        // sqlx에서 parameter 갯수를 `u16::MAX - 1`으로 제한하고 있음
        // 한번 insert하는데 3개의 파라미터를 넣으니까 대충 20000개씩 나눠서 쿼리날리면됨
        let seperated = new_book_tags.chunks(20000);

        for new_book_tags in seperated.map(|x| x.iter().cloned()) {
            let (vars, values): (Vec<_>, Vec<_>) = new_book_tags
                .into_iter()
                .enumerate()
                .map(|(i, x)| ((i + 1) * 3, x))
                .map(|(i, x)| {
                    (
                        format!("(${}, ${}, ${})", i - 2, i - 1, i),
                        vec![(x.0, (x.1, x.2))],
                    )
                })
                .unzip();

            let vars = vars.join(",");
            let values = values.into_iter().flat_map(|x| {
                x.into_iter()
                    .flat_map(|x| vec![x.0.into(), x.1 .0.into(), x.1 .1.into()])
            });

            let insert_book_tags = format!(
                r#"
                INSERT INTO
                    {}(id, kind, name)
                VALUES
                    {}
                ON CONFLICT (id)
                    DO NOTHING
            "#,
                book_tag::Table.as_str(),
                vars
            );

            manager
                .get_connection()
                .execute(Statement::from_sql_and_values(
                    manager.get_database_backend(),
                    &insert_book_tags,
                    values,
                ))
                .await
                .unwrap();
        }

        let old_book_tags = manager
            .get_connection()
            .query_all(Statement::from_string(
                manager.get_database_backend(),
                select_old_book_tags,
            ))
            .await
            .unwrap();

        let mut books_tag_ref = Vec::new();

        for old_book_tag in old_book_tags {
            let book_id = old_book_tag.try_get::<i32>("", "fk_book_id").unwrap();
            let mut kind = old_book_tag.try_get::<String>("", "type").unwrap();
            let mut name = old_book_tag.try_get::<String>("", "name").unwrap();

            to_new_tag(&mut kind, &mut name);

            let tag_id = new_book_tags
                .iter()
                .find(|(_id, kind_a, name_a)| kind_a == &kind && name_a == &name)
                .unwrap()
                .0;

            books_tag_ref.push((book_id as i64, tag_id as i64));
        }

        let seperated = books_tag_ref.chunks(30000);

        stream::iter(seperated.map(|x| x.to_vec()))
            .for_each_concurrent(5, |books_tag_ref| async {
                let (vars, values): (Vec<_>, Vec<_>) = books_tag_ref
                    .into_iter()
                    .enumerate()
                    .map(|(i, x)| ((i + 1) * 2, x))
                    .map(|(i, x)| (format!("(${}, ${})", i - 1, i), vec![x.0, x.1]))
                    .unzip();

                let vars = vars.join(",");
                let values = values
                    .into_iter()
                    .flat_map(|x| x.into_iter().map(Into::into));

                let insert_book_tags = format!(
                    r#"
                    INSERT INTO
                        {}(book_id, book_tag_id)
                    VALUES
                        {}
                "#,
                    book_tag_ref::Table.as_str(),
                    vars
                );

                println!("{insert_book_tags}");

                manager
                    .get_connection()
                    .execute(Statement::from_sql_and_values(
                        manager.get_database_backend(),
                        &insert_book_tags,
                        values,
                    ))
                    .await
                    .unwrap();
            })
            .await;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}

mod book_tag {
    use sea_schema::sea_query::Iden;

    pub struct Table;

    impl Table {
        pub fn as_str(&self) -> &str {
            "book_tags"
        }
    }

    impl Iden for Table {
        fn unquoted(&self, s: &mut dyn std::fmt::Write) {
            s.write_str(self.as_str()).unwrap();
        }
    }

    pub enum Column {
        Id,
        Kind,
        Name,
    }

    impl Column {
        pub fn as_str(&self) -> &str {
            use Column::*;

            match self {
                Id => "id",
                Kind => "kind",
                Name => "name",
            }
        }
    }

    impl Iden for Column {
        fn unquoted(&self, s: &mut dyn std::fmt::Write) {
            s.write_str(self.as_str()).unwrap();
        }
    }
}

mod book_tag_ref {
    use sea_schema::sea_query::Iden;

    pub struct Table;

    impl Table {
        pub fn as_str(&self) -> &str {
            "books_tag_ref"
        }
    }

    impl Iden for Table {
        fn unquoted(&self, s: &mut dyn std::fmt::Write) {
            s.write_str(self.as_str()).unwrap();
        }
    }

    pub enum Column {
        Id,
        BookId,
        BookTagId,
    }

    impl Column {
        pub fn as_str(&self) -> &str {
            use Column::*;

            match self {
                Id => "id",
                BookId => "book_id",
                BookTagId => "book_tag_id",
            }
        }
    }

    impl Iden for Column {
        fn unquoted(&self, s: &mut dyn std::fmt::Write) {
            s.write_str(self.as_str()).unwrap();
        }
    }
}

mod book {
    use sea_schema::sea_query::Iden;

    pub struct Table;

    impl Table {
        pub fn as_str(&self) -> &str {
            "books"
        }
    }

    impl Iden for Table {
        fn unquoted(&self, s: &mut dyn std::fmt::Write) {
            s.write_str(self.as_str()).unwrap();
        }
    }

    pub enum Column {
        Id,
        Title,
        Kind,
        Page,
        Language,
        CreatedAt,
    }

    impl Column {
        pub fn as_str(&self) -> &str {
            use Column::*;

            match self {
                Id => "id",
                Title => "title",
                Kind => "kind",
                Page => "page",
                Language => "language",
                CreatedAt => "created_at",
            }
        }
    }

    impl Iden for Column {
        fn unquoted(&self, s: &mut dyn std::fmt::Write) {
            s.write_str(self.as_str()).unwrap();
        }
    }
}

mod old_book {
    use sea_schema::sea_query::Iden;

    pub struct Table;

    impl Table {
        pub fn as_str(&self) -> &str {
            "book"
        }
    }

    impl Iden for Table {
        fn unquoted(&self, s: &mut dyn std::fmt::Write) {
            s.write_str(self.as_str()).unwrap();
        }
    }
}

mod old_book_tag {
    use sea_schema::sea_query::Iden;

    pub struct Table;

    impl Table {
        pub fn as_str(&self) -> &str {
            "book_metadata"
        }
    }

    impl Iden for Table {
        fn unquoted(&self, s: &mut dyn std::fmt::Write) {
            s.write_str(self.as_str()).unwrap();
        }
    }
}
