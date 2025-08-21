#[derive(Debug)]
pub enum Error {
    HttpError(reqwest::Error),
    MissingParameter,
    ConverterError(String),
}
