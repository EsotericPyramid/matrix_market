use crate::reader::{MatrixArrayReader, MatrixCoordinateReader};

use super::*;
use super::reader::MtxReader;
use super::writer::MatrixWriter;

#[test]
fn matrix_array_write_test() {
    let matrix = vec![
        vec![1,  2,  3,  4],
        vec![5,  6,  7,  8],
        vec![9, 10, 11, 12],
    ];

    let mut buf = Vec::new();
    let writer = MatrixWriter::new(&mut buf, matrix.len(), matrix[0].len());
    writer.write_array(|Position { row, col }| &matrix[row][col]).unwrap();

    println!("As file:\n{}", String::from_utf8(buf.clone()).unwrap());

    let reader = MtxReader::new_reader(&*buf).unwrap();
    let MatrixArrayReader::Integer(matrix_array_reader) = reader.matrix().unwrap().array().unwrap() else {panic!("Incorrect kind of reader")};

    let mut processed_matrix = vec![vec![0; 4]; 3];
    for (col_idx, column) in matrix_array_reader.enumerate() {
        for (row_idx, field) in column.unwrap().enumerate() {
            processed_matrix[row_idx][col_idx] = field;
        }
    }

    assert_eq!(matrix, processed_matrix);
}

#[test]
fn matrix_array_read_test() {
    let mtx_text = "\
        %%MatrixMarket matrix array integer general\n\
        % A 4x3 dense matrix\n\
        4 3\n\
        1\n\
        2\n\
        3\n\
        4\n\
        5\n\
        6\n\
        7\n\
        8\n\
        9\n\
        10\n\
        11\n\
        12"
    ;

    let matrix = vec![
        vec![1, 5,  9],
        vec![2, 6, 10],
        vec![3, 7, 11],
        vec![4, 8, 12],
    ];

    let reader = MtxReader::new_reader(mtx_text.as_bytes()).unwrap();
    let MatrixArrayReader::Integer(matrix_array_reader) = reader.matrix().unwrap().array().unwrap() else {panic!("Incorrect kind of reader")};

    let mut processed_matrix = vec![vec![0; 3]; 4];
    for (col_idx, column) in matrix_array_reader.enumerate() {
        for (row_idx, field) in column.unwrap().enumerate() {
            processed_matrix[row_idx][col_idx] = field;
        }
    }

    assert_eq!(matrix, processed_matrix);
}

#[test]
fn matrix_array_symmetric_write_test() {
    let matrix = vec![
        vec![1, 2, 3, 4],
        vec![2, 5, 6, 7],
        vec![3, 6, 8, 9],
        vec![4, 7, 9, 0],
    ];

    let mut buf = Vec::new();
    let mut writer = MatrixWriter::new(&mut buf, matrix.len(), matrix[0].len());
    writer.add_symmetry(Symmetry::Symmetric).unwrap();
    writer.write_array(|Position { row, col }| &matrix[row][col]).unwrap();

    println!("As file:\n{}", String::from_utf8(buf.clone()).unwrap());

    let reader = MtxReader::new_reader(&*buf).unwrap();
    let MatrixArrayReader::Integer(matrix_array_reader) = reader.matrix().unwrap().array().unwrap() else {panic!("Incorrect kind of reader")};

    let mut processed_matrix = vec![vec![0; 4]; 4];
    for (col_idx, column) in matrix_array_reader.enumerate() {
        for (row_idx, field) in column.unwrap().enumerate() {
            processed_matrix[row_idx][col_idx] = field;
        }
    }

    assert_eq!(matrix, processed_matrix);
}

#[test]
fn matrix_array_symmetric_read_test() {
    let mtx_text = "\
        %%MatrixMarket matrix array integer symmetric\n\
        % A 4x4 dense matrix\n\
        4 4\n\
        1\n\
        2\n\
        3\n\
        4\n\
        5\n\
        6\n\
        7\n\
        8\n\
        9\n\
        0"
    ;

    let matrix = vec![
        vec![1, 2, 3, 4],
        vec![2, 5, 6, 7],
        vec![3, 6, 8, 9],
        vec![4, 7, 9, 0],
    ];

    let reader = MtxReader::new_reader(mtx_text.as_bytes()).unwrap();
    let MatrixArrayReader::Integer(matrix_array_reader) = reader.matrix().unwrap().array().unwrap() else {panic!("Incorrect kind of reader")};

    let mut processed_matrix = vec![vec![0; 4]; 4];
    for (col_idx, column) in matrix_array_reader.enumerate() {
        for (row_idx, field) in column.unwrap().enumerate() {
            processed_matrix[row_idx][col_idx] = field;
        }
    }

    assert_eq!(matrix, processed_matrix);
}

#[test]
fn matrix_array_skew_symmetric_write_test() {
    let matrix = vec![
        vec![0, -1, -2, -3],
        vec![1,  0, -4, -5],
        vec![2,  4,  0, -6],
        vec![3,  5,  6,  0],
    ];

    let mut buf = Vec::new();
    let mut writer = MatrixWriter::new(&mut buf, matrix.len(), matrix[0].len());
    writer.add_symmetry(Symmetry::SkewSymmetric).unwrap();
    writer.write_array(|Position { row, col }| &matrix[row][col]).unwrap();

    println!("As file:\n{}", String::from_utf8(buf.clone()).unwrap());

    let reader = MtxReader::new_reader(&*buf).unwrap();
    let MatrixArrayReader::Integer(matrix_array_reader) = reader.matrix().unwrap().array().unwrap() else {panic!("Incorrect kind of reader")};

    let mut processed_matrix = vec![vec![0; 4]; 4];
    for (col_idx, column) in matrix_array_reader.enumerate() {
        for (row_idx, field) in column.unwrap().enumerate() {
            processed_matrix[row_idx][col_idx] = field;
        }
    }

    assert_eq!(matrix, processed_matrix);
}

#[test]
fn matrix_array_skew_symmetric_read_test() {
    let mtx_text = "\
        %%MatrixMarket matrix array integer skew-symmetric\n\
        % A 4x4 dense matrix\n\
        4 4\n\
        1\n\
        2\n\
        3\n\
        4\n\
        5\n\
        6"
    ;

    let matrix = vec![
        vec![0, -1, -2, -3],
        vec![1,  0, -4, -5],
        vec![2,  4,  0, -6],
        vec![3,  5,  6,  0],
    ];

    let reader = MtxReader::new_reader(mtx_text.as_bytes()).unwrap();
    let MatrixArrayReader::Integer(matrix_array_reader) = reader.matrix().unwrap().array().unwrap() else {panic!("Incorrect kind of reader")};

    let mut processed_matrix = vec![vec![0; 4]; 4];
    for (col_idx, column) in matrix_array_reader.enumerate() {
        for (row_idx, field) in column.unwrap().enumerate() {
            processed_matrix[row_idx][col_idx] = field;
        }
    }

    assert_eq!(matrix, processed_matrix);
}

#[test]
fn matrix_array_hermitian_write_test() {
    let matrix = vec![
        vec![Complex{re: 1.0, im: 0.0}, Complex{re: 2.0, im: -1.0}, Complex{re: 3.0, im: -2.0}, Complex{re: 4.0, im: -3.0}],
        vec![Complex{re: 2.0, im: 1.0}, Complex{re: 5.0, im:  0.0}, Complex{re: 6.0, im: -4.0}, Complex{re: 7.0, im: -5.0}],
        vec![Complex{re: 3.0, im: 2.0}, Complex{re: 6.0, im:  4.0}, Complex{re: 8.0, im:  0.0}, Complex{re: 9.0, im: -6.0}],
        vec![Complex{re: 4.0, im: 3.0}, Complex{re: 7.0, im:  5.0}, Complex{re: 9.0, im:  6.0}, Complex{re: 0.0, im:  0.0}],
    ];

    let mut buf = Vec::new();
    let mut writer = MatrixWriter::new(&mut buf, matrix.len(), matrix[0].len());
    writer.add_symmetry(Symmetry::Hermitian).unwrap();
    writer.write_array(|Position { row, col }| &matrix[row][col]).unwrap();

    println!("As file:\n{}", String::from_utf8(buf.clone()).unwrap());

    let reader = MtxReader::new_reader(&*buf).unwrap();
    let MatrixArrayReader::Complex(matrix_array_reader) = reader.matrix().unwrap().array().unwrap() else {panic!("Incorrect kind of reader")};

    let mut processed_matrix = vec![vec![Complex{re: 0.0, im: 0.0}; 4]; 4];
    for (col_idx, column) in matrix_array_reader.enumerate() {
        for (row_idx, field) in column.unwrap().enumerate() {
            processed_matrix[row_idx][col_idx] = field;
        }
    }

    assert_eq!(matrix, processed_matrix);
}

#[test]
fn matrix_coord_write_test() {
    // matrix:
    // [
    //  [1, 0, 0, 0, 0],
    //  [0, 0, 6, 0, 0],
    //  [0, 0, 2, 7, 0],
    //  [4, 0, 0, 0, 0],
    //  [0, 5, 0, 0, 3],
    // ]
    let coords = vec![
        (Position{row: 0, col: 0}, 1),
        (Position{row: 2, col: 2}, 2),
        (Position{row: 4, col: 4}, 3),
        (Position{row: 3, col: 0}, 4),
        (Position{row: 4, col: 1}, 5),
        (Position{row: 1, col: 2}, 6),
        (Position{row: 2, col: 3}, 7),
    ];

    let mut buf = Vec::new();
    let writer = MatrixWriter::new(&mut buf, 5, 5);
    writer.write_coordinate(coords.iter(), coords.len()).unwrap();

    println!("As file:\n{}", String::from_utf8(buf.clone()).unwrap());

    let reader = MtxReader::new_reader(&*buf).unwrap();
    let MatrixCoordinateReader::Integer(coord_reader) = reader.matrix().unwrap().coordinate().unwrap() else {panic!("Incorrect kind of reader")};

    let mut processed_coords = coord_reader.map(|x| x.unwrap()).collect::<Vec<_>>();
    // note: as is, the order of coords isn't changed but that is potentionally subject to change sooooo...
    processed_coords.sort_unstable_by_key(|x| x.1);
    assert_eq!(coords, processed_coords);
}

#[test]
fn matrix_coord_read_test() {
    let mtx_text = "\
    %%MatrixMarket matrix coordinate integer general\n\
    % A 5x5 sparse matrix with 8 nonzeros\n\
    5 5 8\n\
    1 1 1\n\
    2 2 10\n\
    4 2 250\n\
    3 3 3\n\
    1 4 6\n\
    4 4 -280\n\
    4 5 33\n\
    5 5 12";

    let matrix = vec![
        vec![1,   0, 0,    6,  0],
        vec![0,  10, 0,    0,  0],
        vec![0,   0, 3,    0,  0],
        vec![0, 250, 0, -280, 33],
        vec![0,   0, 0,    0, 12],
    ];

    let reader = MtxReader::new_reader(mtx_text.as_bytes()).unwrap();
    let MatrixCoordinateReader::Integer(coord_reader) = reader.matrix().unwrap().coordinate().unwrap() else {panic!("Incorrect kind of reader")};

    let mut processed_matrix = vec![vec![0; 5]; 5];
    for (Position { row, col }, field) in coord_reader.map(|x| x.unwrap()) {
        processed_matrix[row][col] = field;
    }

    assert_eq!(matrix, processed_matrix);
}

#[test]
fn matrix_coord_symmetric_write_test() {
    // matrix:
    // [
    //  [1, 0, 0, 4, 0],
    //  [0, 0, 6, 0, 5],
    //  [0, 6, 2, 7, 0],
    //  [4, 0, 7, 0, 0],
    //  [0, 5, 0, 0, 3],
    // ]
    let mut coords = vec![
        (Position{row: 0, col: 0}, 1),
        (Position{row: 2, col: 2}, 2),
        (Position{row: 4, col: 4}, 3),
        (Position{row: 3, col: 0}, 4),
        (Position{row: 0, col: 3}, 4),
        (Position{row: 4, col: 1}, 5),
        (Position{row: 1, col: 4}, 5),
        (Position{row: 1, col: 2}, 6),
        (Position{row: 2, col: 1}, 6),
        (Position{row: 2, col: 3}, 7),
        (Position{row: 3, col: 2}, 7),
    ];

    let mut buf = Vec::new();
    let mut writer = MatrixWriter::new(&mut buf, 5, 5);
    writer.add_symmetry(Symmetry::Symmetric).unwrap();
    writer.write_coordinate(coords.iter().filter(|x| x.0.row >= x.0.col), 7).unwrap();

    println!("As file:\n{}", String::from_utf8(buf.clone()).unwrap());

    let reader = MtxReader::new_reader(&*buf).unwrap();
    let MatrixCoordinateReader::Integer(coord_reader) = reader.matrix().unwrap().coordinate().unwrap() else {panic!("Incorrect kind of reader")};

    let mut processed_coords = coord_reader.map(|x| x.unwrap()).collect::<Vec<_>>();
    // note: as is, the order of coords isn't changed but that is potentionally subject to change sooooo...
    processed_coords.sort_unstable_by_key(|x| x.0.col);
    processed_coords.sort_by_key(|x| x.0.row);
    coords.sort_unstable_by_key(|x| x.0.col);
    coords.sort_by_key(|x| x.0.row);
    assert_eq!(coords, processed_coords);
}

#[test]
fn matrix_coord_skew_symmetric_write_test() {
    // matrix:
    // [
    //  [0, -1, 0,  0,  0],
    //  [1,  0, 0,  2, -3],
    //  [0,  0, 0, -4,  0],
    //  [0, -2, 4,  0,  0],
    //  [0,  3, 0,  0,  0],
    // ]
    let mut coords = vec![
        (Position{row: 1, col: 0},  1),
        (Position{row: 0, col: 1}, -1),
        (Position{row: 3, col: 1}, -2),
        (Position{row: 1, col: 3},  2),
        (Position{row: 4, col: 1},  3),
        (Position{row: 1, col: 4}, -3),
        (Position{row: 3, col: 2},  4),
        (Position{row: 2, col: 3}, -4),
    ];

    let mut buf = Vec::new();
    let mut writer = MatrixWriter::new(&mut buf, 5, 5);
    writer.add_symmetry(Symmetry::SkewSymmetric).unwrap();
    writer.write_coordinate(coords.iter().filter(|x| x.0.row > x.0.col), 4).unwrap();

    println!("As file:\n{}", String::from_utf8(buf.clone()).unwrap());

    let reader = MtxReader::new_reader(&*buf).unwrap();
    let MatrixCoordinateReader::Integer(coord_reader) = reader.matrix().unwrap().coordinate().unwrap() else {panic!("Incorrect kind of reader")};

    let mut processed_coords = coord_reader.map(|x| x.unwrap()).collect::<Vec<_>>();
    // note: as is, the order of coords isn't changed but that is potentionally subject to change sooooo...
    processed_coords.sort_unstable_by_key(|x| x.0.col);
    processed_coords.sort_by_key(|x| x.0.row);
    coords.sort_unstable_by_key(|x| x.0.col);
    coords.sort_by_key(|x| x.0.row);
    assert_eq!(coords, processed_coords);
}