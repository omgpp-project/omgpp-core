fn main() {
    // TODO uncomment and set output path to env::var("OUT_DIR")

    csbindgen::Builder::default()
        .input_extern_file("src/ffi.rs")
        .csharp_dll_name("server")
        .csharp_type_rename(move |x| match x.as_str() {     // optional, default: `|x| x`
            "Server" => "void".into(),
            _ => x,
        })
        .csharp_class_name("OmgppServerNative")     
        .csharp_class_accessibility("public")
        .csharp_namespace("OmgppNative")         
        .generate_csharp_file("../../generated/csharp/Server.g.cs")
        .unwrap();
    
}
