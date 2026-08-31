use std::{error::Error, fmt, io};

#[derive(Debug)]
pub enum ConversionError {
    InvalidConfig(String),
    Excel(String),
    ResourceDiscovery(String),
    TargetResolution(String),
    InvalidRegex(String),
    Xml(String),
    Io(io::Error),
    Transaction(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(message) => write!(formatter, "配置错误: {message}"),
            Self::Excel(message) => write!(formatter, "Excel 错误: {message}"),
            Self::ResourceDiscovery(message) => write!(formatter, "资源目录错误: {message}"),
            Self::TargetResolution(message) => write!(formatter, "目标文件错误: {message}"),
            Self::InvalidRegex(message) => write!(formatter, "正则表达式错误: {message}"),
            Self::Xml(message) => write!(formatter, "XML 错误: {message}"),
            Self::Io(error) => write!(formatter, "I/O 错误: {error}"),
            Self::Transaction(message) => write!(formatter, "文件事务错误: {message}"),
        }
    }
}

impl Error for ConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for ConversionError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<quick_xml::Error> for ConversionError {
    fn from(error: quick_xml::Error) -> Self {
        Self::Xml(error.to_string())
    }
}

impl From<calamine::XlsxError> for ConversionError {
    fn from(error: calamine::XlsxError) -> Self {
        Self::Excel(error.to_string())
    }
}

pub type ConversionResult<T> = Result<T, ConversionError>;
