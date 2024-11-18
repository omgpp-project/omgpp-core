fn main(){
    csbindgen::Builder::default()
    .input_extern_file("src/ffi.rs")
    .input_extern_file("../omgpp-core/src/ffi.rs")
    .input_extern_file("../omgpp-core/src/lib.rs")

    .csharp_dll_name("server")
    .csharp_type_rename(move |x| match x.as_str() {     // optional, default: `|x| x`
        "Server" => "void".into(),
        _ => x,
    })
    .csharp_class_name("OmgppClientNative")     
    .csharp_class_accessibility("public")
    .csharp_namespace("OmgppNative")         
    .generate_csharp_file("../../generated/csharp/Client.g.cs")
    .unwrap();
}