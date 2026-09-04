//! RF2 release file name parsing, per `spec/03-file-naming.md`.
//!
//! ```
//! use snomed_rf2::filename::ReleaseFileName;
//! use snomed_rf2::release_type::ReleaseType;
//!
//! let f = ReleaseFileName::parse("der2_cRefset_LanguageDelta-en_INT_20190731.txt").unwrap();
//! assert_eq!(f.summary, "Language");
//! assert_eq!(f.release_type, ReleaseType::Delta);
//! assert_eq!(f.language_code.as_deref(), Some("en"));
//! ```

use std::fmt;

use snomed_core::time::EffectiveTime;

use crate::release_type::ReleaseType;

/// The FileType element of a release file name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    /// `sct2` — core terminology component file.
    Sct2,
    /// `der2` — derivative (reference set) file.
    Der2,
}

impl FileType {
    pub const fn as_str(self) -> &'static str {
        match self {
            FileType::Sct2 => "sct2",
            FileType::Der2 => "der2",
        }
    }
}

/// Errors from parsing a release file name.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileNameError {
    /// Not a `.txt` file.
    Extension,
    /// Not five underscore-separated elements.
    Structure,
    /// Unrecognized FileType element.
    FileType(String),
    /// ContentSubType does not end in Full/Snapshot/Delta.
    ReleaseType(String),
    /// VersionDate is not a valid YYYYMMDD date.
    VersionDate(String),
}

impl fmt::Display for FileNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileNameError::Extension => write!(f, "release file name must end in .txt"),
            FileNameError::Structure => {
                write!(
                    f,
                    "release file name must have 5 underscore-separated elements"
                )
            }
            FileNameError::FileType(s) => write!(f, "unrecognized file type `{s}`"),
            FileNameError::ReleaseType(s) => {
                write!(
                    f,
                    "content subtype `{s}` does not end in Full, Snapshot, or Delta"
                )
            }
            FileNameError::VersionDate(s) => write!(f, "invalid version date `{s}`"),
        }
    }
}

impl std::error::Error for FileNameError {}

/// A parsed RF2 release file name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseFileName {
    pub file_type: FileType,
    /// The ContentType element, e.g. `Concept`, `cRefset`, `iisssccRefset`.
    pub content_type: String,
    /// The summary prefix of ContentSubType, e.g. `Language`; empty for
    /// plain component files.
    pub summary: String,
    pub release_type: ReleaseType,
    /// Language code suffix of ContentSubType, e.g. `en` or `en-GB`.
    pub language_code: Option<String>,
    /// The CountryNamespace element, e.g. `INT`, `US1000124`.
    pub country_namespace: String,
    /// The VersionDate element.
    pub version_date: EffectiveTime,
}

impl ReleaseFileName {
    /// Parses a file name (base name only, no directory).
    pub fn parse(name: &str) -> Result<Self, FileNameError> {
        let stem = name.strip_suffix(".txt").ok_or(FileNameError::Extension)?;
        let parts: Vec<&str> = stem.split('_').collect();
        let [file_type, content_type, content_sub_type, country_namespace, version_date] =
            parts.as_slice()
        else {
            return Err(FileNameError::Structure);
        };

        let file_type = match *file_type {
            "sct2" => FileType::Sct2,
            "der2" => FileType::Der2,
            other => return Err(FileNameError::FileType(other.to_string())),
        };

        // ContentSubType = [Summary]ReleaseType[-LanguageCode]
        let (head, language_code) = match content_sub_type.split_once('-') {
            Some((head, lang)) => (head, Some(lang.to_string())),
            None => (*content_sub_type, None),
        };
        let (summary, release_type) = ReleaseType::split_suffix(head)
            .ok_or_else(|| FileNameError::ReleaseType(content_sub_type.to_string()))?;

        let version_date = EffectiveTime::parse(version_date)
            .map_err(|_| FileNameError::VersionDate(version_date.to_string()))?;

        Ok(ReleaseFileName {
            file_type,
            content_type: content_type.to_string(),
            summary: summary.to_string(),
            release_type,
            language_code,
            country_namespace: country_namespace.to_string(),
            version_date,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_concept_full() {
        let f = ReleaseFileName::parse("sct2_Concept_Full_INT_20190731.txt").unwrap();
        assert_eq!(f.file_type, FileType::Sct2);
        assert_eq!(f.content_type, "Concept");
        assert_eq!(f.summary, "");
        assert_eq!(f.release_type, ReleaseType::Full);
        assert_eq!(f.language_code, None);
        assert_eq!(f.country_namespace, "INT");
        assert_eq!(f.version_date.as_u32(), 20190731);
    }

    #[test]
    fn parses_description_snapshot_with_language() {
        let f = ReleaseFileName::parse("sct2_Description_Snapshot-en_INT_20190731.txt").unwrap();
        assert_eq!(f.content_type, "Description");
        assert_eq!(f.release_type, ReleaseType::Snapshot);
        assert_eq!(f.language_code.as_deref(), Some("en"));
    }

    #[test]
    fn parses_extended_map_refset() {
        let f = ReleaseFileName::parse("der2_iisssccRefset_ExtendedMapSnapshot_INT_20190731.txt")
            .unwrap();
        assert_eq!(f.file_type, FileType::Der2);
        assert_eq!(f.content_type, "iisssccRefset");
        assert_eq!(f.summary, "ExtendedMap");
        assert_eq!(f.release_type, ReleaseType::Snapshot);
    }

    #[test]
    fn parses_regional_language_code() {
        let f =
            ReleaseFileName::parse("der2_cRefset_LanguageSnapshot-en-GB_GB1000000_20190401.txt")
                .unwrap();
        assert_eq!(f.summary, "Language");
        assert_eq!(f.language_code.as_deref(), Some("en-GB"));
        assert_eq!(f.country_namespace, "GB1000000");
    }

    #[test]
    fn rejects_malformed_names() {
        assert_eq!(
            ReleaseFileName::parse("sct2_Concept_Full_INT_20190731.csv"),
            Err(FileNameError::Extension)
        );
        assert_eq!(
            ReleaseFileName::parse("sct2_Concept_Full_20190731.txt"),
            Err(FileNameError::Structure)
        );
        assert_eq!(
            ReleaseFileName::parse("xxx2_Concept_Full_INT_20190731.txt"),
            Err(FileNameError::FileType("xxx2".to_string()))
        );
        assert_eq!(
            ReleaseFileName::parse("sct2_Concept_Fully_INT_20190731.txt"),
            Err(FileNameError::ReleaseType("Fully".to_string()))
        );
        assert_eq!(
            ReleaseFileName::parse("sct2_Concept_Full_INT_2019073.txt"),
            Err(FileNameError::VersionDate("2019073".to_string()))
        );
    }
}
