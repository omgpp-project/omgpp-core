use glob::glob;
use glob_match::glob_match;
use pathsub::sub_paths;
use std::path::Path;

pub struct AssetCollection {
    paths: Vec<String>,
}
impl AssetCollection {
    pub fn new(paths: Vec<&str>) -> AssetCollection {
        AssetCollection {
            paths: paths.into_iter().map(|str| String::from(str)).collect(),
        }
    }
}

pub struct Folder {
    path: String,
    exclude: Vec<String>,
}

impl Folder {
    pub fn new(path: &str, exclude: Vec<&str>) -> Folder {
        Folder {
            path: String::from(path),
            exclude: exclude.into_iter().map(|str| String::from(str)).collect(),
        }
    }
}

pub enum Assets {
    AssetCollection(AssetCollection),
    Folder(Folder),
}
pub struct Resource {
    name: String,
    assets: Vec<Assets>,
}
impl Resource {
    pub fn new(name: &str, paths: Vec<Assets>) -> Resource {
        Resource {
            name: String::from(name),
            assets: paths,
        }
    }
    pub fn add(&mut self, asset: Assets) {
        self.assets.push(asset);
    }
}

pub struct ResourceManager {
    resources: Vec<Resource>,
}

impl ResourceManager {
    pub fn new() -> ResourceManager {
        ResourceManager {
            resources: Vec::new(),
        }
    }
    pub fn add(&mut self, resource: Resource) {
        self.resources.push(resource);
    }
    pub fn create_index(&self, base_path_str: String) {
        let resources = &self.resources;
        let base_path = Path::new(&base_path_str);
        for resource in resources.into_iter() {
            println!("{:?}", resource.name);
            let assets = &resource.assets;
            for asset in assets.into_iter() {
                match asset {
                    Assets::AssetCollection(asset_collection) => {
                        if asset_collection.paths.len() == 0 {
                            continue;
                        }
                        let assets = &asset_collection.paths;
                        for asset in assets.into_iter() {
                            let mut path_buf = base_path.to_path_buf();
                            path_buf.push(asset);
                            for entry in glob(path_buf.to_str().unwrap()).unwrap() {
                                if let Some(path) = entry.ok() {
                                    if let Some(sub) = sub_paths(&path, base_path){
                                        println!("\t{:?} {:?}", &sub, path.is_file());
                                    }
                                }
                            }
                        }
                    }
                    Assets::Folder(folder) => {
                        let path = Path::new(&folder.path);
                        let mut full_folder_path = base_path.to_path_buf();
                        full_folder_path.push(path);
                        if !full_folder_path.exists() {
                            println!("Folder {:?} does not exist", full_folder_path);
                            continue;
                        }
                        full_folder_path.push("**/*?*");
                        let full_glob_pattern = full_folder_path.to_str().unwrap();
                        for entry in glob(full_glob_pattern).unwrap() {
                            if let Some(path) = &entry.ok() {
                                let sub = sub_paths(&path, base_path).unwrap();
                                let exclude_patterns = &folder.exclude;
                                let mut excluded = false;
                                for exclude in exclude_patterns.into_iter() {
                                    if glob_match(&exclude, sub.to_str().unwrap()) {
                                        excluded = true;
                                        break;
                                    }else{
                                        // println!("{:?} passed {:?}",sub.to_str(),&exclude)
                                    }
                                }
                                if excluded {
                                    continue;
                                }
                                if path.is_file() {
                                    println!("\t{:?}", &sub);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
