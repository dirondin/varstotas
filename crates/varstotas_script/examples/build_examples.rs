//! Rebuild repository-only demos: cargo run -p varstotas_script --example build_examples
use std::path::Path;
use varstotas_kernel::{Model, io};
use varstotas_script::{ScriptCtx, run};
fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in [
        "bookshelf",
        "workbench",
        "dining_set",
        "garden_pergola",
        "staircase",
        "small_workshop",
    ] {
        let source =
            std::fs::read_to_string(root.join(format!("examples/scripts/{name}.rhai"))).unwrap();
        let mut model = Model::new();
        let result = run(&mut model, &ScriptCtx::default(), &source);
        assert!(result.error.is_none(), "{name}: {:?}", result.error);
        assert!(result.changed, "{name}: no geometry");
        io::validate_model(&model).unwrap();
        let path = root.join(format!("examples/demos/{name}.vrst"));
        io::save(&model, &path).unwrap();
        let loaded = io::load(&path).unwrap();
        let faces: usize = loaded.defs.values().map(|d| d.mesh.faces.len()).sum();
        assert!(faces > 0);
        println!(
            "{name}: {} definitions, {faces} faces, {} materials",
            loaded.defs.len(),
            loaded.materials.len()
        );
    }
}
