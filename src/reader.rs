use std::{
    io::{
        self, 
        BufRead, 
        BufReader, 
        Read,
    }, 
    iter,
};
use num_complex::Complex;

use super::*;

pub enum MtxReader<R: Read> {
    Matrix(MatrixReader<R>),
    // note, this is an enum to extend for an extended format
}

impl<R: Read> MtxReader<R> {
    pub fn new_reader(reader: R) -> Result<Self, Error> {
        let mut reader = BufReader::new(reader).lines().peekable();
        // note: .mtx is an ascii format so ascii methods are being used
        // note: .mtx is case insensitve as a whole
        let header_line = reader.next().ok_or(Error::NoHeader)??.to_ascii_lowercase();
        let mut header_line_iter = header_line.split_ascii_whitespace();
        if header_line_iter.next().ok_or(Error::MalformedHeader)? != "%%matrixmarket" {return Err(Error::MalformedHeader)}
        let object = header_line_iter.next().ok_or(Error::MalformedHeader)?;
        let format = header_line_iter.next().ok_or(Error::MalformedHeader)?;
        // skipping through comments
        loop {
            let Some(line) = reader.peek() else {return Err(Error::InsufficientContent)};
            let Ok(line) = line else {
                let e = reader.next().unwrap().unwrap_err();
                return Err(Error::IoError(e));
            };
            if !line.starts_with('%') {
                break;
            } else {
                let _ = reader.next().unwrap();
            }
        }
        // processing the qualifiers and header data in the content
        // note: this is set up like this to be potentially extended for an extended version of .mtx
        match (object, format) {
            ("matrix", "array") | ("matrix", "coordinate") => {
                let field = header_line_iter.next().ok_or(Error::MalformedHeader)?;
                let symmetry = header_line_iter.next().ok_or(Error::MalformedHeader)?;
                // there are no other qualifiers but I see no reason to make an error of that
                let content_header_line = reader.next().ok_or(Error::InsufficientContent)??.to_ascii_lowercase(); 
                let mut content_header_line_iter = content_header_line.split_ascii_whitespace(); 
                let num_rows = content_header_line_iter.next().ok_or(Error::MalformerContentHeader)?.parse::<usize>()?;
                let num_cols = content_header_line_iter.next().ok_or(Error::MalformerContentHeader)?.parse::<usize>()?;
                match symmetry {
                    "general" => {},
                    "symmetric" | "skew-symmetric" | "hermitian" => if num_cols != num_rows {return Err(Error::NotSquare)},
                    _ => return Err(Error::UnsupportedHeaderOptions),
                }
                Ok(match format {
                    "array" => {
                        MtxReader::Matrix(MatrixReader::MatrixArray(match (field, symmetry) {
                            ("real", "general") => {
                                MatrixArrayReader::Real(ParamatrizedMatrixArrayReader::General(GeneralMatrixArrayReader::internal_new(reader, num_rows, num_cols)))
                            },
                            ("integer", "general") => {
                                MatrixArrayReader::Integer(ParamatrizedMatrixArrayReader::General(GeneralMatrixArrayReader::internal_new(reader, num_rows, num_cols)))
                            },
                            ("complex", "general") => {
                                MatrixArrayReader::Complex(ParamatrizedMatrixArrayReader::General(GeneralMatrixArrayReader::internal_new(reader, num_rows, num_cols)))
                            },
                            ("real", "symmetric") => {
                                MatrixArrayReader::Real(ParamatrizedMatrixArrayReader::LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader::internal_new(reader, num_cols, std::clone::Clone::clone)))
                            },
                            ("integer", "symmetric") => {
                                MatrixArrayReader::Integer(ParamatrizedMatrixArrayReader::LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader::internal_new(reader, num_cols, std::clone::Clone::clone)))
                            },
                            ("complex", "symmetric") => {
                                MatrixArrayReader::Complex(ParamatrizedMatrixArrayReader::LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader::internal_new(reader, num_cols, std::clone::Clone::clone)))
                            },
                            ("real", "skew-symmetric") => {
                                MatrixArrayReader::Real(ParamatrizedMatrixArrayReader::LowerTriangleDiagonalExclusive(LowerTriExcMatrixArrayReader::internal_new(reader, num_cols, Field::inverse, Field::zero)))
                            },
                            ("integer", "skew-symmetric") => {
                                MatrixArrayReader::Integer(ParamatrizedMatrixArrayReader::LowerTriangleDiagonalExclusive(LowerTriExcMatrixArrayReader::internal_new(reader, num_cols, Field::inverse, Field::zero)))
                            },
                            ("complex", "skew-symmetric") => {
                                MatrixArrayReader::Complex(ParamatrizedMatrixArrayReader::LowerTriangleDiagonalExclusive(LowerTriExcMatrixArrayReader::internal_new(reader, num_cols, Field::inverse, Field::zero)))
                            },
                            ("complex", "hermitian") => {
                                MatrixArrayReader::Complex(ParamatrizedMatrixArrayReader::LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader::internal_new(reader, num_cols, Field::conjugate)))
                            },
                            _ => return Err(Error::UnsupportedHeaderOptions),
                        }))
                    }
                    "coordinate" => {
                        let num_to_read = content_header_line_iter.next().ok_or(Error::MalformerContentHeader)?.parse::<usize>()?;
                        MtxReader::Matrix(MatrixReader::MatrixCoordinate(match (field, symmetry) {
                            ("real", "general") => {
                                MatrixCoordinateReader::Real(ParamatrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, None))
                            }
                            ("integer", "general") => {
                                MatrixCoordinateReader::Integer(ParamatrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, None))
                            }
                            ("complex", "general") => {
                                MatrixCoordinateReader::Complex(ParamatrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, None))
                            }
                            ("real", "symmetric") => {
                                MatrixCoordinateReader::Real(ParamatrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(std::clone::Clone::clone)))
                            }
                            ("integer", "symmetric") => {
                                MatrixCoordinateReader::Integer(ParamatrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(std::clone::Clone::clone)))
                            }
                            ("complex", "symmetric") => {
                                MatrixCoordinateReader::Complex(ParamatrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(std::clone::Clone::clone)))
                            }
                            ("real", "skew-symmetric") => {
                                MatrixCoordinateReader::Real(ParamatrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(Field::inverse)))
                            }
                            ("integer", "skew-symmetric") => {
                                MatrixCoordinateReader::Integer(ParamatrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(Field::inverse)))
                            }
                            ("complex", "skew-symmetric") => {
                                MatrixCoordinateReader::Complex(ParamatrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(Field::inverse)))
                            }
                            ("complex", "hermitian") => {
                                MatrixCoordinateReader::Complex(ParamatrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(Field::conjugate)))
                            }
                            ("pattern", "general") => {
                                MatrixCoordinateReader::Pattern(ParamatrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, None))
                            }
                            ("pattern", "symmetric") => {
                                MatrixCoordinateReader::Pattern(ParamatrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(std::clone::Clone::clone)))
                            }
                            _ => return Err(Error:: UnsupportedHeaderOptions),
                        }))
                    }
                    _ => panic!("Internal Error (format not matched)")
                })
            }
            _ => return Err(Error::UnsupportedHeaderOptions),
        }
    }

    pub fn is_matrix(&self) -> bool {
        #[allow(unreachable_patterns)]
        match self {
            Self::Matrix(..) => true,
            _ => false,
        }
    }

    pub fn matrix(self) -> Option<MatrixReader<R>> {
        #[allow(unreachable_patterns)]
        match self {
            Self::Matrix(inner) => Some(inner),
            _ => None,
        }
    }
}

pub enum MatrixReader<R: Read> {
    MatrixArray(MatrixArrayReader<R>),
    MatrixCoordinate(MatrixCoordinateReader<R>),
}

impl<R: Read> MatrixReader<R> {
    pub fn is_array(&self) -> bool {
        match self {
            Self::MatrixArray(..) => true,
            _ => false,
        }
    }
    
    pub fn is_coordinate(&self) -> bool {
        match self {
            Self::MatrixCoordinate(..) => true,
            _ => false,
        }
    }
    
    pub fn array(self) -> Option<MatrixArrayReader<R>> {
        match self {
            Self::MatrixArray(inner) => Some(inner),
            _ => None,
        }
    }
    
    pub fn coordinate(self) -> Option<MatrixCoordinateReader<R>> {
        match self {
            Self::MatrixCoordinate(inner) => Some(inner),
            _ => None,
        }
    }
}

pub enum MatrixArrayReader<R: Read> {
    Real(ParamatrizedMatrixArrayReader<R, f64>),
    Integer(ParamatrizedMatrixArrayReader<R, i64>),
    Complex(ParamatrizedMatrixArrayReader<R, Complex<f64>>),
    // note: Pattern is not a valid field for the array format
}

impl<R: Read> Iterator for MatrixArrayReader<R> {
    type Item = Result<MatrixArrayColumn, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Real(inner) => inner.next().map(|col| col.map(|col| MatrixArrayColumn::Real(col))),
            Self::Integer(inner) => inner.next().map(|col| col.map(|col| MatrixArrayColumn::Integer(col))),
            Self::Complex(inner) => inner.next().map(|col| col.map(|col| MatrixArrayColumn::Complex(col))),
        }
    }
}

pub enum ParamatrizedMatrixArrayReader<R: Read, T: Field> {
    General(GeneralMatrixArrayReader<R, T>),
    LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader<R, T>),
    LowerTriangleDiagonalExclusive(LowerTriExcMatrixArrayReader<R, T>),
}

impl<R: Read, T: Field> Iterator for ParamatrizedMatrixArrayReader<R, T> {
    type Item = Result<ParamatrizedMatrixArrayColumn<T>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::General(inner) => inner.next(),
            Self::LowerTriangleDiagonalInclusive(inner) => inner.next(),
            Self::LowerTriangleDiagonalExclusive(inner) => inner.next(),
        }
    }
}

pub enum MatrixArrayColumn {
    Real(ParamatrizedMatrixArrayColumn<f64>),
    Integer(ParamatrizedMatrixArrayColumn<i64>),
    Complex(ParamatrizedMatrixArrayColumn<Complex<f64>>),
    // note: Pattern is not a valid field for the array format
}

impl Iterator for MatrixArrayColumn {
    type Item = FieldVal;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Real(inner) => inner.next().map(|v| FieldVal::Real(v)),
            Self::Integer(inner) => inner.next().map(|v| FieldVal::Integer(v)),
            Self::Complex(inner) => inner.next().map(|v| FieldVal::Complex(v)),
        }
    }
}

pub struct ParamatrizedMatrixArrayColumn<T: Field> {
    column: <Vec<T> as IntoIterator>::IntoIter,
}

impl<T: Field> Iterator for ParamatrizedMatrixArrayColumn<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.column.next()
    }
}

pub struct GeneralMatrixArrayReader<R: Read, T: Field> {
    reader: iter::Peekable<io::Lines<BufReader<R>>>,
    num_rows: usize,
    current_col: usize,
    num_cols: usize,
    field: std::marker::PhantomData<T>,
}

impl<R: Read, T: Field> GeneralMatrixArrayReader<R, T> {
    fn internal_new(reader: iter::Peekable<io::Lines<BufReader<R>>>, num_rows: usize, num_cols: usize) -> Self {
        Self {
            reader,
            num_rows,
            current_col: 0,
            num_cols,
            field: std::marker::PhantomData,
        }
    }
}

impl<R: Read, T: Field> Iterator for GeneralMatrixArrayReader<R, T> {
    type Item = Result<ParamatrizedMatrixArrayColumn<T>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_col < self.num_cols {
            self.current_col += 1;
            let mut vec = Vec::with_capacity(self.num_rows);
            for field_text in self.reader.by_ref().take(self.num_rows) {
                let field_text = match field_text {
                    Ok(field_text) => field_text,
                    Err(e) => return Some(Err(e.into())),
                };
                let field = match T::read(field_text.split_ascii_whitespace()) {
                    Ok(field) => field,
                    Err(e) => return Some(Err(e)),
                };
                vec.push(field);
            }
            if vec.len() < self.num_rows {
                Some(Err(Error::InsufficientContent))
            } else {
                Some(Ok(ParamatrizedMatrixArrayColumn {
                    column: vec.into_iter()
                }))
            }
        } else {
            None
        }
    }
}

/// AKA: LowerTriangleDiagonalInclusiveMatrixArrayReader
pub struct LowerTriIncMatrixArrayReader<R: Read, T: Field> {
    reader: iter::Peekable<io::Lines<BufReader<R>>>,
    size: usize, // known to be square
    current_col: usize,
    mirror: fn(&T) -> T,
    columns: Vec<Vec<T>>, // note: *reversed order* so that columns can be conveniently popped off
}

impl<R: Read, T: Field> LowerTriIncMatrixArrayReader<R, T> {
    fn internal_new(reader: iter::Peekable<io::Lines<BufReader<R>>>, size: usize, mirror: fn(&T) -> T) -> Self {
        let mut columns = Vec::with_capacity(size);
        for _ in 0..size {
            columns.push(Vec::with_capacity(size))
        }
        Self {
            reader,
            size,
            current_col: 0,
            mirror,
            columns: columns,
        }
    }
}

impl<R: Read, T: Field> Iterator for LowerTriIncMatrixArrayReader<R, T> {
    type Item = Result<ParamatrizedMatrixArrayColumn<T>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_col < self.size {
            self.current_col += 1;
            let mut vec = self.columns.pop().unwrap();
            {
                let field_text = match self.reader.next() {
                    Some(Ok(field_text)) => field_text,
                    Some(Err(e)) => return Some(Err(e.into())),
                    None => return Some(Err(Error::InsufficientContent)),
                };
                let field = match T::read(field_text.split_ascii_whitespace()) {
                    Ok(field) => field,
                    Err(e) => return Some(Err(e)),
                };
                vec.push(field);
            }
            for (field_text, other_col) in self.reader.by_ref().take(self.columns.len()).zip(self.columns.iter_mut().rev()) {
                let field_text = match field_text {
                    Ok(field_text) => field_text,
                    Err(e) => return Some(Err(e.into())),
                };
                let field = match T::read(field_text.split_ascii_whitespace()) {
                    Ok(field) => field,
                    Err(e) => return Some(Err(e)),
                };
                other_col.push((self.mirror)(&field));
                vec.push(field);
            }
            if vec.len() == self.size {
                Some(Ok(ParamatrizedMatrixArrayColumn {
                    column: vec.into_iter()
                }))
            } else {
                // likely because of some earlier error preventing a field from being added
                Some(Err(Error::GenericError))
            }
        } else {
            None
        }
    }
}

/// AKA: LowerTriangleDiagonalExclusiveMatrixArrayReader
pub struct LowerTriExcMatrixArrayReader<R: Read, T: Field> {
    reader: iter::Peekable<io::Lines<BufReader<R>>>,
    size: usize, // known to be square
    current_col: usize,
    mirror: fn(&T) -> T,
    diag: fn() -> T,
    columns: Vec<Vec<T>>, // note: *reversed order* so that columns can be conveniently popped off
}

impl<R: Read, T: Field> LowerTriExcMatrixArrayReader<R, T> {
    fn internal_new(reader: iter::Peekable<io::Lines<BufReader<R>>>, size: usize, mirror: fn(&T) -> T, diag: fn() -> T) -> Self {
        let mut columns = Vec::with_capacity(size);
        for _ in 0..size {
            columns.push(Vec::with_capacity(size))
        }
        Self {
            reader,
            size,
            current_col: 0,
            mirror,
            diag,
            columns: columns,
        }
    }
}

impl<R: Read, T: Field> Iterator for LowerTriExcMatrixArrayReader<R, T> {
    type Item = Result<ParamatrizedMatrixArrayColumn<T>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_col < self.size {
            self.current_col += 1;
            let mut vec = self.columns.pop().unwrap();
            vec.push((self.diag)());
            for (field_text, other_col) in self.reader.by_ref().take(self.columns.len()).zip(self.columns.iter_mut().rev()) {
                let field_text = match field_text {
                    Ok(field_text) => field_text,
                    Err(e) => return Some(Err(e.into())),
                };
                let field = match T::read(field_text.split_ascii_whitespace()) {
                    Ok(field) => field,
                    Err(e) => return Some(Err(e)),
                };
                other_col.push((self.mirror)(&field));
                vec.push(field);
            }
            if vec.len() == self.size {
                Some(Ok(ParamatrizedMatrixArrayColumn {
                    column: vec.into_iter()
                }))
            } else {
                // likely because of some earlier error preventing a field from being added
                Some(Err(Error::GenericError))
            }
        } else {
            None
        }
    }
}

pub enum MatrixCoordinateReader<R: Read> {
    Real(ParamatrizedMatrixCoordinateReader<R, f64>),
    Integer(ParamatrizedMatrixCoordinateReader<R, i64>),
    Complex(ParamatrizedMatrixCoordinateReader<R, Complex<f64>>),
    Pattern(ParamatrizedMatrixCoordinateReader<R, Pattern>),
}

impl<R: Read> Iterator for MatrixCoordinateReader<R> {
    type Item = Result<(Position, FieldVal), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Real(inner) => inner.next().map(|v| v.map(|v| (v.0, FieldVal::Real(v.1)))),
            Self::Integer(inner) => inner.next().map(|v| v.map(|v| (v.0, FieldVal::Integer(v.1)))),
            Self::Complex(inner) => inner.next().map(|v| v.map(|v| (v.0, FieldVal::Complex(v.1)))),
            Self::Pattern(inner) => inner.next().map(|v| v.map(|v| (v.0, FieldVal::Pattern(v.1)))),
        }
    }
}


pub struct ParamatrizedMatrixCoordinateReader<R: Read, T: Field> {
    reader: iter::Peekable<io::Lines<BufReader<R>>>,
    num_rows: usize,
    num_cols: usize,
    num_left: usize,
    mirror: Option<fn(&T) -> T>,
    buffer: Option<(Position, T)>,
}

impl<R: Read, T: Field> ParamatrizedMatrixCoordinateReader<R, T> {
    fn internal_new(reader: iter::Peekable<io::Lines<BufReader<R>>>, num_rows: usize, num_cols: usize, num_left: usize, mirror: Option<fn(&T) -> T>) -> Self {
        Self {
            reader,
            num_rows,
            num_cols,
            num_left,
            mirror,
            buffer: None,
        }
    }
}

impl<R: Read, T: Field> Iterator for ParamatrizedMatrixCoordinateReader<R, T> {
    type Item = Result<(Position, T), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_some() {
            return Some(Ok(self.buffer.take().unwrap()))
        }
        if self.num_left > 0 {
            self.num_left -= 1;
            let data_line = match self.reader.next() {
                Some(Ok(data_line)) => data_line,
                Some(Err(e)) => return Some(Err(e.into())),
                None => return Some(Err(Error::InsufficientContent)),
            };
            let mut data_line_iter = data_line.split_ascii_whitespace();
            let row = match data_line_iter.next().map(|x| x.parse::<usize>()) {
                Some(Ok(row)) => row -1, // 0-indexing
                Some(Err(e)) => return Some(Err(e.into())),
                None => return Some(Err(Error::InsufficientContent)),
            };
            if row > self.num_rows {return Some(Err(Error::MalformedContent))}
            let col = match data_line_iter.next().map(|x| x.parse::<usize>()) {
                Some(Ok(col)) => col -1, // 0-indexing
                Some(Err(e)) => return Some(Err(e.into())),
                None => return Some(Err(Error::InsufficientContent)),
            };
            if col > self.num_cols {return Some(Err(Error::MalformedContent))}
            let field = match T::read(data_line_iter) {
                Ok(field) => field,
                Err(e) => return Some(Err(e))
            };
            if let Some(mirror) = self.mirror {
                // note: no correction for vals on diag for skew-symmetric
                if row != col {
                    self.buffer = Some((Position{row: col, col: row}, mirror(&field)));
                }
            }
            Some(Ok((Position{row, col}, field)))
        } else {
            None
        }
    }
}
