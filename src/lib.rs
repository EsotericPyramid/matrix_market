//! A simple parsing library for the matrix market (.mtx) file format
//! 
//! Provides the structs MtxReader (and its variations) and MatrixWriter to 
//! read and write matrix market files as laid out in its original specification.[^specification]
//! In this file format, you can store matrices, sparse or dense, in a human readable
//! format (pure ASCII only). 
//! 
//! This format supports:
//! - comments
//! - dense matrices
//! - sparse matrices
//! - real (f64), integer (i64), complex (based on f64), and pattern[^pattern] matrices
//! - optimizations for certain symmetries:
//!     - symmetric
//!     - skew-symmetric
//!     - hermitian (aka self-adjoint)
//! 
//! Currently, this library doesn't support any extensions to the original mtx format but
//! is designed such that it shouldn't be that hard to add them if needed
//! 
//! Example:
//! ```
//! use matrix_market::reader::{
//!     MtxReader,
//!     MatrixCoordinateReader,
//! };
//! use matrix_market::Position;
//! 
//! let mtx_text = "\
//!     %%MatrixMarket matrix coordinate integer general\n\
//!     % A 4x4 Sparse matrix\n\
//!     4 4 7\n\
//!     1 1 1\n\
//!     3 3 2\n\
//!     2 2 3\n\
//!     4 4 4\n\
//!     3 2 5\n\
//!     1 3 6\n\
//!     2 4 7"
//! ;
//! 
//! let reader = MtxReader::new_reader(mtx_text.as_bytes()).unwrap();
//! let MatrixCoordinateReader::Integer(coord_reader) = reader.matrix().unwrap().coordinate().unwrap() else {panic!("Incorrect kind of reader")};
//!
//! let mut matrix = vec![vec![0; 4]; 4];
//! for (Position { row, col }, field) in coord_reader.map(|x| x.unwrap()) {
//!     matrix[row][col] = field;
//! }
//! 
//! println!("{:?}", matrix);
//! ```
//! 
//! [^specification]: This library checks against that *strictly*, little deviation is allowed in read files.
//! That specification can be found [here](https://math.nist.gov/MatrixMarket/formats.html)
//! 
//! [^pattern]: A pattern matrix is a kind of sparse matrix where each value specified in the matrix is non-zero (and all else is 0)

use std::{
    io,  
    num::{
        ParseFloatError,
        ParseIntError,
    }
};
use num_complex::Complex;

pub mod reader;
pub mod writer;

#[cfg(test)]
mod tests;

/// A enum of all the various errors this library can produce
#[derive(Debug)]
pub enum Error {
    /// the header line of a .mtx file is absent
    NoHeader, 
    /// the header line of a .mtx file is incorrectly formated
    /// 
    /// The correct format of the header line is:
    /// `%%MatrixMarket object format [qualifier ...]`
    /// 
    /// see the [specification](https://math.nist.gov/MatrixMarket/formats.html) for specific values
    MalformedHeader,
    /// the header line of the actual content is incorrectly formated
    /// 
    /// for example the content header of a dense array must be `num_rows num_cols`
    MalformerContentHeader,
    /// additional values were expected from the reader/iterator but none were found
    /// 
    /// ie. a 4x4 dense matrix file only provides values for the first 8
    InsufficientContent,
    /// the content of an mtx file itself is illegal
    /// 
    /// ie. a 4x4 spare array which has a value at (100, 100)
    MalformedContent,
    /// the set of options in the header line is not supported
    /// 
    /// this can either be because one of the options itself is not recognized (ex. `vector`)
    /// or because the combination of two or more is illegal (ex. `array` with `pattern`)
    /// 
    /// check the [specifications](https://math.nist.gov/MatrixMarket/formats.html) for supported combos
    UnsupportedHeaderOptions,
    /// something has gone wrong but it isn't clear what exactly
    /// 
    /// this is likely the result of some previous error having downstream consequences
    GenericError,
    /// the matrix was expected to be square but wasn't
    /// 
    /// mostly from trying to apply a symmetry to a non-square matrix
    NotSquare,
    /// an error occurred when trying to parse a float
    FloatError(ParseFloatError),
    /// an error occurred when trying to parse a integer
    IntError(ParseIntError),
    /// an error occurred when trying to read/write an io Read/Write object
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoHeader => write!(f, "MTX Error: No Header"),
            Self::MalformedHeader => write!(f, "MTX Error: Malformed Header"),
            Self::MalformerContentHeader => write!(f, "MTX Error: Malformed Content Header"),
            Self::InsufficientContent => write!(f, "MTX Error: Insufficient Content"),
            Self::MalformedContent => write!(f, "MTX Error: Malformed Content"),
            Self::UnsupportedHeaderOptions => write!(f, "MTX Error: Unsupported Header Options"),
            Self::GenericError => write!(f, "MTX Error: Generic Error"),
            Self::NotSquare => write!(f, "MTX Error: Not Square"),
            Self::FloatError(e) => write!(f, "MTX Error: Float Error ({})", e),
            Self::IntError(e) => write!(f, "MTX Error: Integer Error ({})", e),
            Self::IoError(e) => write!(f, "MTX Error: IO Error ({})", e),
        }
    }
}

impl std::error::Error for Error {}

/// a struct describing the pattern field in the mtx format
/// 
/// represents the field in a matrix where each value is simply zero or non-zero
/// 
/// In the original mtx format, Pattern is only usable with sparse matrices
/// and so needed no actual data to be stored within itself but this does 
/// so that it can behave more like an actual field
#[derive(Clone, Copy, Debug)]
pub struct Pattern{pub is_non_zero: bool}

/// an enum of all the different fields supported by the mtx format
#[derive(Clone, Copy, Debug)]
pub enum FieldVal {
    /// real numbers (aka `R`)
    Real(f64),
    /// integer numbers (aka `Z`)
    Integer(i64),
    /// complex numbers (aka `C`) (those in the form `a + b * i` where `a` and `b` are real and `i = sqrt(-1)`)
    Complex(Complex<f64>),
    /// A field which is either zero or non-zero
    Pattern(Pattern), 
}

/// an enum of all the different *kinds* of fields supported by the mtx format
/// 
/// does not store any actual value, see [`FieldVal`] for that
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKind {
    /// real numbers (aka `R`)
    Real,
    /// integer numbers (aka `Z`)
    Integer,
    /// complex numbers (aka `C`) (those in the form `a + b * i` where `a` and `b` are real and `i = sqrt(-1)`)
    Complex,
    /// A field which is either zero or non-zero
    Pattern,
}

impl FieldKind {
    /// represents this field kind as it would appear in the header line
    fn as_string(&self) -> &'static str {
        match self {
            Self::Real => "real",
            Self::Integer => "integer",
            Self::Complex => "complex",
            Self::Pattern => "pattern",
        }
    }
}

/// a trait of all methods expected of fields by the mtx format
/// 
/// not all methods are expected to be called by all types (ex. `int.conjugate()`)
/// but are still implemented as reasonably as possible
pub trait Field: Sized + Clone {
    /// read an representation of the field from the iterator
    fn read<'a>(iter: impl Iterator<Item = &'a str>) -> Result<Self, Error>;
    /// write the representation of this field to the outputted `String`
    fn write(&self) -> String;
    /// inverses this field (ie. `x = -x`)
    fn inverse(&self) -> Self; // for skew-symmetric
    /// conjugates this field (ie. `1 + 2i => 1 - 2i` )
    fn conjugate(&self) -> Self; // for hermetian / conjugate transpose
    /// gets the 0 for this field
    fn zero() -> Self; // for skew-symmetric
    /// gets the corresponding [`FieldKind`] for this field
    fn kind() -> FieldKind; 
}

impl Field for f64 {
    fn read<'a>(mut iter: impl Iterator<Item = &'a str>) -> Result<Self, Error> {
        Ok(iter.next().ok_or(Error::InsufficientContent)?.parse::<f64>()?)
    }
    fn write(&self) -> String {
        format!("{:e}", self)
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
    fn kind() -> FieldKind {
        FieldKind::Real
    }
}

impl Field for i64 {
    fn read<'a>(mut iter: impl Iterator<Item = &'a str>) -> Result<Self, Error> {
        Ok(iter.next().ok_or(Error::InsufficientContent)?.parse::<i64>()?)
    }
    fn write(&self) -> String {
        format!("{}", self)
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
    fn kind() -> FieldKind {
        FieldKind::Integer
    }
}

impl Field for Complex<f64> {
    fn read<'a>(mut iter: impl Iterator<Item = &'a str>) -> Result<Self, Error> {
        let real = iter.next().ok_or(Error::InsufficientContent)?.parse::<f64>()?;
        let imaginary = iter.next().ok_or(Error::InsufficientContent)?.parse::<f64>()?;
        Ok(Complex { re: real, im: imaginary })
    }
    fn write(&self) -> String {
        format!("{:e} {:e}", self.re, self.im)
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
    fn kind() -> FieldKind {
        FieldKind::Complex
    }
}

impl Field for Pattern { // stand-in for Pattern
    fn read<'a>(_: impl Iterator<Item = &'a str>) -> Result<Self, Error> {
        // note: in the .mtx file, pattern have no representation since they are only used in coordinate where they just say that that coord is nonzero
        Ok(Pattern { is_non_zero: true })
    }
    fn write(&self) -> String {
        // note: in the .mtx file, pattern have no representation since they are only used in coordinate where they just say that that coord is nonzero
        String::new()
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
    fn kind() -> FieldKind {
        FieldKind::Pattern
    }
}

/// represents the kinds of symmetries a matrix can have (or not have)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symmetry {
    /// no symmetry
    General,
    /// symmetrical across its diagonal (ie. `m = transpose(m)`)
    Symmetric,
    /// a skew-symmetric matrix (ie. `m = -transpose(m)`)
    SkewSymmetric,
    /// a hermitian / self-adjoint matrix (ie. `m = conjugate(transpose(m))`)
    Hermitian,
}

impl Symmetry {
    /// represents this symmetry as it would appear in the header line
    fn as_string(&self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Symmetric => "symmetric",
            Self::SkewSymmetric => "skew-symmetric",
            Self::Hermitian => "hermitian",
        }
    }
}

/// a position in a matrix
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

/// the size of a matrix
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatrixSize {
    pub num_rows: usize,
    pub num_cols: usize,
}
