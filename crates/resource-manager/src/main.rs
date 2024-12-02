use std::path::Path;

use resource_manager::{AssetCollection, Assets, Folder, Resource, ResourceManager};

fn main() {
    let mut rm = ResourceManager::new();
    let mut protos = Resource::new("proto", vec![]);
    protos.add(Assets::AssetCollection(AssetCollection::new(vec![
        "proto/**/*",
    ])));
    let mut csharp = Resource::new("csharp", vec![]);
    csharp.add(Assets::Folder(Folder::new(
        "languages",
        vec!["**/__pycache__/**"],
    )));

    rm.add(protos);
    rm.add(csharp);
    rm.create_index("../../../../omgpp/omgpp-protoc-plugin".to_string());

    // rm.create_index("/home/rust/omgpp/omgpp-protoc-plugin".to_string());
}
