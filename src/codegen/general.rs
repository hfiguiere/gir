use std::fmt::Display;
use std::io::{Result, Write};

use analysis::general::StatusedTypeId;
use analysis::imports::Imports;
use analysis::namespaces;
use config::Config;
use env::Env;
use gir_version::VERSION;
use version::Version;
use writer::primitives::tabs;

pub fn start_comments(w: &mut Write, conf: &Config) -> Result<()>{
    try!(writeln!(w, "// This file was generated by gir ({}) from gir-files ({})",
            VERSION, conf.girs_version));
    try!(writeln!(w, "// DO NOT EDIT"));

    Ok(())
}

pub fn uses(w: &mut Write, env: &Env, imports: &Imports) -> Result<()> {
    try!(writeln!(w, ""));
    for (name, &version) in imports.iter() {
        try!(version_condition(w, env, version, false, 0));
        try!(writeln!(w, "use {};", name));
    }

    Ok(())
}

pub fn define_object_type(w: &mut Write, env: &Env, type_name: &str, glib_name: &str,
        glib_func_name: &str, parents: &[StatusedTypeId]) -> Result<()> {
    let mut external_parents = false;
    let parents: Vec<String> = parents.iter()
        .filter(|p| !p.status.ignored())
        .map(|p| {
            if p.type_id.ns_id == namespaces::MAIN {
                p.name.clone()
            } else {
                external_parents = true;
                format!("{krate}::{name} => {krate}_ffi::{ffi_name}",
                    krate = env.namespaces[p.type_id.ns_id].crate_name,
                    name = p.name,
                    ffi_name = env.library.type_(p.type_id).get_glib_name().unwrap())
            }
        })
        .collect();

    try!(writeln!(w, ""));
    try!(writeln!(w, "glib_wrapper! {{"));
    if parents.is_empty() {
        try!(writeln!(w, "\tpub struct {}(Object<ffi::{}>);", type_name, glib_name));
    } else if external_parents {
        try!(writeln!(w, "\tpub struct {}(Object<ffi::{}>): [", type_name, glib_name));
        for parent in parents {
            try!(writeln!(w, "\t\t{},", parent));
        }
        try!(writeln!(w, "\t];"));
    } else {
        try!(writeln!(w, "\tpub struct {}(Object<ffi::{}>): {};", type_name, glib_name,
            parents.join(", ")));
    }
    try!(writeln!(w, ""));
    try!(writeln!(w, "\tmatch fn {{"));
    try!(writeln!(w, "\t\tget_type => || ffi::{}(),", glib_func_name));
    try!(writeln!(w, "\t}}"));
    try!(writeln!(w, "}}"));

    Ok(())
}

pub fn define_boxed_type(w: &mut Write, type_name: &str, glib_name: &str,
                                copy_fn: &str, free_fn: &str) -> Result<()>{
    try!(writeln!(w, ""));
    try!(writeln!(w, "glib_wrapper! {{"));
    try!(writeln!(w, "\tpub struct {}(Boxed<ffi::{}>);", type_name, glib_name));
    try!(writeln!(w, ""));
    try!(writeln!(w, "\tmatch fn {{"));
    try!(writeln!(w, "\t\tcopy => |ptr| ffi::{}(mut_override(ptr)),", copy_fn));
    try!(writeln!(w, "\t\tfree => |ptr| ffi::{}(ptr),", free_fn));
    try!(writeln!(w, "\t}}"));
    try!(writeln!(w, "}}"));

    Ok(())
}

pub fn define_shared_type(w: &mut Write, type_name: &str, glib_name: &str,
                          ref_fn: &str, unref_fn: &str) -> Result<()>{
    try!(writeln!(w, ""));
    try!(writeln!(w, "glib_wrapper! {{"));
    try!(writeln!(w, "\tpub struct {}(Shared<ffi::{}>);", type_name, glib_name));
    try!(writeln!(w, ""));
    try!(writeln!(w, "\tmatch fn {{"));
    try!(writeln!(w, "\t\tref => |ptr| ffi::{}(ptr),", ref_fn));
    try!(writeln!(w, "\t\tunref => |ptr| ffi::{}(ptr),", unref_fn));
    try!(writeln!(w, "\t}}"));
    try!(writeln!(w, "}}"));

    Ok(())
}

pub fn version_condition(w: &mut Write, env: &Env, version: Option<Version>, commented: bool,
                         indent: usize) -> Result<()> {
    if let Some(s) = version_condition_string(env, version, commented, indent) {
        try!(writeln!(w, "{}", s));
    }
    Ok(())
}

pub fn version_condition_string(env: &Env, version: Option<Version>, commented: bool, indent: usize)
        -> Option<String> {
    match version {
        Some(v) if v > env.config.min_cfg_version => {
            let comment = if commented { "//" } else { "" };
            Some(format!("{}{}#[cfg({})]", tabs(indent), comment, v.to_cfg()))
        }
        _ => None
    }
}

pub fn cfg_condition(w: &mut Write, cfg_condition: &Option<String>, commented: bool, indent: usize)
                     -> Result<()> {
    let s = cfg_condition_string(cfg_condition, commented, indent);
    if let Some(s) = s {
        try!(writeln!(w, "{}", s));
    }
    Ok(())
}

pub fn cfg_condition_string(cfg_condition: &Option<String>, commented: bool, indent: usize)
                            -> Option<String> {
    match cfg_condition.as_ref() {
        Some(v) => {
            let comment = if commented { "//" } else { "" };
            Some(format!("{}{}#[cfg({})]", tabs(indent), comment, v))
        }
        None => None,
    }
}

pub fn write_vec<T: Display>(w: &mut Write, v: &[T]) -> Result<()> {
    for s in v {
        try!(writeln!(w, "{}", s));
    }
    Ok(())
}
