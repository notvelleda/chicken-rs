use super::VMBuilder;
use std::fs::read_to_string;

#[test]
fn quine() {
    assert_eq!(
        VMBuilder::from_chicken("chicken").build().run(),
        Ok("chicken".to_string())
    )
}

#[test]
fn cat() {
    assert_eq!(
        VMBuilder::from_chicken(&read_to_string("examples/cat.chicken").unwrap())
            .input("this is a test")
            .build()
            .run(),
        Ok("this is a test".to_string())
    )
}

#[test]
fn hello_world() {
    assert_eq!(
        VMBuilder::from_chicken(&read_to_string("examples/helloworld.chicken").unwrap())
            .build()
            .run(),
        Ok("Hello world".to_string())
    )
}

#[test]
fn chickens() {
    fn make_chickens(num: usize) -> String {
        (0..=num)
            .rev()
            .map(|n| {
                if n == 0 {
                    "no chickens\n".to_string()
                } else if n == 1 {
                    "1 chicken".to_string()
                } else {
                    format!("{} chickens", n)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    assert_eq!(
        VMBuilder::from_chicken(&read_to_string("examples/99chickens.chicken").unwrap())
            .input("9")
            .build()
            .run(),
        Ok(make_chickens(9))
    );
    assert_eq!(
        VMBuilder::from_chicken(&read_to_string("examples/99chickens.chicken").unwrap())
            .input("128")
            .build()
            .run(),
        Ok(make_chickens(128))
    );
    assert_eq!(
        VMBuilder::from_chicken(&read_to_string("examples/99chickens.chicken").unwrap())
            .input("512")
            .build()
            .run(),
        Ok(make_chickens(512))
    );
    assert_eq!(
        VMBuilder::from_chicken(&read_to_string("examples/99chickens.chicken").unwrap())
            .input("1024")
            .build()
            .run(),
        Ok(make_chickens(1024))
    )
}

#[test]
fn deadfish() {
    assert_eq!(
        VMBuilder::from_chicken(&read_to_string("examples/deadfish.chicken").unwrap())
            .input("iissiso")
            .build()
            .run(),
        Ok(" 289 ".to_string())
    );
    assert_eq!(
        VMBuilder::from_chicken(&read_to_string("examples/deadfish.chicken").unwrap())
            .input("iissso")
            .build()
            .run(),
        Ok(" 0 ".to_string())
    );
    assert_eq!(
        VMBuilder::from_chicken(&read_to_string("examples/deadfish.chicken").unwrap())
            .input("diissisdo")
            .build()
            .run(),
        Ok(" 288 ".to_string())
    );
    assert_eq!(
        VMBuilder::from_chicken(&read_to_string("examples/deadfish.chicken").unwrap())
            .input("iissisdddddddddddddddddddddddddddddddddo")
            .build()
            .run(),
        Ok(" 0 ".to_string())
    );
}
