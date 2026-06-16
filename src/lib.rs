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

#[derive(Debug)]
pub enum Error {
    NoHeader,
    MalformedHeader,
    MalformerContentHeader,
    InsufficientContent,
    MalformedContent,
    UnsupportedHeaderOptions,
    GenericError,
    NotSquare,
    AlreadyWritten,
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

#[derive(PartialEq, Eq)]
pub enum FieldKind {
    Real,
    Integer,
    Complex,
    Pattern,
}

pub trait Field: Sized + Clone + std::fmt::Debug {
    fn read<'a>(iter: impl Iterator<Item = &'a str>) -> Result<Self, Error>;
    fn write(&self) -> String;
    fn inverse(&self) -> Self; // for skew-symmetric
    fn conjugate(&self) -> Self; // for hermetian / conjugate transpose
    fn zero() -> Self; // for skew-symmetric
    fn as_string() -> &'static str;
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
    fn as_string() -> &'static str {
        "real"
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
    fn as_string() -> &'static str {
        "integer"
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
    fn as_string() -> &'static str {
        "complex"
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
    fn as_string() -> &'static str {
        "pattern"
    }
    fn kind() -> FieldKind {
        FieldKind::Pattern
    }
}

#[derive(PartialEq, Eq)]
pub enum Symmetry {
    General,
    Symmetric,
    SkewSymmetric,
    Hermitian, // aka, self-adjoint
}

impl Symmetry {
    fn as_string(&self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Symmetric => "symmetric",
            Self::SkewSymmetric => "skew-symmetric",
            Self::Hermitian => "hermitian",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}
