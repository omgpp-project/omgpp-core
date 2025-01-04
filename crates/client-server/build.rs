fn main(){
    let client_csharp_native = csbindgen::Builder::default()
    .input_extern_file("src/client/ffi.rs")
    .csharp_dll_name("client_server")
    .csharp_type_rename(move |x| match x.as_str() {     // optional, default: `|x| x`
        "Client" => "void".into(),
        _ => x,
    })
    .csharp_class_name("OmgppClientNative")
    .csharp_class_accessibility("public")
    .csharp_namespace("OmgppNative")
    .generate_csharp_file("../../generated/csharp/Client.g.cs");
    if let Err(error) = client_csharp_native {
        panic!("Failed to generate file: {}", &error.to_string());
    }

    let server_csharp_native = csbindgen::Builder::default()
    .input_extern_file("src/server/ffi.rs")
    .csharp_dll_name("client_server")
    .csharp_type_rename(move |x| match x.as_str() {     // optional, default: `|x| x`
        "Server" => "void".into(),
        _ => x,
    })
    .csharp_class_name("OmgppServerNative")
    .csharp_class_accessibility("public")
    .csharp_namespace("OmgppNative")
    .generate_csharp_file("../../generated/csharp/Server.g.cs");
    if let Err(error) = server_csharp_native {
        panic!("Failed to generate file: {}", &error.to_string());
    }
}
