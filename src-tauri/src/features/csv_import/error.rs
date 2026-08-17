#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CsvImportError {
    EmptyInput,
    UnsupportedDialect,
    MissingRequiredHeader(&'static str),
    DuplicateHeader(&'static str),
    MalformedCsv {
        row: usize,
    },
    InvalidValue {
        row: usize,
        field: &'static str,
        expected: &'static str,
    },
    DuplicateUid {
        first_row: usize,
        row: usize,
    },
    TooManyRows {
        limit: usize,
    },
}

impl std::fmt::Display for CsvImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "CSV input is empty"),
            Self::UnsupportedDialect => write!(f, "CSV must use comma or semicolon delimiters"),
            Self::MissingRequiredHeader(header) => {
                write!(f, "CSV is missing required {header} header")
            }
            Self::DuplicateHeader(header) => write!(f, "CSV has duplicate {header} headers"),
            Self::MalformedCsv { row } => write!(f, "CSV record {row} is malformed"),
            Self::InvalidValue {
                row,
                field,
                expected,
            } => write!(
                f,
                "CSV record {row} has invalid {field}; expected {expected}"
            ),
            Self::DuplicateUid { first_row, row } => {
                write!(
                    f,
                    "CSV record {row} repeats the Unique ID from record {first_row}"
                )
            }
            Self::TooManyRows { limit } => write!(f, "CSV contains more than {limit} rows"),
        }
    }
}

impl std::error::Error for CsvImportError {}
