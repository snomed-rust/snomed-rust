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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::components::{AlternateIdentifier, Concept, RelationshipConcreteValue};
    use snomed_core::concrete_value::ConcreteValue;
    use snomed_core::sctid::{ComponentType, SctId};
    use std::io::{self, BufRead, BufReader, Cursor, Read};

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
    fn reads_and_validates_alternate_identifiers() {
        let scheme = SctId::compose(2001, ComponentType::Concept, None).unwrap();
        let component = SctId::compose(2002, ComponentType::Concept, None).unwrap();
        let data = format!(
            "alternateIdentifier\teffectiveTime\tactive\tmoduleId\tidentifierSchemeId\treferencedComponentId\n\
             L-1000\t20250321\t1\t900000000000207008\t{scheme}\t{component}\n"
        );
        let rows: Vec<AlternateIdentifier> = read_all(data.as_bytes()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].alternate_identifier, "L-1000");
        assert_eq!(rows[0].identifier_scheme_id, scheme);
        assert_eq!(rows[0].referenced_component_id, component);

        for (invalid, column) in [
            (data.replace("L-1000\t", "\t"), "alternateIdentifier"),
            (data.replace("20250321", "20251340"), "effectiveTime"),
            (data.replace("\t1\t", "\tmaybe\t"), "active"),
            (
                data.replace("900000000000207008", "not-an-sctid"),
                "moduleId",
            ),
            (
                data.replace(&scheme.to_string(), "not-an-sctid"),
                "identifierSchemeId",
            ),
            (
                data.replace(&component.to_string(), "not-an-sctid"),
                "referencedComponentId",
            ),
        ] {
            assert!(
                matches!(
                    read_all::<_, AlternateIdentifier>(invalid.as_bytes()),
                    Err(Rf2Error::Field { column: actual, .. }) if actual == column
                ),
                "invalid {column} did not report that column"
            );
        }
    }

    #[test]
    fn reads_concrete_relationship_values_without_losing_decimal_precision() {
        let relationship = SctId::compose(3001, ComponentType::Relationship, None).unwrap();
        let source = SctId::compose(3002, ComponentType::Concept, None).unwrap();
        let data = format!(
            "id\teffectiveTime\tactive\tmoduleId\tsourceId\tvalue\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId\n\
             {relationship}\t20250321\t1\t900000000000207008\t{source}\t#1.50\t2\t116680003\t900000000000011006\t900000000000451002\n"
        );
        let rows: Vec<RelationshipConcreteValue> = read_all(data.as_bytes()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].value, ConcreteValue::Number("1.50".to_string()));
        assert_eq!(rows[0].relationship_group, 2);
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

    struct FailingRead;

    impl Read for FailingRead {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("synthetic read failure"))
        }
    }

    impl BufRead for FailingRead {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Err(io::Error::other("synthetic read failure"))
        }

        fn consume(&mut self, _amt: usize) {}
    }

    #[test]
    fn reports_an_io_error_after_a_valid_header() {
        let mut direct = FailingRead;
        let mut empty = [];
        let _ = direct.read(&mut empty).unwrap_err();

        let header = format!("{HEADER}\n");
        let reader = Cursor::new(header.into_bytes()).chain(FailingRead);
        let mut rows = Rf2Reader::<_, Concept>::new(reader).unwrap();
        let error = rows.next().unwrap().unwrap_err();
        assert!(matches!(error, Rf2Error::Io(_)));
    }

    #[test]
    fn reports_an_io_error_while_reading_the_header() {
        let error = match Rf2Reader::<_, Concept>::new(BufReader::new(FailingRead)) {
            Ok(_) => panic!("a failing reader unexpectedly produced an RF2 reader"),
            Err(error) => error,
        };
        assert!(matches!(error, Rf2Error::Io(_)));
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
