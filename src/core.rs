use crate::{
    Result,
    config::{DatabaseConfig, ExcelConfig},
    utils::read_header,
};
use calamine::{Reader, open_workbook_auto};
use sqlx::query;

pub async fn excel2sql(excel_config: &ExcelConfig, db_config: &DatabaseConfig) -> Result<()> {
    let pool = db_config.connect().await?;
    let mut workbook = open_workbook_auto(excel_config.path())?;
    let sheet = workbook.worksheet_range(excel_config.sheet())?;
    let (_header, _col_types) = read_header(excel_config, &sheet)?;

    let create_cmd = format!(
        "CREATE TABLE IF NOT EXISTS {} ({} {});",
        excel_config.sheet(),
        1,
        1,
    );

    query(&create_cmd).execute(&pool).await?;

    Ok(())
}
