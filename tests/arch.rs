use std::fs;
use std::path::Path;

mod src {
    mod domain {
        #[test]
        fn then_domain_does_not_import_from_application_or_adapters() {
            let domain_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/domain");
            assert_no_imports_from(&domain_dir, &["application", "adapters"]);
        }
    }

    mod application {
        #[test]
        fn then_application_does_not_import_concrete_adapters() {
            let app_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/application");
            assert_no_imports_from(&app_dir, &["adapters"]);
        }
    }
}

fn assert_no_imports_from(dir: &Path, forbidden: &[&str]) {
    if !dir.exists() {
        return;
    }
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            assert_no_imports_from(&path, forbidden);
            continue;
        }
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap();
        let file = path.to_string_lossy();
        for forbidden_mod in forbidden {
            assert!(
                !content.contains(&format!("use crate::{forbidden_mod}"))
                    && !content.contains(&format!("mod {forbidden_mod}")),
                "{file} imports from {forbidden_mod} — hexagonal boundary violation"
            );
        }
    }
}