use crux_core::typegen::TypeGen;
use shared::{view_model::Category, App};
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../shared");

    let mut gen = TypeGen::new();

    gen.register_app::<App>().expect("register");

    // we might be able to get rid of this if we use all variants of the enum
    gen.register_type_with_samples(vec![
        Category::Total,
        Category::Gas,
        Category::Coal,
        Category::Biomass,
        Category::Nuclear,
        Category::Hydro,
        Category::Imports,
        Category::Other,
        Category::Wind,
        Category::Solar,
    ])
    .expect("register");

    let output_root = PathBuf::from("./generated");

    gen.swift("SharedTypes", output_root.join("swift"))
        .expect("swift type gen failed");

    gen.java("com.example.counter.shared_types", output_root.join("java"))
        .expect("java type gen failed");

    gen.typescript("shared_types", output_root.join("typescript"))
        .expect("typescript type gen failed");
}
