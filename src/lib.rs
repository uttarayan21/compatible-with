//! # Compatible
//! Compatibility Layer for older data using serde  
//! You just need to provide a `Current: From<Old>` implementation  
//! And the rest is handled automatically  
//! Keep in mind that this uses untagged enums so it comes with performance cost  
pub use compatible_with_derive::CompatibleWith;
use serde::*;

/// This is the main type you will be using  
/// It wraps your old and current type and provides a way to deserialize existing data that might  
/// match either of the types  
/// It will deserialize the old type into an deserialize impl for the old type and then convert it  
/// to the new type  
/// ```rust,ignore
/// #[derive(Serialize, Deserialize)]
/// pub struct Current;
/// pub struct SomeType {
///     my_var: Current,
/// }
/// ```

/// The `Current` version of the struct is `CompatibleWith<Old>`
pub trait CompatibleWith<Old> {
    fn from_old(value: Old) -> Self;
}

/// The `Old` is `CompatibleTo<Current>` version of the struct
pub trait CompatibleTo<Current> {
    fn into_current(self) -> Current;
}

impl<Old, Current> CompatibleWith<Old> for Current
where
    Current: From<Old>,
{
    fn from_old(value: Old) -> Self {
        value.into()
    }
}

impl<Old, Current> CompatibleTo<Current> for Old
where
    Current: CompatibleWith<Old>,
{
    fn into_current(self) -> Current {
        Current::from_old(self)
    }
}

mod with {
    use super::{Compatible, CompatibleWith};
    use serde::Deserialize;

    impl<Old, Current> Compatible<Old, Current> {
        pub fn deserialize_with<'de, D>(deserializer: D) -> Result<Current, D::Error>
        where
            D: serde::de::Deserializer<'de>,
            Self: Deserialize<'de>,
            Current: CompatibleWith<Old>,
        {
            let compatible: Compatible<Old, Current> = Compatible::deserialize(deserializer)?;
            Ok(compatible.into_current())
        }
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Hash, Clone, Copy)]
pub struct Compatible<Old, Current>(Alt<Old, Current>);

impl<Old, Current> Compatible<Old, Current>
where
    Current: CompatibleWith<Old>,
{
    pub fn into_current(self) -> Current {
        match self.0 {
            Alt::Old(old) => old.into_current(),
            Alt::Current(current) => current,
        }
    }

    pub fn make_current(mut self) -> Self {
        if let Alt::Old(old) = self.0 {
            self.0 = Alt::Current(old.into_current())
        };
        self
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Hash, Clone, Copy)]
pub enum Alt<Old, Current> {
    Old(Old),
    Current(Current),
}

impl<'de, Old, Current> serde::de::Deserialize<'de> for Compatible<Old, Current>
where
    Current: CompatibleWith<Old>,
    Alt<Old, Current>: serde::de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let alt = Alt::deserialize(deserializer)?;

        Ok(Compatible(alt).make_current())
    }
}

impl<Old, Current> serde::ser::Serialize for Compatible<Old, Current>
where
    Old: Serialize,
    Current: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self.0 {
            Alt::Old(ref old) => old.serialize(serializer),
            Alt::Current(ref current) => current.serialize(serializer),
        }
    }
}

#[test]
pub fn test_simple() {
    use serde::*;
    #[derive(Serialize, Deserialize)]
    pub struct Old {
        pub a: i32,
    }

    #[derive(Serialize, Deserialize)]
    pub struct New {
        pub a: String,
        pub b: i32,
    }

    impl From<Old> for New {
        fn from(old: Old) -> Self {
            New {
                a: old.a.to_string(),
                b: 0,
            }
        }
    }

    let old = Old { a: 1 };
    let old_serialized = serde_json::to_string(&old).unwrap();
    let migrated: Compatible<Old, New> = serde_json::from_str(&old_serialized).unwrap();
    let migrated = migrated.into_current();

    assert_eq!(migrated.a, "1");
    assert_eq!(migrated.b, 0);
}

#[test]
pub fn test_complex() {
    use serde::*;
    #[derive(Serialize, Deserialize)]
    pub struct Dir {
        id: i64,
        name: String,
        path: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct DirNode {
        id: i64,
        name: String,
        path: String,
        children: Vec<DirNode>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Old {
        pub dirs: Vec<Dir>,
        pub root: Dir,
    }

    #[derive(Serialize, Deserialize)]
    pub struct New {
        pub dirs: Compatible<Vec<Dir>, DirNode>,
    }

    impl From<Dir> for DirNode {
        fn from(old: Dir) -> Self {
            DirNode {
                id: old.id,
                name: old.name,
                path: old.path,
                children: vec![],
            }
        }
    }

    impl From<Vec<Dir>> for DirNode {
        fn from(old: Vec<Dir>) -> Self {
            let mut root = DirNode {
                id: 0,
                name: "root".to_string(),
                path: "/".to_string(),
                children: vec![],
            };
            root.children.extend(old.into_iter().map(|d| d.into()));
            root
        }
    }

    let old = Old {
        dirs: vec![
            Dir {
                id: 1,
                name: "a".to_string(),
                path: "/a".to_string(),
            },
            Dir {
                id: 2,
                name: "b".to_string(),
                path: "/b".to_string(),
            },
        ],
        root: Dir {
            id: 0,
            name: "root".to_string(),
            path: "/".to_string(),
        },
    };

    let old_serialized = serde_json::to_string(&old).unwrap();
    let migrated: New = serde_json::from_str(&old_serialized).unwrap();
    let migrated_serialized = serde_json::to_string(&migrated).unwrap();
    assert_eq!(
        migrated_serialized,
        r#"{"dirs":{"id":0,"name":"root","path":"/","children":[{"id":1,"name":"a","path":"/a","children":[]},{"id":2,"name":"b","path":"/b","children":[]}]}}"#
    );
}

#[test]
pub fn test_with() {
    use serde::*;

    #[derive(Debug, Deserialize, PartialEq)]
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
        #[serde(deserialize_with = "Compatible::<i32, MyType>::deserialize_with")]
        pub a: MyType,
    }

    let old = Old { a: 1 };
    let old_serialized = serde_json::to_string(&old).unwrap();
    let migrated: New = serde_json::from_str(&old_serialized).unwrap();

    assert_eq!(migrated.a, MyType("1".into()));
}
