use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum Version {
    Lua51,
    Lua52,
    Lua53,
    Lua54,
}
pub use self::Version::*;

pub struct Build {
    out_dir: Option<PathBuf>,
    target: Option<String>,
}

pub struct Artifacts {
    lib_dir: PathBuf,
    libs: Vec<String>,
}

impl Default for Build {
    fn default() -> Build {
        Build {
            out_dir: env::var_os("OUT_DIR").map(PathBuf::from),
            target: env::var("TARGET").ok(),
        }
    }
}

impl Build {
    pub fn new() -> Build {
        Build::default()
    }

    pub fn out_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Build {
        self.out_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn target(&mut self, target: &str) -> &mut Build {
        self.target = Some(target.to_string());
        self
    }

    pub fn build(&mut self, version: Version) -> Artifacts {
        let target = &self.target.as_ref().expect("TARGET is not set")[..];
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR is not set");
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut source_dir = manifest_dir.join(version.source_dir());
        let build_dir = out_dir.join("lua-build");

        let mut config = cc::Build::new();
        config.warnings(false).cargo_metadata(false);

        match target {
            _ if target.contains("linux") => {
                config.define("LUA_USE_LINUX", None);
            }
            _ if target.ends_with("bsd") => {
                config.define("LUA_USE_LINUX", None);
            }
            _ if target.contains("apple-darwin") => {
                match version {
                    Lua51 => config.define("LUA_USE_LINUX", None),
                    _ => config.define("LUA_USE_MACOSX", None),
                };
            }
            _ if target.contains("apple-ios") => {
                match version {
                    Lua54 => config.define("LUA_USE_IOS", None),
                    _ => config.define("LUA_USE_POSIX", None),
                };
            }
            _ if target.contains("windows") => {
                // Defined in Lua >= 5.3
                config.define("LUA_USE_WINDOWS", None);
            }
            _ if target.ends_with("emscripten") => {
                config
                    .define("LUA_USE_POSIX", None)
                    .cpp(true)
                    .flag("-fexceptions"); // Enable exceptions to be caught

                let cpp_source_dir = out_dir.join("cpp_source");
                if cpp_source_dir.exists() {
                    fs::remove_dir_all(&cpp_source_dir).unwrap();
                }
                fs::create_dir_all(&cpp_source_dir).unwrap();

                for file in fs::read_dir(&source_dir).unwrap() {
                    let file = file.unwrap();
                    let filename = file.file_name();
                    let filename = filename.to_str().unwrap();
                    let src_file = source_dir.join(file.file_name());
                    let dst_file = cpp_source_dir.join(file.file_name());

                    let mut content = fs::read(src_file).unwrap();
                    if ["lauxlib.h", "lua.h", "lualib.h"].contains(&filename) {
                        content.splice(0..0, b"extern \"C\" {\n".to_vec());
                        content.extend(b"\n}".to_vec())
                    }
                    fs::write(dst_file, content).unwrap();
                }
                source_dir = cpp_source_dir
            }
            _ => panic!("don't know how to build Lua for {target}"),
        }

        if let Lua54 = version {
            config.define("LUA_COMPAT_5_3", None);
            #[cfg(feature = "ucid")]
            config.define("LUA_UCID", None);
        }

        if cfg!(debug_assertions) {
            config.define("LUA_USE_APICHECK", None);
        }

        config
            .include(&source_dir)
            .flag("-w") // Suppress all warnings
            .flag_if_supported("-fno-common") // Compile common globals like normal definitions
            .add_files_by_ext(&source_dir, "c")
            .out_dir(&build_dir)
            .compile(version.lib_name());

        Artifacts {
            lib_dir: build_dir,
            libs: vec![version.lib_name().to_string()],
        }
    }
}

impl Version {
    fn source_dir(&self) -> &str {
        match self {
            Lua51 => "lua-5.1.5",
            Lua52 => "lua-5.2.4",
            Lua53 => "lua-5.3.6",
            Lua54 => "lua-5.4.8",
        }
    }

    fn lib_name(&self) -> &str {
        match self {
            Lua51 => "lua5.1",
            Lua52 => "lua5.2",
            Lua53 => "lua5.3",
            Lua54 => "lua5.4",
        }
    }
}

impl Artifacts {
    pub fn lib_dir(&self) -> &Path {
        &self.lib_dir
    }

    pub fn libs(&self) -> &[String] {
        &self.libs
    }

    pub fn print_cargo_metadata(&self) {
        println!("cargo:rustc-link-search=native={}", self.lib_dir.display());
        for lib in self.libs.iter() {
            println!("cargo:rustc-link-lib=static={lib}");
        }
    }
}

trait AddFilesByExt {
    fn add_files_by_ext(&mut self, dir: &Path, ext: &str) -> &mut Self;
}

impl AddFilesByExt for cc::Build {
    fn add_files_by_ext(&mut self, dir: &Path, ext: &str) -> &mut Self {
        for entry in fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension() == Some(ext.as_ref()))
        {
            self.file(entry.path());
        }
        self
    }
}
