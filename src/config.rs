use crate::{Error, Result};
use calamine::{Reader, open_workbook_auto};
use regex::Regex;
use sqlx::{AnyPool, any::AnyPoolOptions};
use std::{path::PathBuf, sync::LazyLock};

static DATABASE_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<database>mysql|postgres)://(?P<user>[^:]+):(?P<password>[^@]+)@(?P<host>[^:]+):(?P<port>\d+)/(?P<name>[^?]+)").unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Database {
    MySQL,
    Postgres,
}

impl std::fmt::Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Database::MySQL => write!(f, "mysql"),
            Database::Postgres => write!(f, "postgres"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    database: Database,
    host: String,
    port: u16,
    name: String,
    user: String,
    password: String,
}

impl std::fmt::Display for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}://{}:{}@{}:{}/{}",
            self.database, self.user, self.password, self.host, self.port, self.name
        )
    }
}

impl DatabaseConfig {
    pub fn new(url: &str) -> Result<Self> {
        parse_url(url)
    }

    pub async fn connect(&self) -> Result<AnyPool> {
        Ok(AnyPoolOptions::new().connect(&self.to_string()).await?)
    }

    pub fn from_env() -> Result<Self> {
        let db_url = std::env::var("DATABASE_URL").map_err(|e| {
            Error::DatabaseConfigError(format!(
                "Failed to get environment variable `DATABASE_URL`: {}",
                e
            ))
        })?;
        parse_url(&db_url)
    }

    pub fn database(&self) -> Database {
        self.database
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseConfigBuilder {
    database: Database,
    host: Option<String>,
    port: Option<u16>,
    name: Option<String>,
    user: Option<String>,
    password: Option<String>,
}

impl DatabaseConfigBuilder {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            host: None,
            port: None,
            name: None,
            user: None,
            password: None,
        }
    }

    pub fn host(&mut self, host: &str) -> &mut Self {
        self.host = Some(host.to_owned());
        self
    }

    pub fn port(&mut self, port: u16) -> &mut Self {
        self.port = Some(port);
        self
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_owned());
        self
    }

    pub fn user(&mut self, user: &str) -> &mut Self {
        self.user = Some(user.to_owned());
        self
    }

    pub fn password(&mut self, password: &str) -> &mut Self {
        self.password = Some(password.to_owned());
        self
    }

    pub fn build(&self) -> Result<DatabaseConfig> {
        let host = self.host.clone().unwrap_or("localhost".to_owned());
        let port = self.port.unwrap_or(match self.database {
            Database::MySQL => 3306,
            Database::Postgres => 5432,
        });
        let name = self.name.clone().ok_or(Error::DatabaseConfigError(
            "The database name is not specified".to_owned(),
        ))?;
        let user = self.user.clone().ok_or(Error::DatabaseConfigError(
            "The username is not specified".to_owned(),
        ))?;
        let password = self.password.clone().ok_or(Error::DatabaseConfigError(
            "The password is not specified".to_owned(),
        ))?;

        Ok(DatabaseConfig {
            database: self.database,
            host,
            port,
            name,
            user,
            password,
        })
    }
}

fn parse_url(url: &str) -> Result<DatabaseConfig> {
    let caps = DATABASE_URL_REGEX
        .captures(url)
        .ok_or(Error::DatabaseConfigError(
            "Failed to parse database URL".to_owned(),
        ))?;

    let database = caps
        .name("database")
        .ok_or(Error::DatabaseConfigError(
            "Failed to parse database URL".to_owned(),
        ))?
        .as_str();
    let database = match database {
        "mysql" => Database::MySQL,
        "postgres" => Database::Postgres,
        _ => {
            return Err(Error::DatabaseConfigError(
                "Unsupported database type".to_owned(),
            ));
        }
    };

    let host = caps
        .name("host")
        .ok_or(Error::DatabaseConfigError(
            "Failed to parse database URL".to_owned(),
        ))?
        .as_str()
        .to_owned();
    let port = caps
        .name("port")
        .ok_or(Error::DatabaseConfigError(
            "Failed to parse database URL".to_owned(),
        ))?
        .as_str()
        .parse::<u16>()
        .unwrap();
    let name = caps
        .name("name")
        .ok_or(Error::DatabaseConfigError(
            "Failed to parse database URL".to_owned(),
        ))?
        .as_str()
        .to_owned();
    let user = caps
        .name("user")
        .ok_or(Error::DatabaseConfigError(
            "Failed to parse database URL".to_owned(),
        ))?
        .as_str()
        .to_owned();
    let password = caps
        .name("password")
        .ok_or(Error::DatabaseConfigError(
            "Failed to parse database URL".to_owned(),
        ))?
        .as_str()
        .to_owned();

    Ok(DatabaseConfig {
        database,
        host,
        port,
        name,
        user,
        password,
    })
}

#[derive(Debug, Clone)]
pub struct ExcelConfig {
    path: PathBuf,
    sheet: String,
    data_start_row: usize,
    headers: Option<Vec<String>>,
}

impl ExcelConfig {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn sheet(&self) -> &str {
        &self.sheet
    }

    pub fn data_start_row(&self) -> usize {
        self.data_start_row
    }

    pub fn headers(&self) -> Option<&Vec<String>> {
        self.headers.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct ExcelConfigBuilder {
    path: PathBuf,
    sheet: Option<String>,
    data_start_row: Option<usize>,
    headers: Option<Vec<String>>,
}

impl ExcelConfigBuilder {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            sheet: None,
            data_start_row: None,
            headers: None,
        }
    }

    pub fn sheet(mut self, sheet: impl Into<String>) -> Self {
        self.sheet = Some(sheet.into());
        self
    }

    pub fn data_start_row(mut self, row: usize) -> Self {
        self.data_start_row = Some(row);
        self
    }

    pub fn headers(mut self, headers: impl Into<Vec<String>>) -> Self {
        self.headers = Some(headers.into());
        self
    }

    pub fn build(self) -> Result<ExcelConfig> {
        let workbook = open_workbook_auto(&self.path)?;
        let sheets = workbook.sheet_names();
        if sheets.is_empty() {
            return Err(Error::ExcelConfigError("No sheet found".to_string()));
        }
        let sheet = self.sheet.unwrap_or(sheets[0].clone());
        Ok(ExcelConfig {
            path: self.path,
            sheet,
            data_start_row: self.data_start_row.unwrap_or(1),
            headers: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let url = "mysql://root:123456@localhost:3306/test";
        let config = parse_url(url).unwrap();
        assert_eq!(config.database, Database::MySQL);
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 3306);
        assert_eq!(config.name, "test");
        assert_eq!(config.user, "root");
        assert_eq!(config.password, "123456");
    }
}
