mod gen;
mod template_str;
mod value;

use crate::config;
use crate::sites::Site;
pub use gen::*;
pub use template_str::*;
pub use value::*;

/// Holds the information about the execution of a single run on a specific site with its bound run configurations.
#[derive(Debug, Clone)]
pub struct Context {
    #[allow(dead_code)]
    // The part of the code that uses this is not yet implemented, so it's not dead code.
    pub site: Site,

    #[allow(dead_code)]
    // The part of the code that uses this is not yet implemented, so it's not dead code.
    pub run: config::runs::RunConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::data::GeoDeg;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_context_dir() {
        let wd = PathBuf::from("/tmp");
        let ctx = Context {
            site: Site {
                id: 0,
                lon: GeoDeg::from(15.222),
                lat: GeoDeg::from(-15.23133),
            },
            run: config::runs::RunConfig {
                name: String::from("r1"),
                extra: HashMap::new(),
                template: PathBuf::from("dummy"),
            },
        };

        assert_eq!(ctx.dir(&wd), PathBuf::from("/tmp/r1/15_2220N/15_2313W"));
    }
}
