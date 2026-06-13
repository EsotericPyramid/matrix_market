use std::{
    io::{
        self, 
        BufRead, 
        BufReader, 
        Read,
    }, iter, num::{
        ParseFloatError,
        ParseIntError,
    }
};
use num_complex::Complex;

pub enum Error {
    NoHeader,
    MalformedHeader,
    MalformerContentHeader,
    InsufficientContent,
    MalformedContent,
    UnsupportedHeaderOptions,
    GenericError,
    NotSquare,
    FloatError(ParseFloatError),
    IntError(ParseIntError),
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IoError(value)
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Error::FloatError(value)
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Error::IntError(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Pattern{pub is_non_zero: bool}

pub enum FieldVal {
    Real(f64),
    Integer(i64),
    Complex(Complex<f64>),
    Pattern(Pattern), 
}

pub enum FieldKind {
    Real(f64),
    Integer(i64),
    Complex(Complex<f64>),
    Pattern(bool), // technically only its presence is important
}

pub trait Field: Sized + Clone {
    fn read<'a>(iter: impl Iterator<Item = &'a str>) -> Result<Self, Error>;
    fn inverse(&self) -> Self; // for skew-symmetric
    fn conjugate(&self) -> Self; // for hermetian / conjugate transpose
    fn zero() -> Self; // for skew-symmetric
}

// to get function pointers
fn inverse<T: Field>(val: &T) -> T {
    val.inverse()
}

fn conjugate<T: Field>(val: &T) -> T {
    val.conjugate()
}

fn zero<T: Field>() -> T {
    T::zero()
}

impl Field for f64 {
    fn read<'a>(mut iter: impl Iterator<Item = &'a str>) -> Result<Self, Error> {
        Ok(iter.next().ok_or(Error::InsufficientContent)?.parse::<f64>()?)
    }
    fn inverse(&self) -> Self {
        -self
    }
    fn conjugate(&self) -> Self {
        *self
    }
    fn zero() -> Self {
        0.0
    }
}

impl Field for i64 {
    fn read<'a>(mut iter: impl Iterator<Item = &'a str>) -> Result<Self, Error> {
        Ok(iter.next().ok_or(Error::InsufficientContent)?.parse::<i64>()?)
    }
    fn inverse(&self) -> Self {
        -self
    }
    fn conjugate(&self) -> Self {
        *self
    }
    fn zero() -> Self {
        0
    }
}

impl Field for Complex<f64> {
    fn read<'a>(mut iter: impl Iterator<Item = &'a str>) -> Result<Self, Error> {
        let real = iter.next().ok_or(Error::InsufficientContent)?.parse::<f64>()?;
        let imaginary = iter.next().ok_or(Error::InsufficientContent)?.parse::<f64>()?;
        Ok(Complex { re: real, im: imaginary })
    }
    fn inverse(&self) -> Self {
        -self
    }
    fn conjugate(&self) -> Self {
        self.conj()
    }
    fn zero() -> Self {
        Complex { re: 0.0, im: 0.0 }
    }
}

impl Field for Pattern { // stand-in for Pattern
    fn read<'a>(_: impl Iterator<Item = &'a str>) -> Result<Self, Error> {
        // note: in the .mtx file, pattern have no representation since they are only used in coordinate where they just say that that coord is nonzero
        Ok(Pattern { is_non_zero: true })
    }
    fn inverse(&self) -> Self {
        self.clone() // -(non-zero) != 0, -(0) == 0
    }
    fn conjugate(&self) -> Self {
        self.clone()
    }
    fn zero() -> Self {
        Pattern { is_non_zero: false }
    }
}

pub enum Reader<R: Read> {
    Matrix(GenericFieldMatrixReader<R>),
    // note, this is an enum to extend for an extended format
}

impl<R: Read> Reader<R> {
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
                    "symmetric" | "skew_symmetric" | "hermitian" => if num_cols != num_rows {return Err(Error::NotSquare)},
                    _ => return Err(Error::UnsupportedHeaderOptions),
                }
                Ok(match format {
                    "array" => {
                        Reader::Matrix(GenericFieldMatrixReader::MatrixArray(match (field, symmetry) {
                            ("real", "general") => {
                                GenericFieldMatrixArrayReader::Real(MatrixArrayReader::General(GeneralMatrixArrayReader::internal_new(reader, num_rows, num_cols)))
                            },
                            ("integer", "general") => {
                                GenericFieldMatrixArrayReader::Integer(MatrixArrayReader::General(GeneralMatrixArrayReader::internal_new(reader, num_rows, num_cols)))
                            },
                            ("complex", "general") => {
                                GenericFieldMatrixArrayReader::Complex(MatrixArrayReader::General(GeneralMatrixArrayReader::internal_new(reader, num_rows, num_cols)))
                            },
                            ("real", "symmetric") => {
                                GenericFieldMatrixArrayReader::Real(MatrixArrayReader::LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader::internal_new(reader, num_cols, std::clone::Clone::clone)))
                            },
                            ("integer", "symmetric") => {
                                GenericFieldMatrixArrayReader::Integer(MatrixArrayReader::LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader::internal_new(reader, num_cols, std::clone::Clone::clone)))
                            },
                            ("complex", "symmetric") => {
                                GenericFieldMatrixArrayReader::Complex(MatrixArrayReader::LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader::internal_new(reader, num_cols, std::clone::Clone::clone)))
                            },
                            ("real", "skew-symmetric") => {
                                GenericFieldMatrixArrayReader::Real(MatrixArrayReader::LowerTriangleDiagonalExclusive(LowerTriExcMatrixArrayReader::internal_new(reader, num_cols, inverse, zero)))
                            },
                            ("integer", "skew-symmetric") => {
                                GenericFieldMatrixArrayReader::Integer(MatrixArrayReader::LowerTriangleDiagonalExclusive(LowerTriExcMatrixArrayReader::internal_new(reader, num_cols, inverse, zero)))
                            },
                            ("complex", "skew-symmetric") => {
                                GenericFieldMatrixArrayReader::Complex(MatrixArrayReader::LowerTriangleDiagonalExclusive(LowerTriExcMatrixArrayReader::internal_new(reader, num_cols, inverse, zero)))
                            },
                            ("complex", "hermitian") => {
                                GenericFieldMatrixArrayReader::Complex(MatrixArrayReader::LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader::internal_new(reader, num_cols, conjugate)))
                            },
                            _ => return Err(Error::UnsupportedHeaderOptions),
                        }))
                    }
                    "coordinate" => {
                        let num_to_read = content_header_line_iter.next().ok_or(Error::MalformerContentHeader)?.parse::<usize>()?;
                        Reader::Matrix(GenericFieldMatrixReader::MatrixCoordinate(match (field, symmetry) {
                            ("real", "general") => {
                                GenericFieldMatrixCoordinateReader::Real(MatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, None))
                            }
                            ("integer", "general") => {
                                GenericFieldMatrixCoordinateReader::Integer(MatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, None))
                            }
                            ("complex", "general") => {
                                GenericFieldMatrixCoordinateReader::Complex(MatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, None))
                            }
                            ("real", "symmetric") => {
                                GenericFieldMatrixCoordinateReader::Real(MatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(std::clone::Clone::clone)))
                            }
                            ("integer", "symmetric") => {
                                GenericFieldMatrixCoordinateReader::Integer(MatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(std::clone::Clone::clone)))
                            }
                            ("complex", "symmetric") => {
                                GenericFieldMatrixCoordinateReader::Complex(MatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(std::clone::Clone::clone)))
                            }
                            ("real", "skew-symmetric") => {
                                GenericFieldMatrixCoordinateReader::Real(MatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(inverse)))
                            }
                            ("integer", "skew-symmetric") => {
                                GenericFieldMatrixCoordinateReader::Integer(MatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(inverse)))
                            }
                            ("complex", "skew-symmetric") => {
                                GenericFieldMatrixCoordinateReader::Complex(MatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(inverse)))
                            }
                            ("complex", "hermitian") => {
                                GenericFieldMatrixCoordinateReader::Complex(MatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(conjugate)))
                            }
                            ("pattern", "general") => {
                                GenericFieldMatrixCoordinateReader::Pattern(MatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, None))
                            }
                            ("pattern", "symmetric") => {
                                GenericFieldMatrixCoordinateReader::Pattern(MatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(std::clone::Clone::clone)))
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
}

pub enum GenericFieldMatrixReader<R: Read> {
    MatrixArray(GenericFieldMatrixArrayReader<R>),
    MatrixCoordinate(GenericFieldMatrixCoordinateReader<R>),
}

pub enum GenericFieldMatrixArrayReader<R: Read> {
    Real(MatrixArrayReader<R, f64>),
    Integer(MatrixArrayReader<R, i64>),
    Complex(MatrixArrayReader<R, Complex<f64>>),
    // note: Pattern is not a valid field for the array format
}

impl<R: Read> Iterator for GenericFieldMatrixArrayReader<R> {
    type Item = Result<GenericFieldMatrixArrayColumn, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Real(inner) => inner.next().map(|col| col.map(|col| GenericFieldMatrixArrayColumn::Real(col))),
            Self::Integer(inner) => inner.next().map(|col| col.map(|col| GenericFieldMatrixArrayColumn::Integer(col))),
            Self::Complex(inner) => inner.next().map(|col| col.map(|col| GenericFieldMatrixArrayColumn::Complex(col))),
        }
    }
}

pub enum MatrixArrayReader<R: Read, T: Field> {
    General(GeneralMatrixArrayReader<R, T>),
    LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader<R, T>),
    LowerTriangleDiagonalExclusive(LowerTriExcMatrixArrayReader<R, T>),
}

impl<R: Read, T: Field> Iterator for MatrixArrayReader<R, T> {
    type Item = Result<MatrixArrayColumn<T>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::General(inner) => inner.next(),
            Self::LowerTriangleDiagonalInclusive(inner) => inner.next(),
            Self::LowerTriangleDiagonalExclusive(inner) => inner.next(),
        }
    }
}

pub enum GenericFieldMatrixArrayColumn {
    Real(MatrixArrayColumn<f64>),
    Integer(MatrixArrayColumn<i64>),
    Complex(MatrixArrayColumn<Complex<f64>>),
    // note: Pattern is not a valid field for the array format
}

impl Iterator for GenericFieldMatrixArrayColumn {
    type Item = FieldVal;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Real(inner) => inner.next().map(|v| FieldVal::Real(v)),
            Self::Integer(inner) => inner.next().map(|v| FieldVal::Integer(v)),
            Self::Complex(inner) => inner.next().map(|v| FieldVal::Complex(v)),
        }
    }
}

pub struct MatrixArrayColumn<T: Field> {
    column: <Vec<T> as IntoIterator>::IntoIter,
}

impl<T: Field> Iterator for MatrixArrayColumn<T> {
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
    type Item = Result<MatrixArrayColumn<T>, Error>;

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
                Some(Ok(MatrixArrayColumn {
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
    field: std::marker::PhantomData<T>,
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
            field: std::marker::PhantomData,
            mirror,
            columns: columns,
        }
    }
}

impl<R: Read, T: Field> Iterator for LowerTriIncMatrixArrayReader<R, T> {
    type Item = Result<MatrixArrayColumn<T>, Error>;

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
            for (field_text, other_col) in self.reader.by_ref().zip(self.columns.iter_mut().rev()) {
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
                Some(Ok(MatrixArrayColumn {
                    column: vec.into_iter()
                }))
            } else {
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
    field: std::marker::PhantomData<T>,
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
            field: std::marker::PhantomData,
            mirror,
            diag,
            columns: columns,
        }
    }
}

impl<R: Read, T: Field> Iterator for LowerTriExcMatrixArrayReader<R, T> {
    type Item = Result<MatrixArrayColumn<T>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_col < self.size {
            self.current_col += 1;
            let mut vec = self.columns.pop().unwrap();
            vec.push((self.diag)());
            for (field_text, other_col) in self.reader.by_ref().zip(self.columns.iter_mut().rev()) {
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
                Some(Ok(MatrixArrayColumn {
                    column: vec.into_iter()
                }))
            } else {
                Some(Err(Error::GenericError))
            }
        } else {
            None
        }
    }
}

pub enum GenericFieldMatrixCoordinateReader<R: Read> {
    Real(MatrixCoordinateReader<R, f64>),
    Integer(MatrixCoordinateReader<R, i64>),
    Complex(MatrixCoordinateReader<R, Complex<f64>>),
    Pattern(MatrixCoordinateReader<R, Pattern>),
}

impl<R: Read> Iterator for GenericFieldMatrixCoordinateReader<R> {
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

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

pub struct MatrixCoordinateReader<R: Read, T: Field> {
    reader: iter::Peekable<io::Lines<BufReader<R>>>,
    num_rows: usize,
    num_cols: usize,
    num_left: usize,
    field: std::marker::PhantomData<T>,
    mirror: Option<fn(&T) -> T>,
    buffer: Option<(Position, T)>,
}

impl<R: Read, T: Field> MatrixCoordinateReader<R, T> {
    fn internal_new(reader: iter::Peekable<io::Lines<BufReader<R>>>, num_rows: usize, num_cols: usize, num_left: usize, mirror: Option<fn(&T) -> T>) -> Self {
        Self {
            reader,
            num_rows,
            num_cols,
            num_left,
            field: std::marker::PhantomData,
            mirror,
            buffer: None,
        }
    }
}

impl<R: Read, T: Field> Iterator for MatrixCoordinateReader<R, T> {
    type Item = Result<(Position, T), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num_left > 0 {
            if self.buffer.is_some() {
                return Some(Ok(self.buffer.take().unwrap()))
            }
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
