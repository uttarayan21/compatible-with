use serde::*;

pub struct CompatibleWith<Old, Current>(Alt<Old, Current>);

impl<Old, Current> CompatibleWith<Old, Current>
where
    Current: From<Old>,
{
    pub fn into_current(self) -> Current {
        match self.0 {
            Alt::Old(old) => old.into(),
            Alt::Current(current) => current,
        }
    }

    pub fn make_current(mut self) -> Self {
        if let Alt::Old(old) = self.0 {
            self.0 = Alt::Current(old.into())
        };
        self
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Alt<Old, Current> {
    Old(Old),
    Current(Current),
}

impl<'de, Old, Current> serde::de::Deserialize<'de> for CompatibleWith<Old, Current>
where
    Current: From<Old>,
    Alt<Old, Current>: serde::de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let alt = Alt::deserialize(deserializer)?;
        Ok(CompatibleWith(alt))
    }
}

impl<Old, Current> serde::ser::Serialize for CompatibleWith<Old, Current>
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
    let migrated: CompatibleWith<Old, New> = serde_json::from_str(&old_serialized).unwrap();
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
        dirs: CompatibleWith<Dir, DirNode>,
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
}
