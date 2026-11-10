fn a() {
    todo!();
    unimplemented!("x");
    dbg!(1);
    std::dbg![2];
    todo! {}
    todo !();
    let s = "todo!() dbg!(x)";
    // todo!() in a comment is not code
    let todo = my_todo!() + todo_x!();
    macro_rules! todo { () => {} }
    let r = r#"dbg!(1)"#;
    unimplemented
        !
        ();
    let r = 0..dbg!(x);
}
