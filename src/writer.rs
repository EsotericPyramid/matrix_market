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

    /// note: kinda iffy on the num_to_read input, may change in the future
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