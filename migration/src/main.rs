use migration::Migrator;
use sea_schema::migration::*;

#[async_std::main]
async fn main() {
    cli::run_cli(Migrator).await;

    /* dotenv::dotenv().ok();

    let url = std::env::var("DATABASE_URL").unwrap();

    println!("url = {}", url);

    let db = Database::connect(url).await.unwrap();

    Migrator::up(&db, None).await.unwrap(); */
}
