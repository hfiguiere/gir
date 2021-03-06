use std::io::{Result, Write};

use analysis;
use analysis::special_functions::Type;
use env::Env;
use super::{function, general, trait_impls};

pub fn generate(w: &mut Write, env: &Env, analysis: &analysis::record::Info) -> Result<()>{
    let type_ = analysis.type_(&env.library);

    try!(general::start_comments(w, &env.config));
    try!(general::uses(w, env, &analysis.imports));

    if let (Some(ref_fn), Some(unref_fn)) = (analysis.specials.get(&Type::Ref),
                                             analysis.specials.get(&Type::Unref)) {
        try!(general::define_shared_type(w, &analysis.name, &type_.c_type, ref_fn, unref_fn));
    } else if let (Some(copy_fn), Some(free_fn)) = (analysis.specials.get(&Type::Copy),
                                                    analysis.specials.get(&Type::Free)) {
        try!(general::define_boxed_type(w, &analysis.name, &type_.c_type, copy_fn, free_fn));
    } else {
        panic!("Missing memory management functions for {}", analysis.full_name);
    }
    try!(writeln!(w, ""));
    try!(writeln!(w, "impl {} {{", analysis.name));

    for func_analysis in &analysis.functions {
        try!(function::generate(w, env, func_analysis, false, false, 1));
    }

    try!(writeln!(w, "}}"));

    try!(trait_impls::generate(w, &analysis.name, &analysis.functions, &analysis.specials));

    Ok(())
}

pub fn generate_reexports(env: &Env, analysis: &analysis::record::Info, module_name: &str,
                          contents: &mut Vec<String>) {
    let cfg_condition = general::cfg_condition_string(&analysis.cfg_condition, false, 0);
    let version_cfg = general::version_condition_string(env, analysis.version, false, 0);
    let mut cfg = String::new();
    if let Some(s) = cfg_condition {
        cfg.push_str(&s);
        cfg.push('\n');
    };
    if let Some(s) = version_cfg {
        cfg.push_str(&s);
        cfg.push('\n');
    };
    contents.push(format!(""));
    contents.push(format!("{}mod {};", cfg, module_name));
    contents.push(format!("{}pub use self::{}::{};", cfg, module_name, analysis.name));
}
