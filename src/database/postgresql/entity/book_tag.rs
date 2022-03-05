use sea_orm::{
    prelude::*,
    sea_query::{ColumnDef, Index, Table},
    ConnectionTrait, Statement,
};

use crate::entity::BookTag;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "book_tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    // TODO: to enum
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<BookTag> for ActiveModel {
    fn from(book_tag: BookTag) -> Self {
        use sea_orm::ActiveValue::*;

        Self {
            id: NotSet,
            kind: Set(book_tag.kind().to_string()),
            name: Set(book_tag.name().to_string()),
        }
    }
}

impl From<Model> for BookTag {
    fn from(Model { kind, name, .. }: Model) -> Self {
        match kind.as_str() {
            "artist" => Self::Artist(name),
            "series" => Self::Series(name),
            "group" => Self::Group(name),
            "character" => Self::Character(name),
            "female" => Self::Female(name),
            "male" => Self::Male(name),
            "misc" => Self::Misc(name),
            _ => unreachable!(),
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
                .primary_key()
                .auto_increment(),
        )
        .col(ColumnDef::new(Column::Kind).string().not_null())
        .col(ColumnDef::new(Column::Name).string().not_null())
        .to_owned();

    let builder = db.get_database_backend();
    db.execute(builder.build(&stmt))
        .await
        .expect("create table entity::book_tag");

    let idx_name = Statement::from_string(
        builder,
        format!(
            r#"
            CREATE INDEX IF NOT EXISTS "idx-name"
                ON "{}" ("{}")
            "#,
            Entity.as_str(),
            Column::Name.as_str()
        ),
    );

    db.execute(idx_name)
        .await
        .expect("create index entity::book_tag idx-name");
}
