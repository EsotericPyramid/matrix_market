A simple parsing library for the matrix market (.mtx) file format

Provides the structs MtxReader (and its variations) and MatrixWriter to 
read and write matrix market files as laid out in its original specification\*.
In this file format, you can store matrices, sparse or dense, in a human readable
format (pure ASCII only). 

This format supports:
- comments
- dense matrices
- sparse matrices
- real (f64), integer (i64), complex (based on f64), and pattern\*\* matrices
- optimizations for certain symmetries:
    - symmetric
    - skew-symmetric
    - hermitian (aka self-adjoint)

Currently, this library doesn't support any extensions to the original mtx format but
is designed such that it shouldn't be that hard to add them if needed

Example:
```rust
use matrix_market::reader::{
    MtxReader,
    MatrixCoordinateReader,
};
use matrix_market::Position;

let mtx_text = "\
    %%MatrixMarket matrix coordinate integer general\n\
    % A 4x4 Sparse matrix\n\
    4 4 7\n\
    1 1 1\n\
    3 3 2\n\
    2 2 3\n\
    4 4 4\n\
    3 2 5\n\
    1 3 6\n\
    2 4 7"
;

let reader = MtxReader::new_reader(mtx_text.as_bytes()).unwrap();
let MatrixCoordinateReader::Integer(coord_reader) = reader.matrix().unwrap().coordinate().unwrap() else {panic!("Incorrect kind of reader")};
let mut matrix = vec![vec![0; 4]; 4];
for (Position { row, col }, field) in coord_reader.map(|x| x.unwrap()) {
    matrix[row][col] = field;
}

println!("{:?}", matrix);
```

\*: This library checks against that *strictly*, little deviation is allowed in read files.
That specification can be found [here](https://math.nist.gov/MatrixMarket/formats.html)

\*\*: A pattern matrix is a kind of sparse matrix where each value specified in the matrix is non-zero (and all else is 0)