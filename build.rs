fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("running prost codegen");
    tonic_build::configure().build_server(false)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &[
                "proto/gigachatv1.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}