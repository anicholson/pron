use std::fs;
use std::path::Path;

mod src {
    use super::*;

    mod domain {
        use super::*;

        #[test]
        fn then_domain_does_not_import_from_application_or_adapters() {
            let domain_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/domain");
            assert_no_references_to(&domain_dir, &["crate::application", "crate::adapters"]);
        }
    }

    mod application {
        use super::*;

        #[test]
        fn then_application_does_not_import_concrete_adapters() {
            let app_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/application");
            assert_no_references_to(&app_dir, &["crate::adapters"]);
        }
    }
}

fn assert_no_references_to(dir: &Path, forbidden: &[&str]) {
    assert!(
        dir.exists(),
        "{} does not exist — architectural contract requires this directory",
        dir.display()
    );
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            assert_no_references_to(&path, forbidden);
            continue;
        }
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap();
        let file = path.to_string_lossy();
        for forbidden_path in forbidden {
            assert!(
                !content.contains(forbidden_path),
                "{file} references {forbidden_path} — hexagonal boundary violation"
            );
        }
    }
}