use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_yml;

const LANGUAGE_EXTENSIONS: &str = &"
Python:
  - \".py\"
Java:
  - \".java\"
JavaScript:
  - \".js\"
  - \".jsx\"
C:
  - \".c\"
  - \".h\"
C++:
  - \".cpp\"
  - \".hpp\"
  - \".cc\"
  - \".cxx\"
  - \".hxx\"
C#:
  - \".cs\"
Ruby:
  - \".rb\"
  - \".rbw\"
Go:
  - \".go\"
Swift:
  - \".swift\"
Kotlin:
  - \".kt\"
  - \".kts\"
Rust:
  - \".rs\"
TypeScript:
  - \".ts\"
  - \".tsx\"
HTML:
  - \".html\"
  - \".htm\"
CSS:
  - \".css\"
PHP:
  - \".php\"
  - \".phtml\"
  - \".php4\"
  - \".php3\"
  - \".php5\"
  - \".php7\"
  - \".phps\"
  - \".php-s\"
Perl:
  - \".pl\"
  - \".pm\"
  - \".t\"
  - \".pod\"
R:
  - \".r\"
  - \".rdata\"
  - \".rds\"
  - \".rda\"
Scala:
  - \".scala\"
  - \".sc\"
Shell:
  - \".sh\"
  - \".bash\"
  - \".zsh\"
  - \".csh\"
  - \".tcsh\"
  - \".ksh\"
Lua:
  - \".lua\"
MATLAB:
  - \".m\"
  - \".mlx\"
Groovy:
  - \".groovy\"
  - \".grt\"
  - \".gtpl\"
  - \".gsp\"
Objective-C:
  - \".m\"
  - \".mm\"
  - \".M\"
  - \".h\"
Assembly:
  - \".asm\"
  - \".s\"
Haskell:
  - \".hs\"
  - \".lhs\"
Julia:
  - \".jl\"
Fortran:
  - \".f\"
  - \".for\"
  - \".f90\"
  - \".f95\"
  - \".f03\"
  - \".f08\"
  - \".f15\"
Pascal:
  - \".pas\"
  - \".pp\"
  - \".p\"
  - \".inc\"
SQL:
  - \".sql\"
XML:
  - \".xml\"
";


pub type FileExtensionList = Vec<String>;
pub type FileExtensionTypeMap = HashMap<String, String>;

pub struct LanguageExtensions {
    pub ext_list: FileExtensionList,
    pub ext_type_map: FileExtensionTypeMap,
}

impl LanguageExtensions  {
    pub fn new() -> Self {
        Self::from_str(LANGUAGE_EXTENSIONS)
    }

    fn from_str(s: &str) -> Self {
        let _file_extension: HashMap<String, Vec<String>> 
            = serde_yml::from_str(s).unwrap();

        let mut file_extension_map: FileExtensionTypeMap = HashMap::new();
        for (k, v) in _file_extension.iter() {
            for i in v.iter() {
                file_extension_map.insert(i.clone(), k.clone());
            }
        };
        Self {
            ext_list: file_extension_map.clone().into_keys().collect::<FileExtensionList>(),
            ext_type_map: file_extension_map
        }
    }

    pub fn new_from_path(path: &Path) -> Self {
        let s = fs::read_to_string(path).unwrap();
        Self::from_str(&s)
    }
    
    pub fn write_to_file(&self, path: &Path) {
        let mut o: HashMap<String, Vec<String>> = HashMap::new();
        self.ext_type_map.values().map(|x| {
            let v = self.ext_type_map.get(x).unwrap().clone();
            o
                .entry(x.clone())
                .and_modify(|e|{
                    e.push(v.clone());
                })
                .or_insert(vec![v])
            ;
        });
        let ignore_string = serde_yml::to_string(&o).unwrap();
        fs::write(path, ignore_string).unwrap();
    }

}
