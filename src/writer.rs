use std::io::{BufWriter, Write};
use super::*;

pub struct MatrixWriter<W: Write>{
    writer: BufWriter<W>,
    symmetry: Symmetry,
    num_rows: usize,
    num_cols: usize,
    comment: String,
}

impl<W: Write> MatrixWriter<W> {
    pub fn new(writer: W, num_rows: usize, num_cols: usize) -> Self {
        Self { 
            writer: BufWriter::new(writer), 
            symmetry: Symmetry::General, 
            num_rows, 
            num_cols, 
            comment: String::new(),
        }
    }

    pub fn add_symmetry(&mut self, symmetry: Symmetry) -> Result<(), Error>{
        if let Symmetry::General = symmetry {
            if self.num_rows != self.num_cols {
                return Err(Error::NotSquare)
            }    
        }
        self.symmetry = symmetry;
        Ok(())
    }

    // adds one line of comment (more if the comment contains '\n')
    pub fn add_comment<S: AsRef<str>>(&mut self, comment: S) -> Result<(), Error> {
        self.comment.push('\n');
        self.comment.push_str(comment.as_ref());
        Ok(())
    }

    // note: the Positions is ref'd so Rust doesn't complain bout a missing lifetime in the return type
    pub fn write_array<T: Field, F: FnMut(&Position) -> &T>(mut self, mut matrix_index: F) -> Result<(), Error> {
        if T::kind() == FieldKind::Pattern {
            return Err(Error::UnsupportedHeaderOptions);
        }
        if (self.symmetry == Symmetry::Hermitian) & (T::kind() != FieldKind::Complex) {
            return Err(Error::UnsupportedHeaderOptions);
        }
        self.writer.write_all(format!("%%MarketMatrix matrix array {} {}\n",  T::as_string(), self.symmetry.as_string()).as_bytes())?;
        for line in self.comment.lines() {
            self.writer.write_all(format!("%{}\n", line).as_bytes())?;
        }
        self.writer.write_all(format!("{} {}", self.num_rows, self.num_cols).as_bytes())?;
        
        match self.symmetry {
            Symmetry::General => {
                for row in 0..self.num_rows {
                    for col in 0..self.num_cols {
                        let position = Position { row, col };
                        let field_text = matrix_index(&position).write();
                        self.writer.write_all(format!("{} {} {}\n", row+1, col+1, field_text).as_bytes())?;
                    }
                }
            }
            Symmetry::Symmetric => {
                for row in 0..self.num_rows {
                    for col in row..self.num_cols {
                        let position = Position { row, col };
                        let field_text = matrix_index(&position).write();
                        self.writer.write_all(format!("{0} {1} {2}\n{1} {0} {2}\n", row+1, col+1, field_text).as_bytes())?;
                    }
                }
            }
            Symmetry::Hermitian => {
                for row in 0..self.num_rows {
                    for col in row..self.num_cols {
                        let position = Position { row, col };
                        let field = matrix_index(&position);
                        let mirrored_field = field.conjugate();
                        self.writer.write_all(format!("{0} {1} {2}\n{1} {0} {3}\n", row+1, col+1, field.write(), mirrored_field.write()).as_bytes())?;
                    }
                }
            }
            Symmetry::SkewSymmetric => {
                for row in 0..self.num_rows {
                    for col in row+1..self.num_cols {
                        let position = Position { row, col };
                        let field = matrix_index(&position);
                        let mirrored_field = field.inverse();
                        self.writer.write_all(format!("{0} {1} {2}\n{1} {0} {3}\n", row+1, col+1, field.write(), mirrored_field.write()).as_bytes())?;
                    }
                }
            }
        }
        self.writer.flush()?;
        Ok(())
    }

    pub fn write_coordinate<T: Field, I: Iterator<Item = (Position, T)>>(mut self, mut coord_iter: I, num_to_read: usize) -> Result<(), Error> {
        if (T::kind() == FieldKind::Pattern) & (self.symmetry != Symmetry::General) & (self.symmetry != Symmetry::Symmetric) {
            return Err(Error::UnsupportedHeaderOptions);
        }
        if (self.symmetry == Symmetry::Hermitian) & (T::kind() != FieldKind::Complex) {
            return Err(Error::UnsupportedHeaderOptions);
        }
        self.writer.write_all(format!("%%MarketMatrix matrix coordinate {} {}\n",  T::as_string(), self.symmetry.as_string()).as_bytes())?;
        for line in self.comment.lines() {
            self.writer.write_all(format!("%{}\n", line).as_bytes())?;
        }
        self.writer.write_all(format!("{} {} {}", self.num_rows, self.num_cols, num_to_read).as_bytes())?;
        
        for _ in 0..num_to_read {
            let (Position {row, col}, field) = coord_iter.next().ok_or(Error::InsufficientContent)?; // mildly odd error but its fine
            self.writer.write_all(format!("{} {} {}", row, col, field.write()).as_bytes())?;
        }

        self.writer.flush()?;
        Ok(())
    }
}