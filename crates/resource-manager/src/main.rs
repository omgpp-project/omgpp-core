use std::path::Path;

use resource_manager::resource_indexer::{
    AssetCollection, Assets, Folder, Resource, ResourceIndexer,
};

fn main() {
    let mut indexer = ResourceIndexer::new();
    let mut protos = Resource::new("proto", vec![]);
    protos.add(Assets::AssetCollection(AssetCollection::new(vec![
        "proto/**/*",
    ])));
    let mut csharp = Resource::new("csharp", vec![]);
    csharp.add(Assets::Folder(Folder::new(
        "languages",
        vec!["**/__pycache__/**"],
    )));

    indexer.add(protos);
    indexer.add(csharp);
    let registry = indexer.create_registry("../../../../omgpp/omgpp-protoc-plugin".to_string());
    println!("{:?}", registry);
    // rm.create_index("/home/rust/omgpp/omgpp-protoc-plugin".to_string());
}
