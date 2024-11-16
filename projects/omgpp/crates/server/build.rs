fn main() {
    // TODO uncomment and set output path to env::var("OUT_DIR")

    /*
    let server_struct_name = String::from("Server");

    csbindgen::Builder::default()
        .input_extern_file("src/ffi.rs")
        .csharp_dll_name("server")
        .csharp_type_rename(move |x| match x {     // optional, default: `|x| x`
            server_struct_name => "IntPtr".into(),
            _ => x,
        })
        .generate_csharp_file("NativeMethods.g.cs")
        .unwrap();
    */
}
