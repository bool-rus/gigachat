fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("running prost codegen");
    tonic_build::configure().build_server(false)
        .protoc_arg("--experimental_allow_proto3_optional")
        //.client_attribute(".", "#[derive(derive_more::From, derive_more::Into)]")
        //.type_attribute("Quotation", "#[derive(Eq, Ord, PartialOrd)]")
        .compile_protos(
            &[
                "proto/gigachatv1.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}