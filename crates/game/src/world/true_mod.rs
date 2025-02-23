pub fn true_mod(a: i32, b: i32) -> i32 {
    (a % b + b) % b
}

#[test]
fn test_true_mod() {
    assert_eq!(true_mod(5, 3), 2);
    assert_eq!(true_mod(-5, 3), 1);

    assert_eq!(true_mod(5, 8), 5);
    assert_eq!(true_mod(-5, 8), 3);
    assert_eq!(true_mod(-13, 8), 3);
    assert_eq!(true_mod(-8, 8), 0);
    assert_eq!(true_mod(-1, 8), 7);
    assert_eq!(true_mod(-9, 8), 7);
}
