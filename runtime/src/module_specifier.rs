/// Module specifier may have a form of
/// 1) {transpiler_name}:{path}
/// 2) {path}
pub struct ModuleSpecifier<'a> {
    transpiler_name: Option<&'a str>,
    path: &'a str
}

impl<'a> ModuleSpecifier<'a> {
    pub fn from(name: &'a str) -> ModuleSpecifier<'a> {
        let mut index = 0;
        let mut transpiler_name = None;
        for c in name.chars() {
            if c == ':' {
                transpiler_name = Some(&name[0..index]);
                break;
            }
            index+=1
        }
        let path = if transpiler_name.is_some() {
            &name[(index+1)..]
        } else {
            name
        };
        ModuleSpecifier {
            transpiler_name,
            path
        }
    }

    pub fn transpiler(&self) -> Option<&str> {
        self.transpiler_name
    }

    pub fn path(&self) -> &str {
        self.path
    }

    pub fn extension(&self) -> Option<&str> {
        std::path::Path::new(self.path).extension().map(|os_str| {
            os_str.to_str().expect("non-utf8")
        })
    }
}

#[test]
fn test_module_specifier() {
    let ms = ModuleSpecifier::from("tr_name:./mod_name");
    assert_eq!(ms.path, "./mod_name");
    assert_eq!(ms.transpiler_name, Some("tr_name"));

    let ms = ModuleSpecifier::from("react");
    assert_eq!(ms.path, "react");
    assert_eq!(ms.transpiler_name, None);
}

#[test]
fn test_empty_path() {
    let ms = ModuleSpecifier::from("tr_name:");
    assert_eq!(ms.path(), "")
}

#[test]
fn test_empty_path_and_transpiler() {
    let ms = ModuleSpecifier::from(":");
    assert_eq!(ms.path(), "")
}

#[test]
fn test_empty_transpiler() {
    let ms = ModuleSpecifier::from(":./test");
    assert_eq!(ms.transpiler(), Some(""))
}