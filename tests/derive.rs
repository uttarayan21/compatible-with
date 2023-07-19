use compatible_with::{Compatible, CompatibleWith};
#[test]
pub fn test_derived() {
    use serde::*;

    #[derive(Debug, Deserialize, PartialEq, CompatibleWith)]
    #[serde(from = "Compatible::<i32, MyType>")]
    pub struct MyType(String);

    #[derive(Serialize)]
    pub struct Old {
        pub a: i32,
    }

    impl From<i32> for MyType {
        fn from(value: i32) -> Self {
            MyType(value.to_string())
        }
    }
    // impl<T: ToString> From<T> for MyType {
    //     fn from(value: T) -> Self {
    //         MyType(value.to_string())
    //     }
    // }

    #[derive(Deserialize)]
    pub struct New {
        pub a: MyType,
    }

    let old = Old { a: 1 };
    let old_serialized = serde_json::to_string(&old).unwrap();
    let migrated: New = serde_json::from_str(&old_serialized).unwrap();

    assert_eq!(migrated.a, MyType("1".into()));
}
