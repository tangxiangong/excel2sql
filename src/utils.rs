use crate::{Error, Result, config::ExcelConfig};
use calamine::{Data, DataType, Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExcelDataType {
    Int,
    Float,
    String,
    Bool,
    DateTime,
    NULL,
}

pub fn read_header(
    config: &ExcelConfig,
    sheet: &Range<Data>,
) -> Result<(Vec<String>, Vec<ExcelDataType>)> {
    let header = match (config.headers(), sheet.headers()) {
        (Some(h), Some(_)) => h.clone(),
        (Some(h), None) => h.clone(),
        (None, Some(h)) => h,
        (None, None) => return Err(Error::ExcelConfigError("No header found".to_string())),
    };

    let first_row = sheet
        .rows()
        .nth(config.data_start_row() - 1)
        .ok_or(Error::ExcelConfigError("No data found".to_owned()))?;

    let col_types = first_row.iter().map(infer_type).collect::<Vec<_>>();

    Ok((header, col_types))
}

fn infer_type(data: &Data) -> ExcelDataType {
    if data.is_int() {
        ExcelDataType::Int
    } else if data.is_float() {
        ExcelDataType::Float
    } else if data.is_bool() {
        ExcelDataType::Bool
    } else if data.is_string() {
        ExcelDataType::String
    } else if data.is_empty() || data.is_error() {
        ExcelDataType::NULL
    } else {
        ExcelDataType::DateTime
    }
}
