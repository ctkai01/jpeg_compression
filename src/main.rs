use array2d::Array2D;
use image::{DynamicImage, GenericImageView};
use std::f32::consts::PI;
struct Dct2Jpeg;

#[derive(Debug, PartialEq, Clone, Copy)]
enum QuantizationMatrix {
    LuminanceQuantizationMatrix,
    ChrominanceQuantizationMatrix,
}

impl QuantizationMatrix {
    fn get(self) -> [[u8; 8]; 8] {
        if self == QuantizationMatrix::ChrominanceQuantizationMatrix {
            [
                [17, 18, 24, 47, 99, 99, 99, 99],
                [18, 21, 26, 66, 99, 99, 99, 99],
                [24, 26, 56, 99, 99, 99, 99, 99],
                [47, 66, 99, 99, 99, 99, 99, 99],
                [99, 99, 99, 99, 99, 99, 99, 99],
                [99, 99, 99, 99, 99, 99, 99, 99],
                [99, 99, 99, 99, 99, 99, 99, 99],
                [99, 99, 99, 99, 99, 99, 99, 99],
            ]
            // [
            //     [5, 3, 4, 4, 4, 3, 5, 4],
            //     [4, 4, 5, 5, 5, 6, 7, 12],
            //     [8, 7, 7, 7, 7, 15, 11, 11],
            //     [9, 12, 13, 15, 18, 18, 17, 15],
            //     [20, 20, 20, 20, 20, 20, 20, 20],
            //     [20, 20, 20, 20, 20, 20, 20, 20],
            //     [20, 20, 20, 20, 20, 20, 20, 20],
            //     [20, 20, 20, 20, 20, 20, 20, 20],
            // ]
        } else {
            [
                [16, 11, 10, 16, 24, 40, 51, 61],
                [12, 12, 14, 19, 26, 58, 60, 55],
                [14, 13, 16, 24, 40, 57, 69, 56],
                [14, 17, 22, 29, 51, 87, 80, 62],
                [18, 22, 37, 56, 68, 109, 103, 77],
                [24, 35, 55, 64, 81, 104, 113, 92],
                [49, 64, 78, 87, 103, 121, 120, 101],
                [72, 92, 95, 98, 112, 100, 103, 99],
            ]
        }
    }
}

const WIDTH_SIZE: u8 = 8;
const HEIGHT_SIZE: u8 = 8;

impl Dct2Jpeg {
    fn forward(image: DynamicImage) {
        let images_block = Self::split_image_block(image);
        let mut dc_previous = 0.0;
        let mut stream_binary = String::new();
        for (index, image_block) in images_block.iter().enumerate() {
            let mut first_block = false;
            if index == 0 {
                first_block = true;
                let (binary, dc_previous_func) = Self::transform_binary(
                    *image_block,
                    first_block,
                    0.0,
                    QuantizationMatrix::ChrominanceQuantizationMatrix,
                );
                dc_previous = dc_previous_func;
                println!("Binary: {}", binary);
                stream_binary.push_str(binary.as_str());
            } else {
                let (binary, dc_previous_func) = Self::transform_binary(
                    *image_block,
                    first_block,
                    dc_previous,
                    QuantizationMatrix::ChrominanceQuantizationMatrix,
                );
                dc_previous = dc_previous_func;
                stream_binary.push_str(binary.as_str());

                // println!("Binary: {}", binary)
            }
        }
        println!("{}", stream_binary);
    }
    
    fn transform_binary(
        matrix: [[u8; 8]; 8],
        first_block: bool,
        dc_previous: f32,
        quantization_matrix: QuantizationMatrix,
    ) -> (String, f32) {
        let mut dct: [[f32; 8]; 8] = [
            [0.0; 8], [0.0; 8], [0.0; 8], [0.0; 8], [0.0; 8], [0.0; 8], [0.0; 8], [0.0; 8],
        ];
        // println!("{:?}", dct);

        // let i: u8;
        let mut ci: f32;
        let mut cj: f32;
        let mut dct1: f32;
        let mut sum: f32;

        for i in 0..WIDTH_SIZE as usize {
            for j in 0..HEIGHT_SIZE as usize {
                // println!("Num i: {:?}, Num j: {:?}", i, j)
                if i == 0 {
                    ci = (1 as f32) / (2 as f32).sqrt();
                } else {
                    ci = 1 as f32
                }

                if j == 0 {
                    cj = (1 as f32) / (2 as f32).sqrt();
                } else {
                    cj = 1 as f32
                }

                sum = 0.0;

                for k in 0..WIDTH_SIZE as usize {
                    for l in 0..HEIGHT_SIZE as usize {
                        dct1 = matrix[k][l] as f32
                            * (((2 * k + 1) * i) as f32 * PI / (2 as f32 * WIDTH_SIZE as f32))
                                .cos()
                            * (((2 * l + 1) * j) as f32 * PI / (2 as f32 * HEIGHT_SIZE as f32))
                                .cos();
                        sum = sum + dct1;
                    }
                }
                let result =
                    ((1_f32 / 4_f32) * ci * cj * sum) / quantization_matrix.get()[i][j] as f32;
                dct[i][j] = (result * 1.0).round() / 1.0;
            }
        }

        let zig_zag_arr = &zig_zag_matrix(dct);
        // println!("{:?}", zig_zag_arr);
        let binary = get_binary_stream_from_zig_zag_array(zig_zag_arr, dc_previous, first_block);
        let dc = zig_zag_arr[0];
        (binary, dc)
    }

    fn split_image_block(img: DynamicImage) -> Vec<[[u8; 8]; 8]> {
        let (w, h) = img.dimensions();
        let mut images_block = Vec::new();
        let mut img_buff = img.as_bytes().chunks(h as usize);
        let mut b = Vec::new();
        while let Some(arr) = img_buff.next() {
            b.push(arr)
        }
        let size = 8;
        let size_w = if w % size == 0 {
            (w / size) as usize
        } else {
            ((w / size) + 1) as usize
        };
        let size_h = if h % size == 0 {
            (h / size) as usize
        } else {
            ((h / size) + 1) as usize
        };
        println!("W: {} H: {}", size_w, size_h);
        for m in 1..size_h + 1 {
            let block = (m - 1) * (size as usize);
            for n in 1..size_w + 1 {
                let block_h = (n - 1) * (size as usize);
                let mut arr_split_8x8 = [[0; 8]; 8];
                let mut index_arr_split = 0;
                for i in block..block + (size as usize) {
                    let mut ele_arr = [0; 8];
                    let mut index_arr = 0;
                    for j in block_h..block_h + (size as usize) {
                        if j >= w as usize || i >= h as usize {
                            ele_arr[index_arr] = 0;
                        } else {
                            ele_arr[index_arr] = b[j][i];
                        }
                        index_arr += 1;
                    }
                    arr_split_8x8[index_arr_split] = ele_arr;
                    index_arr_split += 1;
                }
                images_block.push(arr_split_8x8);
                // println!("{:?}", arr_split_8x8);
                // Self::forward(arr_split_8x8, QuantizationMatrix::ChrominanceQuantizationMatrix);
            }
        }
        images_block
    }
}

fn zig_zag_matrix(matrix: [[f32; 8]; 8]) -> Vec<f32> {
    let mut row = 0;
    let mut col = 0;

    let mut vec_zig_zag: Vec<f32> = Vec::new();
    let mut row_inc = false;

    // print matrix of lower half zig-zag pattern
    const MN: u8 = 8;
    for len in 1..MN + 1 {
        for i in 0..len + 1 {
            //	console.log(arr[row][col]);
            vec_zig_zag.push(matrix[row][col]);

            if i + 1 == len {
                break;
            }
            // If row_increment value is true
            // increment row and decrement col
            // else decrement row and increment col
            if row_inc {
                row += 1;
                col -= 1;
            } else {
                row -= 1;
                col += 1;
            }
        }

        if len == MN {
            break;
        }

        // Update row or col value according
        // to the last increment
        if row_inc {
            row += 1;
            row_inc = false;
        } else {
            col += 1;
            row_inc = true;
        }
    }

    // Update the indexes of row and col variable
    if row == 0 {
        if col == 8 - 1 {
            row += 1;
        } else {
            col += 1;
        }

        row_inc = true;
    } else {
        if row == 8 - 1 {
            col += 1;
        } else {
            row += 1;
        }
        row_inc = false;
    }

    // Print the next half zig-zag pattern
    let max = 8 - 1;
    // while
    let mut len = max;
    let mut diag = max;
    while diag > 0 {
        if diag > MN {
            len = MN;
        } else {
            len = diag;
        }
        for i in 0..len {
            // console.log(arr[row][col]);
            vec_zig_zag.push(matrix[row][col]);
            if i + 1 == len {
                break;
            }

            // Update row or col value according
            // to the last increment
            if row_inc {
                row += 1;
                col -= 1;
            } else {
                row -= 1;
                col += 1;
            }
        }

        // Update the indexes of row and col variable
        if row == 0 || col == 8 - 1 {
            if col == 8 - 1 {
                row += 1;
            } else {
                col += 1;
            }

            row_inc = true;
        } else if col == 0 || row == 8 - 1 {
            if row == 8 - 1 {
                col += 1;
            } else {
                row += 1;
            }
            row_inc = false;
        }
        diag -= 1;
    }
    vec_zig_zag
}

////////// KEY PAIR
use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::{Hash, Hasher},
};

// See explanation (1).
trait KeyPair<A, B> {
    /// Obtains the first element of the pair.
    fn a(&self) -> &A;
    /// Obtains the second element of the pair.
    fn b(&self) -> &B;
}

// See explanation (2).
impl<'a, A, B> Borrow<dyn KeyPair<A, B> + 'a> for (A, B)
where
    A: Eq + Hash + 'a,
    B: Eq + Hash + 'a,
{
    fn borrow(&self) -> &(dyn KeyPair<A, B> + 'a) {
        self
    }
}

// See explanation (3).
impl<A: Hash, B: Hash> Hash for (dyn KeyPair<A, B> + '_) {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.a().hash(state);
        self.b().hash(state);
    }
}

impl<A: Eq, B: Eq> PartialEq for (dyn KeyPair<A, B> + '_) {
    fn eq(&self, other: &Self) -> bool {
        self.a() == other.a() && self.b() == other.b()
    }
}

impl<A: Eq, B: Eq> Eq for (dyn KeyPair<A, B> + '_) {}

// OP's Table struct
pub struct TableACHuffman<'a, A: Eq + Hash, B: Eq + Hash> {
    map: HashMap<(A, B), &'a str>,
}

impl<A: Eq + Hash, B: Eq + Hash> TableACHuffman<'_, A, B> {
    fn new() -> Self {
        TableACHuffman {
            map: HashMap::new(),
        }
    }

    fn get(&self, a: &A, b: &B) -> &str {
        *self.map.get(&(a, b) as &dyn KeyPair<A, B>).unwrap()
    }

    fn set(&mut self, a: A, b: B, v: &'static str) {
        self.map.insert((a, b), v);
    }
}

// Boring stuff below.

impl<A, B> KeyPair<A, B> for (A, B) {
    fn a(&self) -> &A {
        &self.0
    }
    fn b(&self) -> &B {
        &self.1
    }
}
impl<A, B> KeyPair<A, B> for (&A, &B) {
    fn a(&self) -> &A {
        self.0
    }
    fn b(&self) -> &B {
        self.1
    }
}

//----------------------------------------------------------------

#[derive(Eq, PartialEq, Hash, Debug)]
struct A(String);

#[derive(Eq, PartialEq, Hash, Debug)]
struct B(String);

fn transform_positive_negative_binary(binary: String) -> String {
    let mut result = String::new();
    for i in binary.chars() {
        if i == '0' {
            result += "1"
        } else {
            result += "0"
        }
    }
    result
}

fn get_binary_stream_from_zig_zag_array(
    matrix: &Vec<f32>,
    previous_dc: f32,
    first_block: bool,
) -> String {
    let mut table_ac_huffman = TableACHuffman::new();
    table_ac_huffman.set(A("0".to_string()), B("0".to_string()), "1010");
    table_ac_huffman.set(A("0".to_string()), B("1".to_string()), "00");
    table_ac_huffman.set(A("0".to_string()), B("2".to_string()), "01");
    table_ac_huffman.set(A("0".to_string()), B("3".to_string()), "100");
    table_ac_huffman.set(A("0".to_string()), B("4".to_string()), "1011");
    table_ac_huffman.set(A("0".to_string()), B("5".to_string()), "11010");
    table_ac_huffman.set(A("0".to_string()), B("6".to_string()), "1111000");
    table_ac_huffman.set(A("0".to_string()), B("7".to_string()), "11111000");
    table_ac_huffman.set(A("0".to_string()), B("8".to_string()), "1111110110");
    table_ac_huffman.set(A("0".to_string()), B("9".to_string()), "1111111110000010");
    table_ac_huffman.set(A("0".to_string()), B("10".to_string()), "1111111110000011");
    table_ac_huffman.set(A("1".to_string()), B("1".to_string()), "1100");
    table_ac_huffman.set(A("1".to_string()), B("2".to_string()), "11011");
    table_ac_huffman.set(A("1".to_string()), B("3".to_string()), "1111001");
    table_ac_huffman.set(A("1".to_string()), B("4".to_string()), "111110110");
    table_ac_huffman.set(A("1".to_string()), B("5".to_string()), "11111110110");
    table_ac_huffman.set(A("1".to_string()), B("6".to_string()), "1111111110000100");
    table_ac_huffman.set(A("1".to_string()), B("7".to_string()), "1111111110000101");
    table_ac_huffman.set(A("1".to_string()), B("8".to_string()), "1111111110000110");
    table_ac_huffman.set(A("1".to_string()), B("9".to_string()), "1111111110000111");
    table_ac_huffman.set(A("1".to_string()), B("10".to_string()), "1111111110001000");
    table_ac_huffman.set(A("2".to_string()), B("1".to_string()), "11100");
    table_ac_huffman.set(A("2".to_string()), B("2".to_string()), "11111001");
    table_ac_huffman.set(A("2".to_string()), B("3".to_string()), "1111110111");
    table_ac_huffman.set(A("2".to_string()), B("4".to_string()), "111111110100");
    table_ac_huffman.set(A("2".to_string()), B("5".to_string()), "1111111110001001");
    table_ac_huffman.set(A("2".to_string()), B("6".to_string()), "1111111110001010");
    table_ac_huffman.set(A("2".to_string()), B("7".to_string()), "1111111110001011");
    table_ac_huffman.set(A("2".to_string()), B("8".to_string()), "1111111110001100");
    table_ac_huffman.set(A("2".to_string()), B("9".to_string()), "1111111110001101");
    table_ac_huffman.set(A("2".to_string()), B("10".to_string()), "1111111110001110");
    table_ac_huffman.set(A("3".to_string()), B("1".to_string()), "111010");
    table_ac_huffman.set(A("3".to_string()), B("2".to_string()), "111110111");
    table_ac_huffman.set(A("3".to_string()), B("3".to_string()), "111111110101");
    table_ac_huffman.set(A("3".to_string()), B("4".to_string()), "1111111110001111");
    table_ac_huffman.set(A("3".to_string()), B("5".to_string()), "1111111110010000");
    table_ac_huffman.set(A("3".to_string()), B("6".to_string()), "1111111110010001");
    table_ac_huffman.set(A("3".to_string()), B("7".to_string()), "1111111110010010");
    table_ac_huffman.set(A("3".to_string()), B("8".to_string()), "1111111110010011");
    table_ac_huffman.set(A("3".to_string()), B("9".to_string()), "1111111110010100");
    table_ac_huffman.set(A("3".to_string()), B("10".to_string()), "1111111110010101");
    table_ac_huffman.set(A("4".to_string()), B("1".to_string()), "111011");
    table_ac_huffman.set(A("4".to_string()), B("2".to_string()), "1111111000");
    table_ac_huffman.set(A("4".to_string()), B("3".to_string()), "1111111110010110");
    table_ac_huffman.set(A("4".to_string()), B("4".to_string()), "1111111110010111");
    table_ac_huffman.set(A("4".to_string()), B("5".to_string()), "1111111110011000");
    table_ac_huffman.set(A("4".to_string()), B("6".to_string()), "1111111110011001");
    table_ac_huffman.set(A("4".to_string()), B("7".to_string()), "1111111110011010");
    table_ac_huffman.set(A("4".to_string()), B("8".to_string()), "1111111110011011");
    table_ac_huffman.set(A("4".to_string()), B("9".to_string()), "1111111110011100");
    table_ac_huffman.set(A("4".to_string()), B("10".to_string()), "1111111110011101");
    table_ac_huffman.set(A("5".to_string()), B("1".to_string()), "1111010");
    table_ac_huffman.set(A("5".to_string()), B("2".to_string()), "11111110111");
    table_ac_huffman.set(A("5".to_string()), B("3".to_string()), "1111111110011110");
    table_ac_huffman.set(A("5".to_string()), B("4".to_string()), "1111111110011111");
    table_ac_huffman.set(A("5".to_string()), B("5".to_string()), "1111111110100000");
    table_ac_huffman.set(A("5".to_string()), B("6".to_string()), "1111111110100001");
    table_ac_huffman.set(A("5".to_string()), B("7".to_string()), "1111111110100010");
    table_ac_huffman.set(A("5".to_string()), B("8".to_string()), "1111111110100011");
    table_ac_huffman.set(A("5".to_string()), B("9".to_string()), "1111111110100100");
    table_ac_huffman.set(A("5".to_string()), B("10".to_string()), "1111111110100101");

    table_ac_huffman.set(A("6".to_string()), B("1".to_string()), "1111011");
    table_ac_huffman.set(A("6".to_string()), B("2".to_string()), "111111110110");
    table_ac_huffman.set(A("6".to_string()), B("3".to_string()), "1111111110100110");
    table_ac_huffman.set(A("6".to_string()), B("4".to_string()), "1111111110100111");
    table_ac_huffman.set(A("6".to_string()), B("5".to_string()), "1111111110101000");
    table_ac_huffman.set(A("6".to_string()), B("6".to_string()), "1111111110101001");
    table_ac_huffman.set(A("6".to_string()), B("7".to_string()), "1111111110101010");
    table_ac_huffman.set(A("6".to_string()), B("8".to_string()), "1111111110101011");
    table_ac_huffman.set(A("6".to_string()), B("9".to_string()), "1111111110101100");
    table_ac_huffman.set(A("6".to_string()), B("10".to_string()), "1111111110101101");

    table_ac_huffman.set(A("7".to_string()), B("1".to_string()), "11111010");
    table_ac_huffman.set(A("7".to_string()), B("2".to_string()), "111111110111");
    table_ac_huffman.set(A("7".to_string()), B("3".to_string()), "1111111110101110");
    table_ac_huffman.set(A("7".to_string()), B("4".to_string()), "1111111110101111");
    table_ac_huffman.set(A("7".to_string()), B("5".to_string()), "1111111110110000");
    table_ac_huffman.set(A("7".to_string()), B("6".to_string()), "1111111110110001");
    table_ac_huffman.set(A("7".to_string()), B("7".to_string()), "1111111110110010");
    table_ac_huffman.set(A("7".to_string()), B("8".to_string()), "1111111110110011");
    table_ac_huffman.set(A("7".to_string()), B("9".to_string()), "1111111110110100");
    table_ac_huffman.set(A("7".to_string()), B("10".to_string()), "1111111110110101");

    table_ac_huffman.set(A("8".to_string()), B("1".to_string()), "111111000");
    table_ac_huffman.set(A("8".to_string()), B("2".to_string()), "111111111000000");
    table_ac_huffman.set(A("8".to_string()), B("3".to_string()), "1111111110110110");
    table_ac_huffman.set(A("8".to_string()), B("4".to_string()), "1111111110110111");
    table_ac_huffman.set(A("8".to_string()), B("5".to_string()), "1111111110111000");
    table_ac_huffman.set(A("8".to_string()), B("6".to_string()), "1111111110111001");
    table_ac_huffman.set(A("8".to_string()), B("7".to_string()), "1111111110111010");
    table_ac_huffman.set(A("8".to_string()), B("8".to_string()), "1111111110111011");
    table_ac_huffman.set(A("8".to_string()), B("9".to_string()), "1111111110111100");
    table_ac_huffman.set(A("8".to_string()), B("10".to_string()), "1111111110111101");

    table_ac_huffman.set(A("9".to_string()), B("1".to_string()), "111111001");
    table_ac_huffman.set(A("9".to_string()), B("2".to_string()), "1111111110111110");
    table_ac_huffman.set(A("9".to_string()), B("3".to_string()), "1111111110111111");
    table_ac_huffman.set(A("9".to_string()), B("4".to_string()), "1111111111000000");
    table_ac_huffman.set(A("9".to_string()), B("5".to_string()), "1111111111000001");
    table_ac_huffman.set(A("9".to_string()), B("6".to_string()), "1111111111000010");
    table_ac_huffman.set(A("9".to_string()), B("7".to_string()), "1111111111000011");
    table_ac_huffman.set(A("9".to_string()), B("8".to_string()), "1111111111000100");
    table_ac_huffman.set(A("9".to_string()), B("9".to_string()), "1111111111000101");
    table_ac_huffman.set(A("9".to_string()), B("10".to_string()), "1111111111000110");

    table_ac_huffman.set(A("10".to_string()), B("1".to_string()), "111111010");
    table_ac_huffman.set(A("10".to_string()), B("2".to_string()), "1111111111000111");
    table_ac_huffman.set(A("10".to_string()), B("3".to_string()), "1111111111001000");
    table_ac_huffman.set(A("10".to_string()), B("4".to_string()), "1111111111001001");
    table_ac_huffman.set(A("10".to_string()), B("5".to_string()), "1111111111001010");
    table_ac_huffman.set(A("10".to_string()), B("6".to_string()), "1111111111001011");
    table_ac_huffman.set(A("10".to_string()), B("7".to_string()), "1111111111001100");
    table_ac_huffman.set(A("10".to_string()), B("8".to_string()), "1111111111001101");
    table_ac_huffman.set(A("10".to_string()), B("9".to_string()), "1111111111001110");
    table_ac_huffman.set(A("10".to_string()), B("10".to_string()), "1111111111001111");

    table_ac_huffman.set(A("11".to_string()), B("1".to_string()), "1111111001");
    table_ac_huffman.set(A("11".to_string()), B("2".to_string()), "1111111111010000");
    table_ac_huffman.set(A("11".to_string()), B("3".to_string()), "1111111111010001");
    table_ac_huffman.set(A("11".to_string()), B("4".to_string()), "1111111111010010");
    table_ac_huffman.set(A("11".to_string()), B("5".to_string()), "1111111111010011");
    table_ac_huffman.set(A("11".to_string()), B("6".to_string()), "1111111111010100");
    table_ac_huffman.set(A("11".to_string()), B("7".to_string()), "1111111111010101");
    table_ac_huffman.set(A("11".to_string()), B("8".to_string()), "1111111111010110");
    table_ac_huffman.set(A("11".to_string()), B("9".to_string()), "1111111111010111");
    table_ac_huffman.set(A("11".to_string()), B("10".to_string()), "1111111111011000");

    table_ac_huffman.set(A("12".to_string()), B("1".to_string()), "1111111010");
    table_ac_huffman.set(A("12".to_string()), B("2".to_string()), "1111111111011001");
    table_ac_huffman.set(A("12".to_string()), B("3".to_string()), "1111111111011010");
    table_ac_huffman.set(A("12".to_string()), B("4".to_string()), "1111111111011011");
    table_ac_huffman.set(A("12".to_string()), B("5".to_string()), "1111111111011100");
    table_ac_huffman.set(A("12".to_string()), B("6".to_string()), "1111111111011101");
    table_ac_huffman.set(A("12".to_string()), B("7".to_string()), "1111111111011110");
    table_ac_huffman.set(A("12".to_string()), B("8".to_string()), "1111111111011111");
    table_ac_huffman.set(A("12".to_string()), B("9".to_string()), "1111111111100000");
    table_ac_huffman.set(A("12".to_string()), B("10".to_string()), "1111111111100001");

    table_ac_huffman.set(A("13".to_string()), B("1".to_string()), "11111111000");
    table_ac_huffman.set(A("13".to_string()), B("2".to_string()), "1111111111100010");
    table_ac_huffman.set(A("13".to_string()), B("3".to_string()), "1111111111100011");
    table_ac_huffman.set(A("13".to_string()), B("4".to_string()), "1111111111100100");
    table_ac_huffman.set(A("13".to_string()), B("5".to_string()), "1111111111100101");
    table_ac_huffman.set(A("13".to_string()), B("6".to_string()), "1111111111100110");
    table_ac_huffman.set(A("13".to_string()), B("7".to_string()), "1111111111100111");
    table_ac_huffman.set(A("13".to_string()), B("8".to_string()), "1111111111101000");
    table_ac_huffman.set(A("13".to_string()), B("9".to_string()), "1111111111101001");
    table_ac_huffman.set(A("13".to_string()), B("10".to_string()), "1111111111101010");

    table_ac_huffman.set(A("14".to_string()), B("1".to_string()), "1111111111101011");
    table_ac_huffman.set(A("14".to_string()), B("2".to_string()), "1111111111101100");
    table_ac_huffman.set(A("14".to_string()), B("3".to_string()), "1111111111101101");
    table_ac_huffman.set(A("14".to_string()), B("4".to_string()), "1111111111101110");
    table_ac_huffman.set(A("14".to_string()), B("5".to_string()), "1111111111101111");
    table_ac_huffman.set(A("14".to_string()), B("6".to_string()), "1111111111110000");
    table_ac_huffman.set(A("14".to_string()), B("7".to_string()), "1111111111110001");
    table_ac_huffman.set(A("14".to_string()), B("8".to_string()), "1111111111110010");
    table_ac_huffman.set(A("14".to_string()), B("9".to_string()), "1111111111110011");
    table_ac_huffman.set(A("14".to_string()), B("10".to_string()), "1111111111110100");

    table_ac_huffman.set(A("15".to_string()), B("1".to_string()), "1111111111110101");
    table_ac_huffman.set(A("15".to_string()), B("2".to_string()), "1111111111110110");
    table_ac_huffman.set(A("15".to_string()), B("3".to_string()), "1111111111110111");
    table_ac_huffman.set(A("15".to_string()), B("4".to_string()), "1111111111111000");
    table_ac_huffman.set(A("15".to_string()), B("5".to_string()), "1111111111111001");
    table_ac_huffman.set(A("15".to_string()), B("6".to_string()), "1111111111111010");
    table_ac_huffman.set(A("15".to_string()), B("7".to_string()), "1111111111111011");
    table_ac_huffman.set(A("15".to_string()), B("8".to_string()), "1111111111111100");
    table_ac_huffman.set(A("15".to_string()), B("9".to_string()), "1111111111111101");
    table_ac_huffman.set(A("15".to_string()), B("10".to_string()), "1111111111111110");
    table_ac_huffman.set(A("x".to_string()), B("x".to_string()), "11111111001");
    // let matrix = [
    //     85.0, 4.0, 3.0, 1.0, -3.0, 6.0, 2.0, -2.0, 2.0, 0.0, 1.0, 0.0, -1.0, -1.0, 2.0, -1.0, 1.0,
    //     -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.0, -0.0, -0.0, -0.0,
    //     -0.0, -0.0, -0.0, -0.0, -0.0, -0.0, 0.0, -0.0, 0.0, -0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    //     0.0, -0.0, 0.0, -0.0, 0.0, -0.0, -0.0, -0.0, -0.0, -0.0, -0.0, -0.0, 0.0, 0.0, 0.0,
    // ];

    let mut dc_huffman_luminance = HashMap::new();
    dc_huffman_luminance.insert(0, "00");
    dc_huffman_luminance.insert(1, "010");
    dc_huffman_luminance.insert(2, "011");
    dc_huffman_luminance.insert(3, "100");
    dc_huffman_luminance.insert(4, "101");
    dc_huffman_luminance.insert(5, "110");
    dc_huffman_luminance.insert(6, "1110");
    dc_huffman_luminance.insert(7, "11110");
    dc_huffman_luminance.insert(8, "111110");
    dc_huffman_luminance.insert(9, "1111110");
    dc_huffman_luminance.insert(10, "11111110");
    dc_huffman_luminance.insert(11, "111111110");

    let mut vec = Vec::new();
    let mut number_zero = 0;

    // println!("Maxix: {:?}", matrix);
    for i in 1..matrix.len() {
        if (matrix[i] as f32).abs() == 0.0 {
            number_zero += 1;
        } else {
            let mut value = matrix[i] as i32;
            if matrix[i] < 0.0 {
                let a = matrix[i] as i32;
                value = a.abs();
            }
            let number_bit = format!("{:b}", (value as u8)).len();

            let vec_item = ((number_zero, number_bit), matrix[i] as i8);
            vec.push(vec_item);
            number_zero = 0;
        }
    }
    vec.push(((0, 0), 0));
    // println!("{:?}", vec);

    let mut binary_item = String::from("");
    // Binary DC
    // expect: (number_bit, num)
    let mut dc_current = matrix[0];
    if !first_block {
        dc_current -= previous_dc;
    } 

    let number_bit = format!("{:b}", (dc_current as u8)).len();
    binary_item += dc_huffman_luminance.get(&(number_bit as i8).abs()).unwrap();

    if dc_current > 0.0 {
        binary_item += format!("{:b}", dc_current as i8).as_str();
    } else {
        binary_item +=
            transform_positive_negative_binary(format!("{:b}", (dc_current as i8).abs())).as_str();
    }

    // println!("Current DC: {}, pre: {}, before: {}", dc_current, previous_dc, matrix[0]);

    // Binary AC
    // expect: (zero_length, number_bit)(number)
    // println!("Vec: {:?}", vec);
    for item in vec {
        let a = A(format!("{}", (item.0 .0)));
        let b = B(format!("{}", (item.0 .1)));
        // println!("A: {:?}, B: {:?}",a,b);
        // if item.1 > 0 {
        //     println!(
        //         "{:?},   {:?}",
        //         table_ac_huffman.get(&a, &b),
        //         format!("{:b}", item.1).as_str()
        //     );
        // } else {
        //     println!(
        //         "{:?},   {:?}",
        //         table_ac_huffman.get(&a, &b),
        //         transform_positive_negative_binary(format!("{:b}", (item.1 as i8).abs())).as_str()
        //     );
        // }
    
        if a.0.parse::<i32>().unwrap() > 15 {
            binary_item += table_ac_huffman.get(&A("x".to_string()), &B("x".to_string()));

        } else {
        binary_item += table_ac_huffman.get(&a, &b);

        }

        if item != ((0, 0), 0) {
            if item.1 > 0 {
                binary_item += format!("{:b}", item.1).as_str();
            } else {
                binary_item +=
                    transform_positive_negative_binary(format!("{:b}", (item.1 as i8).abs()))
                        .as_str();
            }
            // binary_item += dc_huffman_luminance.get(&item.1.abs()).unwrap();
        }
        // println!("");
    }
    // println!("Binary: {}", binary_item);
    binary_item
}

fn main() {
    // let ma = 1
    // let matrix = [
    //     [144, 139, 149, 155, 153, 155, 155, 155],
    //     [151, 151, 151, 159, 156, 156, 156, 158],
    //     [151, 156, 160, 162, 159, 151, 151, 151],
    //     [158, 163, 161, 160, 160, 160, 160, 161],
    //     [158, 160, 161, 162, 160, 155, 155, 156],
    //     [161, 161, 161, 161, 160, 157, 157, 157],
    //     [162, 162, 161, 160, 161, 157, 157, 157],
    //     [162, 162, 161, 160, 163, 157, 158, 154],
    // ];

    let img = image::open("src/t.jpeg").unwrap().grayscale();
    // let img = image::open("src/dcode-image.png").unwrap().grayscale();
    Dct2Jpeg::forward(img);

    // let (w , h) = img.dimensions();
    // let mut img_buff = img.as_bytes().chunks(h as usize);

    // let mut b = Vec::new();
    // while let Some(arr) = img_buff.next() {
    //     b.push(arr)
    // }

    // let size = 8;

    // let size_w = if w % size == 0 {
    //    (w / size) as usize
    // } else {
    //     ((w / size) + 1) as usize
    // };

    // let size_h = if h % size == 0 {
    //     (h / size) as usize
    // } else {
    //     ((h / size) + 1) as usize
    // };

    // println!("W: {} H: {}", size_w, size_h);

    // for m in 1..size_h + 1 {
    //     let block = (m - 1) * (size as usize);
    //     for n in 1..size_w + 1 {
    //         let block_h = (n - 1) * (size as usize);
    //         let mut arr_split_8x8 = [[0; 8]; 8];
    //         let mut index_arr_split = 0;
    //         for i in block..block + (size as usize) {
    //             let mut ele_arr = [0; 8];
    //             let mut index_arr = 0;

    //             for j in block_h..block_h + (size as usize) {
    //                 if j >= w  as usize || i >= h as usize {
    //                     ele_arr[index_arr] = 0;
    //                 } else {
    //                     ele_arr[index_arr] = b[j][i];
    //                 }
    //                 index_arr += 1;
    //             }
    //             arr_split_8x8[index_arr_split] = ele_arr;
    //             index_arr_split += 1;
    //         }
    //         // println!("{:?}", arr_split_8x8);
    //         Dct2Jpeg::forward(arr_split_8x8, QuantizationMatrix::ChrominanceQuantizationMatrix);
    //         println!();
    //     }
    // }

    // PART 2
}
