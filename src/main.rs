use std::f32::consts::PI;
use image::{DynamicImage, GenericImageView};
struct Dct2Jpeg;

#[derive(Debug, PartialEq, Clone, Copy)]
enum QuantizationMatrix {
    LuminanceQuantizationMatrix,
    ChrominanceQuantizationMatrix,
}

impl QuantizationMatrix {
    fn get(self) -> [[u8; 8]; 8] {
        if self == QuantizationMatrix::ChrominanceQuantizationMatrix {
            // [
            //     [17, 18, 24, 47, 99, 99, 99, 99],
            //     [18, 21, 26, 66, 99, 99, 99, 99],
            //     [24, 26, 56, 99, 99, 99, 99, 99],
            //     [47, 66, 99, 99, 99, 99, 99, 99],
            //     [99, 99, 99, 99, 99, 99, 99, 99],
            //     [99, 99, 99, 99, 99, 99, 99, 99],
            //     [99, 99, 99, 99, 99, 99, 99, 99],
            //     [99, 99, 99, 99, 99, 99, 99, 99],
            // ]
            [
                [5,3,4,4,4,3,5,4],
	[4,4,5,5,5,6,7,12],
	[8,7,7,7,7,15,11,11],
	[9,12,13,15,18,18,17,15],
	[20,20,20,20,20,20,20,20],
	[20,20,20,20,20,20,20,20],
	[20,20,20,20,20,20,20,20],
	[20,20,20,20,20,20,20,20]
            ]
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
    fn forward(matrix: [[u8; 8]; 8], quantization_matrix: QuantizationMatrix) {
        let mut dct: [[f32; 8]; 8] = [
            [0.0; 8], [0.0; 8], [0.0; 8], [0.0; 8], [0.0; 8], [0.0; 8], [0.0; 8], [0.0; 8],
        ];
        // println!("{:?}", dct);

        // let i: u8;
        let mut ci: f32;
        let mut cj: f32;
        let mut dct1: f32;
        let mut sum: f32;

        for i  in 0..WIDTH_SIZE as usize {
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
					    dct1 = matrix[k][l] as f32 *
                        (((2 * k + 1) * i) as f32 * PI / (2 as f32 * WIDTH_SIZE as f32)).cos() *
                        (((2 * l + 1) * j) as f32 * PI / (2 as f32 * HEIGHT_SIZE as f32)).cos();
                        
                        sum = sum + dct1;
                    }
                }
                // let a = quantization_matrix.get()[i][j] as u8;
                let result = ((1_f32/4_f32) * ci * cj * sum) / quantization_matrix.get()[i][j] as f32;
                dct[i][j] = (result * 1.0).round() / 1.0;
            }
        }

        for k in 0..WIDTH_SIZE as usize {
            for l in 0..HEIGHT_SIZE as usize {
                // dct1 = matrix[k][l] as f32 *
                // (((2 * k + 1) * i) as f32 * PI / (2 as f32 * WIDTH_SIZE as f32)).cos() *
                // (((2 * l + 1) * j) as f32 * PI / (2 as f32 * HEIGHT_SIZE as f32)).cos();
                print!("{:?} \t", dct[k][l])
                // sum = sum + dct1;
            }
            println!("")
        }
        let a = zigZagMatrix(dct);
        println!("{:?}", a)
    }
}

fn zigZagMatrix(matrix: [[f32; 8]; 8]) -> Vec<f32> {
    let mut row = 0;
	let mut col = 0;

	let mut vecZigzag: Vec<f32> = Vec::new();
	let mut row_inc = false;

	// print matrix of lower half zig-zag pattern
	const MN: u8 = 8;
	for len in  1..MN + 1 {
		for i in 0..len + 1 {
		//	console.log(arr[row][col]);
            vecZigzag.push(matrix[row][col]);
		

			if i + 1 == len {
				break;

            }                                                                               
			// If row_increment value is true
			// increment row and decrement col
			// else decrement row and increment col
			if row_inc {
				row += 1;
                col -= 1;

            }
			else {
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

        }
		else {
			col += 1;
            row_inc = true;
        }
	}

	// Update the indexes of row and col variable
	if row == 0 {
		if col == 8 - 1 {
			row += 1;

        }
		else {
			col += 1;

        }
            
		row_inc = true;
	}
	else {
		if row == 8 - 1 {
			col += 1;
        }
		else {
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

        }
		else {
			len = diag;
        }
        for i in 0..len {
            // console.log(arr[row][col]);
            vecZigzag.push(matrix[row][col]);
			if i + 1 == len {
				break;

            }

			// Update row or col value according
			// to the last increment
			if row_inc {
                row += 1;
                col -= 1;
            }
			else {
                row -= 1;
                col += 1;

            }
        }


		// Update the indexes of row and col variable
		if row == 0 || col == 8 - 1 {
			if col == 8 - 1 {
                row += 1;

            }
			else {
                col += 1;
            }

			row_inc = true;
		}

		else if col == 0 || row == 8 - 1 {
			if row == 8 - 1 {
                col += 1;

            }
			else {
                row += 1;
                                                                    }
			row_inc = false;
		}
        diag -= 1;
    }
    vecZigzag
}

fn main() {
    // let ma = 1
    let matrix = [
    [144,139,149,155,153,155,155,155],
    [151,151,151,159,156,156,156,158],
    [151,156,160,162,159,151,151,151],
    [158,163,161,160,160,160,160,161],
    [158,160,161,162,160,155,155,156],
    [161,161,161,161,160,157,157,157],
    [162,162,161,160,161,157,157,157],
    [162,162,161,160,163,157,158,154]];

    // let img = image::open("src/earth.jpg").unwrap();
    // let a = img.grayscale();
    // let (w, h) =  img.dimensions();
    // println!("{:?}", img.as_bytes());
    // println!("{}, {}", w, h);
    // let dct = Dct2Jpeg::forward(matrix, QuantizationMatrix::ChrominanceQuantizationMatrix);

    let size = 3;
    let a = [
        [1,  2,  3,  4,  5,  6,  7,  8,  9,  10],
        [11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
        [21, 22, 23, 24, 25, 26, 27, 28, 29, 30],
        [31, 32, 33, 34, 35, 36, 37, 38, 39, 40],
        [41, 42, 43, 44, 45, 46, 47, 48, 49, 50],
        [51, 52, 53, 54, 55, 56, 57, 58, 59, 60],
        [61, 62, 63, 64, 65, 66, 67, 68, 69, 70]
    ];
    let w = 10;
    let h  = 7;
    // let block = 0 + 0 * size;
    // let current_width = 
    // for i in block..block + size {
    //     for j in block..block + size {
    //         print!("{}", a[i][j])
    //     }
    //     println!();
    // }
    // 70 / 9 = 8 
    // 12 9 / = 12
    // let (size_w, is_w_remain) = if w % size == 0 {
    //      (w / size, false) 
    //    } else {
    //        ( (w / size) + 1, true)
    //    };
    //    let (size_h, is_h_remain) = if h % size == 0 {
    //     (h / size, false) 
    //   } else {
    //       ( (h / size) + 1, true)
    //   };   

    let  size_w = 4;
    let size_h = 3;
    for m in 1..size_h + 1 {
        let block = (m - 1) * size;
        // println!("{}", block);
        for n in 1..size_w + 1 {
            let block_h = (n - 1) * size;
            // println!("F: {}", block_h);
            // println!("F: {}", block);
            let mut arr_split_8x8 = [[0; 3]; 3];
            for i in block..block + size {
                let mut index_arr_split = 0;
                let mut ele_arr = [0; 3];
                let mut index_arr = 0;

                for j in block_h..block_h + size {
                    // print!("{} ", a[i][j]);
                    if j >= 10 || i >= 7 {
                        // print!("0");
                        ele_arr[index_arr] = 0;
                    } else {
                        // print!("{} ", a[i][j]);
                        // print!("Index: {} ", index_arr);
                        ele_arr[index_arr] = a[i][j];
                    }
                    index_arr += 1;
                    // println!("i: {}, j: {}", i , j)
                }
                // println!("{}", i);
                println!();
                // println!();
                arr_split_8x8[index_arr_split] = ele_arr;
                index_arr_split += 1;

            }
            println!("{:?}", arr_split_8x8);
            println!();
            

        }
        // for n in 1..size_h {
        //     println!("m: {}, n: {}", m , n);

        // }
        // for i in block..block + size {
        //     for j in block..block + size {
        //         // print!("{} ", a[i][j])
        //         println!("i: {}, j: {}", i , j)
        //     }
        //     // println!("{}", i);
        //     println!();

        // }
        // println!("SPACE");
    }

    // let mut a = [1];
    // a[1] = 0
}
