use crate::reader::{GenericFieldMatrixArrayReader, GenericFieldMatrixReader};

use super::*;
use super::reader::Reader;
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

    let reader = Reader::new_reader(&*buf).unwrap();
    let Reader::Matrix(GenericFieldMatrixReader::MatrixArray(GenericFieldMatrixArrayReader::Integer(matrix_array_reader)))= reader else {panic!("Incorrect kind of reader")};

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

    let reader = Reader::new_reader(mtx_text.as_bytes()).unwrap();
    let Reader::Matrix(GenericFieldMatrixReader::MatrixArray(GenericFieldMatrixArrayReader::Integer(matrix_array_reader)))= reader else {panic!("Incorrect kind of reader")};

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

    let reader = Reader::new_reader(&*buf).unwrap();
    let Reader::Matrix(GenericFieldMatrixReader::MatrixArray(GenericFieldMatrixArrayReader::Integer(matrix_array_reader)))= reader else {panic!("Incorrect kind of reader")};

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

    let reader = Reader::new_reader(&*buf).unwrap();
    let Reader::Matrix(GenericFieldMatrixReader::MatrixArray(GenericFieldMatrixArrayReader::Integer(matrix_array_reader)))= reader else {panic!("Incorrect kind of reader")};

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

    let reader = Reader::new_reader(&*buf).unwrap();
    let Reader::Matrix(GenericFieldMatrixReader::MatrixArray(GenericFieldMatrixArrayReader::Complex(matrix_array_reader)))= reader else {panic!("Incorrect kind of reader")};

    let mut processed_matrix = vec![vec![Complex{re: 0.0, im: 0.0}; 4]; 4];
    for (col_idx, column) in matrix_array_reader.enumerate() {
        for (row_idx, field) in column.unwrap().enumerate() {
            processed_matrix[row_idx][col_idx] = field;
        }
    }

    assert_eq!(matrix, processed_matrix);
}