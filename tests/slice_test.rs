#[test]
pub fn test_slice() {
    let my_vec = vec![ 1_usize, 2, 3, 4 ];

    println!("Hello World {:?}", &*my_vec);
}
