//! Streaming reader for RF2 files: validates the header, then yields one
//! typed record per data row.

use std::io::BufRead;
use std::marker::PhantomData;

use crate::error::Rf2Error;
use crate::record::Rf2Record;

/// Streams `T` records from an RF2 file.
///
/// - Validates the header row against `T::HEADER` on construction.
/// - Accepts LF or CRLF line endings and a leading UTF-8 BOM.
/// - Skips empty lines.
pub struct Rf2Reader<R: BufRead, T: Rf2Record> {
    lines: std::io::Lines<R>,
    /// 1-based line number of the last line read.
    line_no: u64,
    _marker: PhantomData<T>,
}

impl<R: BufRead, T: Rf2Record> Rf2Reader<R, T> {
    /// Reads and validates the header row.
    pub fn new(reader: R) -> Result<Self, Rf2Error> {
        let mut lines = reader.lines();
        let header = match lines.next() {
            None => return Err(Rf2Error::MissingHeader),
            Some(line) => line?,
        };
        let header = header.trim_start_matches('\u{feff}');
        let header = header.strip_suffix('\r').unwrap_or(header);
        let found: Vec<&str> = header.split('\t').collect();
        if found != T::HEADER {
            return Err(Rf2Error::Header {
                expected: T::HEADER.join("\t"),
                found: found.join("\t"),
            });
        }
        Ok(Rf2Reader {
            lines,
            line_no: 1,
            _marker: PhantomData,
        })
    }
}

impl<R: BufRead, T: Rf2Record> Iterator for Rf2Reader<R, T> {
    type Item = Result<T, Rf2Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = match self.lines.next()? {
                Ok(line) => line,
                Err(e) => return Some(Err(e.into())),
            };
            self.line_no += 1;
            let line = line.strip_suffix('\r').unwrap_or(&line);
            if line.is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() != T::HEADER.len() {
                return Some(Err(Rf2Error::ColumnCount {
                    line: self.line_no,
                    expected: T::HEADER.len(),
                    found: fields.len(),
                }));
            }
            return Some(match T::parse_fields(&fields) {
                Ok(record) => Ok(record),
                Err(e) => Err(Rf2Error::Field {
                    line: self.line_no,
                    column: e.column,
                    message: e.message,
                }),
            });
        }
    }
}

/// Reads an entire RF2 file into a `Vec`, stopping at the first error.
pub fn read_all<R: BufRead, T: Rf2Record>(reader: R) -> Result<Vec<T>, Rf2Error> {
    Rf2Reader::<R, T>::new(reader)?.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::components::Concept;
    use snomed_core::sctid::{ComponentType, SctId};

    const HEADER: &str = "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId";

    #[test]
    fn reads_concepts_with_crlf_and_bom() {
        let data = format!(
            "\u{feff}{HEADER}\r\n138875005\t20190731\t1\t900000000000207008\t900000000000074008\r\n\r\n"
        );
        let concepts: Vec<Concept> = read_all(data.as_bytes()).unwrap();
        assert_eq!(concepts.len(), 1);
        assert_eq!(concepts[0].id.as_u64(), 138875005);
        assert!(concepts[0].active);
    }

    #[test]
    fn rejects_wrong_header() {
        let data = "id\teffectiveTime\tactive\tmoduleId\twrongColumn\n";
        let err = read_all::<_, Concept>(data.as_bytes()).unwrap_err();
        assert!(matches!(err, Rf2Error::Header { .. }), "{err}");
    }

    #[test]
    fn rejects_missing_header() {
        let err = read_all::<_, Concept>("".as_bytes()).unwrap_err();
        assert!(matches!(err, Rf2Error::MissingHeader));
    }

    #[test]
    fn reports_line_numbers() {
        let data = format!(
            "{HEADER}\n138875005\t20190731\t1\t900000000000207008\t900000000000074008\n138875005\t20190731\tX\t900000000000207008\t900000000000074008\n"
        );
        let err = read_all::<_, Concept>(data.as_bytes()).unwrap_err();
        match err {
            Rf2Error::Field { line, column, .. } => {
                assert_eq!(line, 3);
                assert_eq!(column, "active");
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn reports_column_count() {
        let data = format!("{HEADER}\n138875005\t20190731\t1\n");
        let err = read_all::<_, Concept>(data.as_bytes()).unwrap_err();
        assert!(matches!(
            err,
            Rf2Error::ColumnCount {
                line: 2,
                expected: 5,
                found: 3
            }
        ));
    }

    #[test]
    fn rejects_a_row_whose_id_partition_names_another_component_type() {
        // spec/05 rule 1 via the reader: a description id (partition 01) in
        // a Concept file is a malformed file, and the error names the column.
        let description_id = SctId::compose(1001, ComponentType::Description, None).unwrap();
        let file = format!(
            "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
             {description_id}\t20240101\t1\t900000000000207008\t900000000000074008\n"
        );
        let mut reader = Rf2Reader::<_, Concept>::new(file.as_bytes()).unwrap();
        match reader.next() {
            Some(Err(Rf2Error::Field { line, column, .. })) => {
                assert_eq!(line, 2);
                assert_eq!(column, "id");
            }
            other => panic!("expected a field error on line 2, got {other:?}"),
        }
    }
}
