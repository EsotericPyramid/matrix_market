//! a module of all the Reader structs & their methods.
//! 
//! It is what is says on the tin. The most important item in this is easily [`MtxReader`]

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

/// A flexible reader for an .mtx file
/// 
/// This reader can read any .mtx file regardless of object, format, and other qualifiers.
/// However, since the reader can contain very different forms of data, it has to be
/// downcast into its variants until it is specific enough for proper use
/// 
/// this enum's variants specify the object of the .mtx file
/// 
/// array format example:
/// ```
/// use matrix_market::reader::{
///     MtxReader,
///     MatrixArrayReader,
/// };
/// 
/// let mtx_text = "\
///     %%MatrixMarket matrix array integer general\n\
///     % A 3x3 dense matrix\n\
///     3 3\n\
///     1\n\
///     2\n\
///     3\n\
///     4\n\
///     5\n\
///     6\n\
///     7\n\
///     8\n\
///     9"
/// ;
///
/// let reader = MtxReader::new_reader(mtx_text.as_bytes()).unwrap();
/// let MatrixArrayReader::Integer(matrix_array_reader) = reader.matrix().unwrap().array().unwrap() else {panic!("Incorrect kind of reader")};
///
/// let mut matrix = vec![vec![0; 3]; 3];
/// for (col_idx, column) in matrix_array_reader.enumerate() {
///     for (row_idx, field) in column.unwrap().enumerate() {
///         matrix[row_idx][col_idx] = field;
///     }
/// }
/// 
/// println!("{:?}", matrix);
/// ```
/// 
/// coordinate format example:
/// ```
/// use matrix_market::reader::{
///     MtxReader,
///     MatrixCoordinateReader,
/// };
/// use matrix_market::Position;
/// 
/// let mtx_text = "\
///     %%MatrixMarket matrix coordinate integer general\n\
///     % A 4x4 Sparse matrix\n\
///     4 4 7\n\
///     1 1 1\n\
///     3 3 2\n\
///     2 2 3\n\
///     4 4 4\n\
///     3 2 5\n\
///     1 3 6\n\
///     2 4 7"
/// ;
/// 
/// let reader = MtxReader::new_reader(mtx_text.as_bytes()).unwrap();
/// let MatrixCoordinateReader::Integer(coord_reader) = reader.matrix().unwrap().coordinate().unwrap() else {panic!("Incorrect kind of reader")};
///
/// let mut matrix = vec![vec![0; 4]; 4];
/// for (Position { row, col }, field) in coord_reader.map(|x| x.unwrap()) {
///     matrix[row][col] = field;
/// }
/// 
/// println!("{:?}", matrix);
/// ```
pub enum MtxReader<R: Read> {
    /// A Reader of a `matrix` object
    Matrix(MatrixReader<R>),
    // note, this is an enum to extend for an extended format
}

impl<R: Read> MtxReader<R> {
    /// create a new [`MtxReader`] from a reader
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
                                MatrixArrayReader::Real(ParametrizedMatrixArrayReader::General(GeneralMatrixArrayReader::internal_new(reader, num_rows, num_cols)))
                            },
                            ("integer", "general") => {
                                MatrixArrayReader::Integer(ParametrizedMatrixArrayReader::General(GeneralMatrixArrayReader::internal_new(reader, num_rows, num_cols)))
                            },
                            ("complex", "general") => {
                                MatrixArrayReader::Complex(ParametrizedMatrixArrayReader::General(GeneralMatrixArrayReader::internal_new(reader, num_rows, num_cols)))
                            },
                            ("real", "symmetric") => {
                                MatrixArrayReader::Real(ParametrizedMatrixArrayReader::LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader::internal_new(reader, num_cols, std::clone::Clone::clone)))
                            },
                            ("integer", "symmetric") => {
                                MatrixArrayReader::Integer(ParametrizedMatrixArrayReader::LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader::internal_new(reader, num_cols, std::clone::Clone::clone)))
                            },
                            ("complex", "symmetric") => {
                                MatrixArrayReader::Complex(ParametrizedMatrixArrayReader::LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader::internal_new(reader, num_cols, std::clone::Clone::clone)))
                            },
                            ("real", "skew-symmetric") => {
                                MatrixArrayReader::Real(ParametrizedMatrixArrayReader::LowerTriangleDiagonalExclusive(LowerTriExcMatrixArrayReader::internal_new(reader, num_cols, Field::inverse, Field::zero)))
                            },
                            ("integer", "skew-symmetric") => {
                                MatrixArrayReader::Integer(ParametrizedMatrixArrayReader::LowerTriangleDiagonalExclusive(LowerTriExcMatrixArrayReader::internal_new(reader, num_cols, Field::inverse, Field::zero)))
                            },
                            ("complex", "skew-symmetric") => {
                                MatrixArrayReader::Complex(ParametrizedMatrixArrayReader::LowerTriangleDiagonalExclusive(LowerTriExcMatrixArrayReader::internal_new(reader, num_cols, Field::inverse, Field::zero)))
                            },
                            ("complex", "hermitian") => {
                                MatrixArrayReader::Complex(ParametrizedMatrixArrayReader::LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader::internal_new(reader, num_cols, Field::conjugate)))
                            },
                            _ => return Err(Error::UnsupportedHeaderOptions),
                        }))
                    }
                    "coordinate" => {
                        let num_to_read = content_header_line_iter.next().ok_or(Error::MalformerContentHeader)?.parse::<usize>()?;
                        MtxReader::Matrix(MatrixReader::MatrixCoordinate(match (field, symmetry) {
                            ("real", "general") => {
                                MatrixCoordinateReader::Real(ParametrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, None))
                            }
                            ("integer", "general") => {
                                MatrixCoordinateReader::Integer(ParametrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, None))
                            }
                            ("complex", "general") => {
                                MatrixCoordinateReader::Complex(ParametrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, None))
                            }
                            ("real", "symmetric") => {
                                MatrixCoordinateReader::Real(ParametrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(std::clone::Clone::clone)))
                            }
                            ("integer", "symmetric") => {
                                MatrixCoordinateReader::Integer(ParametrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(std::clone::Clone::clone)))
                            }
                            ("complex", "symmetric") => {
                                MatrixCoordinateReader::Complex(ParametrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(std::clone::Clone::clone)))
                            }
                            ("real", "skew-symmetric") => {
                                MatrixCoordinateReader::Real(ParametrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(Field::inverse)))
                            }
                            ("integer", "skew-symmetric") => {
                                MatrixCoordinateReader::Integer(ParametrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(Field::inverse)))
                            }
                            ("complex", "skew-symmetric") => {
                                MatrixCoordinateReader::Complex(ParametrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(Field::inverse)))
                            }
                            ("complex", "hermitian") => {
                                MatrixCoordinateReader::Complex(ParametrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(Field::conjugate)))
                            }
                            ("pattern", "general") => {
                                MatrixCoordinateReader::Pattern(ParametrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, None))
                            }
                            ("pattern", "symmetric") => {
                                MatrixCoordinateReader::Pattern(ParametrizedMatrixCoordinateReader::internal_new(reader, num_rows, num_cols, num_to_read, Some(std::clone::Clone::clone)))
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

    /// is this reader's object a matrix? (ie. is this reading a matrix)
    /// 
    /// see [`MtxReader::matrix`] to extract a matrix reader
    pub fn is_matrix(&self) -> bool {
        #[allow(unreachable_patterns)]
        match self {
            Self::Matrix(..) => true,
            _ => false,
        }
    }

    /// returns a MatrixReader if this reader's object is a matrix
    /// 
    /// see [`MtxReader::is_matrix`] to see if its object is a matrix 
    pub fn matrix(self) -> Option<MatrixReader<R>> {
        #[allow(unreachable_patterns)]
        match self {
            Self::Matrix(inner) => Some(inner),
            _ => None,
        }
    }
}

/// A flexible reader for an .mtx file storing a matrix
/// 
/// This reader can read any .mtx file containing a matrix regardless of format, field, and symmetry.
/// However, since the reader can contain very different forms of data, it has to be
/// downcast into its variants until it is specific enough for proper use
/// 
/// see [`MtxReader`] for an example
/// 
/// the variants of this type specify the format of the .mtx file
pub enum MatrixReader<R: Read> {
    /// the matrix is Dense and will be read out as a 2d array
    MatrixArray(MatrixArrayReader<R>),
    /// the matrix is Sparse and will be read out as a list of coordinates with non-zero values
    MatrixCoordinate(MatrixCoordinateReader<R>),
}

impl<R: Read> MatrixReader<R> {
    /// is the format of this matrix array (ie. a dense matrix)?
    /// 
    /// see [`MatrixReader::array`] to extract a reader of a matrix array
    pub fn is_array(&self) -> bool {
        match self {
            Self::MatrixArray(..) => true,
            _ => false,
        }
    }
    
    /// is the format of this matrix coordinate (ie. a sparse matrix)?
    /// 
    /// see [`MatrixReader::coordinate`] to extract a reader of a matrix coordinate
    pub fn is_coordinate(&self) -> bool {
        match self {
            Self::MatrixCoordinate(..) => true,
            _ => false,
        }
    }
    
    /// extract a reader of a matrix formatted as an array (ie. a dense matrix)?
    /// 
    /// see [`MatrixReader::is_array`] to check if it is formatted as such
    pub fn array(self) -> Option<MatrixArrayReader<R>> {
        match self {
            Self::MatrixArray(inner) => Some(inner),
            _ => None,
        }
    }
    
    /// extract a reader of a matrix formatted as coordinate (ie. a sparse matrix)?
    /// 
    /// see [`MatrixReader::is_coordinate`] to check if it is formatted as such
    pub fn coordinate(self) -> Option<MatrixCoordinateReader<R>> {
        match self {
            Self::MatrixCoordinate(inner) => Some(inner),
            _ => None,
        }
    }
}

/// A flexible reader for an .mtx file storing a matrix formatted as an array (ie. a dense matrix)
/// 
/// This reader can read any .mtx file containing a matrix formatted as an array regardless of field, and symmetry.
/// It is specific enough to be used directly but I'd recommend downcasting into one of its variants
/// first to specify its field (basically numerical type).
/// 
/// To use this enum, simply iterate through it to get *columns* of the whole matrix.
/// Since the field isn't specified, they will be enums based on the field
/// 
/// Note: Pattern is not a valid field for the array format
/// 
/// see [`MtxReader`] for an example
pub enum MatrixArrayReader<R: Read> {
    Real(ParametrizedMatrixArrayReader<R, f64>),
    Integer(ParametrizedMatrixArrayReader<R, i64>),
    Complex(ParametrizedMatrixArrayReader<R, Complex<f64>>),
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

/// A flexible reader for an .mtx file storing a matrix formatted as an array (ie. a dense matrix) with the field `T`
/// 
/// This reader can read any such .mtx file regardless of symmetry.
/// This is specific enough for practical use and, although it has variants to downcast into, 
/// I'd reccomend not doing so as their APIs are identical to this one's
/// 
/// To use this enum, simply iterate through it to get *columns* of the whole matrix
/// 
/// see [`MtxReader`] for an example
pub enum ParametrizedMatrixArrayReader<R: Read, T: Field> {
    /// A reader for a matrix with general symmetry (ie. no symmetry)
    General(GeneralMatrixArrayReader<R, T>),
    /// A reader for a matrix with symmetry along the diagonal whose diagonal also needs to be specfied
    /// 
    /// used for [`Symmetry::Symmetric`] and [`Symmetry::Hermitian`]
    LowerTriangleDiagonalInclusive(LowerTriIncMatrixArrayReader<R, T>),
    /// A reader for a matrix with symmetry along the diagonal whose diagonal is omitted
    /// 
    /// used for [`Symmetry::SkewSymmetric`]
    LowerTriangleDiagonalExclusive(LowerTriExcMatrixArrayReader<R, T>),
}

impl<R: Read, T: Field> Iterator for ParametrizedMatrixArrayReader<R, T> {
    type Item = Result<ParametrizedMatrixArrayColumn<T>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::General(inner) => inner.next(),
            Self::LowerTriangleDiagonalInclusive(inner) => inner.next(),
            Self::LowerTriangleDiagonalExclusive(inner) => inner.next(),
        }
    }
}

/// an iterator through the contents of a column of a matrix (top to bottom) returned by the iteration of [`MatrixArrayReader`]
/// 
/// This specific enum has variants of different fields which can be downcasted
/// specify the use of such a field, however, I'd recommend doing so either at the 
/// [`MatrixArrayReader`] level or at the [`FieldVal`] level as those will likely be more convient
pub enum MatrixArrayColumn {
    Real(ParametrizedMatrixArrayColumn<f64>),
    Integer(ParametrizedMatrixArrayColumn<i64>),
    Complex(ParametrizedMatrixArrayColumn<Complex<f64>>),
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

/// an iterator through the contents of a column of a matrix (top to bottom) returned by the iteration of [`ParametrizedMatrixArrayReader`]
pub struct ParametrizedMatrixArrayColumn<T: Field> {
    column: <Vec<T> as IntoIterator>::IntoIter,
}

impl<T: Field> Iterator for ParametrizedMatrixArrayColumn<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.column.next()
    }
}

/// A struct which reads a .mtx file of a matrix formatted as an array with general symmetry
/// 
/// To use this enum, simply iterate through it to get *columns* of the whole matrix
/// 
/// using this struct directly isn't very necessary and [`ParametrizedMatrixArrayReader`] should probably be used instead
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
    type Item = Result<ParametrizedMatrixArrayColumn<T>, Error>;

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
                Some(Ok(ParametrizedMatrixArrayColumn {
                    column: vec.into_iter()
                }))
            }
        } else {
            None
        }
    }
}

/// A struct which reads a .mtx file of a matrix formatted as an array with some symmetry across the diagonal which also specifies the diagonal
/// AKA: LowerTriangleDiagonalInclusiveMatrixArrayReader
/// 
/// To use this enum, simply iterate through it to get *columns* of the whole matrix
/// 
/// using this struct directly isn't very necessary and [`ParametrizedMatrixArrayReader`] should probably be used instead
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
    type Item = Result<ParametrizedMatrixArrayColumn<T>, Error>;

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
                Some(Ok(ParametrizedMatrixArrayColumn {
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

/// A struct which reads a .mtx file of a matrix formatted as an array with some symmetry across the diagonal which also omits the diagonal
/// AKA: LowerTriangleDiagonalExclusiveMatrixArrayReader
/// 
/// To use this enum, simply iterate through it to get *columns* of the whole matrix
/// 
/// using this struct directly isn't very necessary and [`ParametrizedMatrixArrayReader`] should probably be used instead
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
    type Item = Result<ParametrizedMatrixArrayColumn<T>, Error>;

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
                Some(Ok(ParametrizedMatrixArrayColumn {
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

/// A flexible reader for an .mtx file of a matrix formatted as coordinate
/// 
/// This reader can read any .mtx file containing a matrix formatted as coordinate regardless of field and symmetry.
/// It is specific enough to be used directly but I'd recommend downcasting into one of its variants
/// first to specify its field (basically numerical type).
/// 
/// To use this enum, simply iterate through it to get values tagged with positions
/// 
/// see [`MtxReader`] for an example
pub enum MatrixCoordinateReader<R: Read> {
    Real(ParametrizedMatrixCoordinateReader<R, f64>),
    Integer(ParametrizedMatrixCoordinateReader<R, i64>),
    Complex(ParametrizedMatrixCoordinateReader<R, Complex<f64>>),
    Pattern(ParametrizedMatrixCoordinateReader<R, Pattern>),
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

/// A flexible reader for an .mtx file of a matrix formatted as coordinate
/// 
/// This reader can read any .mtx file containing a matrix formatted as coordinate regardless of symmetry.
/// 
/// To use this enum, simply iterate through it to get values tagged with positions
/// 
/// see [`MtxReader`] for an example
pub struct ParametrizedMatrixCoordinateReader<R: Read, T: Field> {
    reader: iter::Peekable<io::Lines<BufReader<R>>>,
    num_rows: usize,
    num_cols: usize,
    num_left: usize,
    mirror: Option<fn(&T) -> T>,
    buffer: Option<(Position, T)>,
}

impl<R: Read, T: Field> ParametrizedMatrixCoordinateReader<R, T> {
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

impl<R: Read, T: Field> Iterator for ParametrizedMatrixCoordinateReader<R, T> {
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
