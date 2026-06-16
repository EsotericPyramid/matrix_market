//! a module of all the Writer structs & their methods.
//! 
//! It is what is says on the tin. Currently the list of writers is:
//! 1. MatrixWriter (both Array (dense) and Coordinate (sparse) matrices)

use std::io::{BufWriter, Write};
use super::*;

/// a struct to help with writing a matrix to a .mtx file
/// 
/// The usage roughly looks like this:
/// 1. create the writer with [`Self::new`]
/// 2. (optional) add a comment and or symmetry (through [`Self::add_comment`] and [`Self::add_symmetry`])
/// 3. consume the writer with [`Self::write_array`] or [`Self::write_coordinate`] based on the kind of matrix
pub struct MatrixWriter<W: Write>{
    writer: BufWriter<W>,
    symmetry: Symmetry,
    num_rows: usize,
    num_cols: usize,
    comment: String,
}

impl<W: Write> MatrixWriter<W> {
    /// create a new [`MatrixWriter`] with no symmetry[^general_default] and no comment
    /// 
    /// the writer is what will be writen to when this writer is eventually consumed.
    /// until this writer is consumed, nothing will be writen to that writer.
    /// 
    /// the writer is currently buffered internally (subject to change)
    /// 
    /// [^general_default]: "no symmetry" is actually a symmetry called General (aka. [`Symmetry::General`])
    pub fn new(writer: W, num_rows: usize, num_cols: usize) -> Self {
        Self { 
            writer: BufWriter::new(writer), 
            symmetry: Symmetry::General, 
            num_rows, 
            num_cols, 
            comment: String::new(),
        }
    }

    /// adds[^set] a symmetry to this [`MatrixWriter`]
    /// 
    /// this method must (and can only be) called before this writer is consumed to have effect.
    /// 
    /// the applied symmetry may not be compatible with whichever format and field combination that is
    /// eventually used. Importantly, <b>this is only checked in the consuming methods, not here</b>.
    /// 
    /// [^set]: it actually *sets* the symmetry of this writer to the given symmetry. 
    /// Since the writer is defaulted with [`Symmetry::General`], this is generally true.
    /// However this also means this method can be called multiple times to overwrite the symmetry if convenient
    pub fn add_symmetry(&mut self, symmetry: Symmetry) -> Result<(), Error>{
        if let Symmetry::General = symmetry {
            if self.num_rows != self.num_cols {
                return Err(Error::NotSquare)
            }    
        }
        self.symmetry = symmetry;
        Ok(())
    }

    /// adds one line of comment to be written with the matrix
    /// 
    /// This method can be called multiple times to add multiple separate lines of text.
    /// Additionally, if the comment string contains multiple lines, each line will correspondingly
    /// be on its own line.
    /// 
    /// This comment is placed right after the header line with each line preceded by a `%` like so:
    /// ```text
    /// %%MatrixMarket matrix coordinate integer general\n\
    /// % this is a comment line
    /// % another line
    /// %
    /// % blank comment line above ^^^
    /// %note: there isnt a space following the % by default, add it yourself
    /// % helpful for describing the data like this:
    /// % A 5x5 sparse matrix with 8 nonzeros\n\
    /// 5 5 8\n\
    /// 1 1 1
    /// 2 3 4
    /// ...
    /// ```
    pub fn add_comment<S: AsRef<str>>(&mut self, comment: S) -> Result<(), Error> {
        self.comment.push('\n');
        self.comment.push_str(comment.as_ref());
        Ok(())
    }

    /// consume this writer to write a matrix formatted as an array (ie. dense matrix)
    /// 
    /// the matrix_index closure should be along the lines of indexing into the matrix like: `|Position{row, col}| &matrix[col][row]`.
    /// this closure should be well behaved for any position within the matrix, although,
    /// based on an added symmetry, it may not actually be called at all positions[^more_closure].
    /// 
    /// Importantly, not every combination of symmetry and field is valid, check the [specification](https://math.nist.gov/MatrixMarket/formats.html) for supported combos
    /// 
    /// [^more_closure]: This reduces the amount it must be called but it also means that this writer <b>makes no attempt at validating symmetry 
    /// or getting close to it</b>. Additionally, although `F` is `FnMut`, the order of positions in which it is called
    /// shouldn't effect the result.
    pub fn write_array<'a, T: Field + 'a, F: FnMut(Position) -> &'a T>(mut self, mut matrix_index: F) -> Result<(), Error> {
        if T::kind() == FieldKind::Pattern {
            return Err(Error::UnsupportedHeaderOptions);
        }
        if (self.symmetry == Symmetry::Hermitian) & (T::kind() != FieldKind::Complex) {
            return Err(Error::UnsupportedHeaderOptions);
        }
        self.writer.write_all(format!("%%MatrixMarket matrix array {} {}\n",  T::kind().as_string(), self.symmetry.as_string()).as_bytes())?;
        for line in self.comment.lines() {
            self.writer.write_all(format!("%{}\n", line).as_bytes())?;
        }
        self.writer.write_all(format!("{} {}\n", self.num_rows, self.num_cols).as_bytes())?;
        
        match self.symmetry {
            Symmetry::General => {
                for col in 0..self.num_cols {
                    for row in 0..self.num_rows {
                        let position = Position { row, col };
                        let field_text = matrix_index(position).write();
                        self.writer.write_all(format!("{}\n", field_text).as_bytes())?;
                    }
                }
            }
            Symmetry::Symmetric | Symmetry::Hermitian=> {
                for col in 0..self.num_cols {
                    for row in col..self.num_rows {
                        let position = Position { row, col };
                        let field_text = matrix_index(position).write();
                        self.writer.write_all(format!("{}\n",field_text).as_bytes())?;
                    }
                }
            }
            Symmetry::SkewSymmetric => {
                for col in 0..self.num_cols {
                    for row in col+1..self.num_rows {
                        let position = Position { row, col };
                        let field_text = matrix_index(position).write();
                        self.writer.write_all(format!("{}\n",field_text).as_bytes())?;
                    }
                }
            }
        };

        self.writer.flush()?;
        Ok(())
    }

    /// consume this writer to write a matrix formatted as coordinate (ie. sparse matrix)
    /// 
    /// `coord_iter` is what is used to get all non-zero coordinates in the sparse matrix. 
    /// `num_to_read` is the number of coords to be written to file/read from `coord_iter`[^num_to_read].
    /// 
    /// Importantly, this writer <b>makes no attempt that the set of coords are correct/reasonable</b>.
    /// That means that there are no checks against duplicate coords (or symmetrical coords with a symmetry)
    /// or coords on the diagonal with [`Symmetry::SkewSymmetric`]. Additionally, not every combination of 
    /// symmetry and field is valid, check the [specification](https://math.nist.gov/MatrixMarket/formats.html) for supported combos
    /// 
    /// note: kinda iffy on the `num_to_read` input, may change in the future
    /// 
    /// [^num_to_read]: this is <b>not</b> the number of non-zero coords but very specifically the number to be read. 
    /// That is an important distinction for symmetries where one coord read can turn into two non-zero coords
    pub fn write_coordinate<'a, T: Field + 'a, I: Iterator<Item = &'a (Position, T)>>(mut self, mut coord_iter: I, num_to_read: usize) -> Result<(), Error> {
        if (T::kind() == FieldKind::Pattern) & (self.symmetry != Symmetry::General) & (self.symmetry != Symmetry::Symmetric) {
            return Err(Error::UnsupportedHeaderOptions);
        }
        if (self.symmetry == Symmetry::Hermitian) & (T::kind() != FieldKind::Complex) {
            return Err(Error::UnsupportedHeaderOptions);
        }
        self.writer.write_all(format!("%%MatrixMarket matrix coordinate {} {}\n",  T::kind().as_string(), self.symmetry.as_string()).as_bytes())?;
        for line in self.comment.lines() {
            self.writer.write_all(format!("%{}\n", line).as_bytes())?;
        }
        self.writer.write_all(format!("{} {} {}\n", self.num_rows, self.num_cols, num_to_read).as_bytes())?;
        
        for _ in 0..num_to_read {
            let (Position {row, col}, field) = coord_iter.next().ok_or(Error::InsufficientContent)?; // mildly odd error but its fine
            self.writer.write_all(format!("{} {} {}\n", row +1, col +1, field.write()).as_bytes())?; //make sure to 1-index these
        }

        self.writer.flush()?;
        Ok(())
    }
}
