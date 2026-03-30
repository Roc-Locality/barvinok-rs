//! Upstream-aligned wrapper generation for the Rust `barvinok` crate.
//!
//! The generator deliberately follows the same high-level rules that isl uses
//! for its own C++/Python bindings in `isl/interface/generator.cc`:
//!
//! - group raw C functions by isl object type
//! - derive ownership from `__isl_take` / `__isl_keep` / `__isl_give`
//! - normalize overloaded names by stripping argument type suffixes
//! - recognize constructors from context/object-first signatures
//!
//! We stay conservative about what gets emitted. Callback-heavy APIs,
//! unsupported foreign types, and signatures that would not fit the existing
//! Rust lifetime/object model remain manual in the main crate.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Annotation {
    Take,
    Keep,
    Give,
}

#[derive(Debug, Clone)]
struct FunctionArg {
    name: String,
    ty: String,
    annotation: Option<Annotation>,
}

#[derive(Debug, Clone)]
struct FunctionDecl {
    name: String,
    return_ty: String,
    return_annotation: Option<Annotation>,
    args: Vec<FunctionArg>,
    is_constructor: bool,
    is_overload: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum PrinterKind {
    Direct,
    Printer,
    None,
}

#[derive(Debug, Clone, Copy)]
struct TypeConfig {
    output: &'static str,
    source: &'static str,
    rust_type: &'static str,
    c_type: &'static str,
    printer: PrinterKind,
    emit_handle: bool,
    exclude: &'static [&'static str],
}

const TYPE_CONFIGS: &[TypeConfig] = &[
    TypeConfig {
        output: "space",
        source: "space",
        rust_type: "Space",
        c_type: "space",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "aff",
        source: "aff",
        rust_type: "Affine",
        c_type: "aff",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "pw_aff",
        source: "pw_aff",
        rust_type: "PiecewiseAffine",
        c_type: "pw_aff",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "union_pw_aff",
        source: "union_pw_aff",
        rust_type: "UnionPiecewiseAffine",
        c_type: "union_pw_aff",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "multi_aff",
        source: "multi_aff",
        rust_type: "MultiAffine",
        c_type: "multi_aff",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "pw_multi_aff",
        source: "pw_multi_aff",
        rust_type: "PiecewiseMultiAffine",
        c_type: "pw_multi_aff",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "union_pw_multi_aff",
        source: "union_pw_multi_aff",
        rust_type: "UnionPiecewiseMultiAffine",
        c_type: "union_pw_multi_aff",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "multi_pw_aff",
        source: "multi_pw_aff",
        rust_type: "MultiPiecewiseAffine",
        c_type: "multi_pw_aff",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "multi_union_pw_aff",
        source: "multi_union_pw_aff",
        rust_type: "MultiUnionPiecewiseAffine",
        c_type: "multi_union_pw_aff",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "point",
        source: "point",
        rust_type: "Point",
        c_type: "point",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "mat",
        source: "mat",
        rust_type: "Matrix",
        c_type: "mat",
        printer: PrinterKind::None,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "constraint",
        source: "constraint",
        rust_type: "Constraint",
        c_type: "constraint",
        printer: PrinterKind::Printer,
        emit_handle: false,
        exclude: &[],
    },
    TypeConfig {
        output: "ident",
        source: "ident",
        rust_type: "Ident",
        c_type: "id",
        printer: PrinterKind::Direct,
        emit_handle: false,
        exclude: &[],
    },
    TypeConfig {
        output: "multi_id",
        source: "multi_id",
        rust_type: "MultiId",
        c_type: "multi_id",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "local_space",
        source: "local_space",
        rust_type: "LocalSpace",
        c_type: "local_space",
        printer: PrinterKind::Printer,
        emit_handle: false,
        exclude: &[],
    },
    TypeConfig {
        output: "basic_map",
        source: "map",
        rust_type: "BasicMap",
        c_type: "basic_map",
        printer: PrinterKind::Direct,
        emit_handle: false,
        exclude: &[],
    },
    TypeConfig {
        output: "map",
        source: "map",
        rust_type: "Map",
        c_type: "map",
        printer: PrinterKind::Direct,
        emit_handle: false,
        exclude: &[],
    },
    TypeConfig {
        output: "basic_set",
        source: "set",
        rust_type: "BasicSet",
        c_type: "basic_set",
        printer: PrinterKind::Direct,
        emit_handle: false,
        exclude: &[],
    },
    TypeConfig {
        output: "set",
        source: "set",
        rust_type: "Set",
        c_type: "set",
        printer: PrinterKind::Direct,
        emit_handle: false,
        exclude: &[],
    },
    TypeConfig {
        output: "qpolynomial",
        source: "polynomial",
        rust_type: "QuasiPolynomial",
        c_type: "qpolynomial",
        printer: PrinterKind::Printer,
        emit_handle: false,
        exclude: &[],
    },
    TypeConfig {
        output: "pw_qpolynomial",
        source: "polynomial",
        rust_type: "PiecewiseQuasiPolynomial",
        c_type: "pw_qpolynomial",
        printer: PrinterKind::Direct,
        emit_handle: false,
        exclude: &[],
    },
    TypeConfig {
        output: "value",
        source: "value",
        rust_type: "Value",
        c_type: "val",
        printer: PrinterKind::Direct,
        emit_handle: false,
        exclude: &[],
    },
    TypeConfig {
        output: "multi_val",
        source: "multi_val",
        rust_type: "MultiValue",
        c_type: "multi_val",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "vec",
        source: "vec",
        rust_type: "Vector",
        c_type: "vec",
        printer: PrinterKind::Printer,
        emit_handle: false,
        exclude: &[],
    },
    TypeConfig {
        output: "union_map",
        source: "union_map",
        rust_type: "UnionMap",
        c_type: "union_map",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
    TypeConfig {
        output: "union_set",
        source: "union_set",
        rust_type: "UnionSet",
        c_type: "union_set",
        printer: PrinterKind::Direct,
        emit_handle: true,
        exclude: &[],
    },
];

const C_TO_RUST: &[(&str, &str)] = &[
    ("aff", "Affine"),
    ("aff_list", "AffineList"),
    ("basic_map", "BasicMap"),
    ("basic_set", "BasicSet"),
    ("basic_set_list", "BasicSetList"),
    ("constraint", "Constraint"),
    ("constraint_list", "ConstraintList"),
    ("id", "Ident"),
    ("id_list", "IdentList"),
    ("local_space", "LocalSpace"),
    ("map", "Map"),
    ("mat", "Matrix"),
    ("multi_aff", "MultiAffine"),
    ("multi_id", "MultiId"),
    ("multi_pw_aff", "MultiPiecewiseAffine"),
    ("multi_union_pw_aff", "MultiUnionPiecewiseAffine"),
    ("multi_val", "MultiValue"),
    ("point", "Point"),
    ("pw_aff", "PiecewiseAffine"),
    ("pw_aff_list", "PiecewiseAffineList"),
    ("pw_multi_aff", "PiecewiseMultiAffine"),
    ("pw_qpolynomial", "PiecewiseQuasiPolynomial"),
    ("qpolynomial", "QuasiPolynomial"),
    ("set", "Set"),
    ("set_list", "SetList"),
    ("space", "Space"),
    ("term", "Term"),
    ("union_map", "UnionMap"),
    ("union_pw_aff", "UnionPiecewiseAffine"),
    ("union_pw_aff_list", "UnionPiecewiseAffineList"),
    ("union_pw_multi_aff", "UnionPiecewiseMultiAffine"),
    ("val", "Value"),
    ("val_list", "ValueList"),
    ("vec", "Vector"),
    ("union_set", "UnionSet"),
];

// Keep constructor naming explicit instead of burying the object-oriented
// surface in open-coded string rewrites.
const KNOWN_CONSTRUCTOR_RENAMES: &[(&str, &str)] = &[
    ("alloc", "new"),
    ("set_alloc", "set"),
    ("params_alloc", "params"),
    ("read_from_str", "from_str"),
    ("from_aff_list", "from_list"),
    ("from_id_list", "from_list"),
    ("from_pw_aff_list", "from_list"),
    ("from_union_pw_aff_list", "from_list"),
    ("from_val_list", "from_list"),
];

const KNOWN_FUNCTION_RENAMES: &[(&str, &str)] = &[
    ("isl_aff_add", "checked_add"),
    ("isl_aff_div", "checked_div"),
    ("isl_aff_mul", "checked_mul"),
    ("isl_aff_neg", "checked_neg"),
    ("isl_aff_sub", "checked_sub"),
    ("isl_aff_zero_on_domain_space", "zero_on_domain_space"),
    ("isl_basic_map_card", "cardinality"),
    ("isl_basic_set_card", "cardinality"),
    ("isl_constraint_alloc_equality", "new_equality"),
    ("isl_constraint_alloc_inequality", "new_inequality"),
    ("isl_constraint_dim", "dim"),
    ("isl_constraint_get_aff", "get_affine"),
    ("isl_constraint_get_bound", "get_bound_type"),
    ("isl_constraint_get_coefficient_val", "get_coefficient"),
    ("isl_constraint_get_constant_val", "get_constant"),
    ("isl_constraint_get_div", "get_div"),
    ("isl_constraint_get_local_space", "get_local_space"),
    ("isl_constraint_get_space", "get_space"),
    ("isl_constraint_involves_dims", "involves_dims"),
    ("isl_constraint_is_div_constraint", "is_div_constraint"),
    ("isl_constraint_is_equality", "is_equality"),
    ("isl_constraint_is_lower_bound", "is_lower_bound"),
    ("isl_constraint_is_upper_bound", "is_upper_bound"),
    ("isl_constraint_negate", "negate"),
    ("isl_constraint_set_coefficient_si", "set_coefficient_si"),
    ("isl_constraint_set_coefficient_val", "set_coefficient_val"),
    ("isl_constraint_set_constant_si", "set_constant_si"),
    ("isl_constraint_set_constant_val", "set_constant_val"),
    ("isl_id_get_name", "name"),
    ("isl_local_space_dim", "dim"),
    ("isl_local_space_domain", "domain"),
    ("isl_local_space_find_dim_by_name", "find_dim_by_name"),
    ("isl_local_space_flatten_domain", "flatten_domain"),
    ("isl_local_space_flatten_range", "flatten_range"),
    ("isl_local_space_get_dim_id", "get_dim_id"),
    ("isl_local_space_get_dim_name", "get_dim_name"),
    ("isl_local_space_get_div", "get_div"),
    ("isl_local_space_get_space", "get_space"),
    ("isl_local_space_has_dim_id", "has_dim_id"),
    ("isl_local_space_has_dim_name", "has_dim_name"),
    ("isl_local_space_is_params", "is_params"),
    ("isl_local_space_is_set", "is_set"),
    ("isl_local_space_lifting", "lifting"),
    ("isl_local_space_range", "range"),
    ("isl_local_space_set_dim_id", "set_dim_id"),
    ("isl_local_space_set_dim_name", "set_dim_name"),
    ("isl_local_space_set_tuple_id", "set_tuple_id"),
    ("isl_local_space_wrap", "wrap"),
    ("isl_map_card", "cardinality"),
    ("isl_pw_qpolynomial_add", "checked_add"),
    ("isl_pw_qpolynomial_as_qpolynomial", "as_qpolynomial"),
    ("isl_pw_qpolynomial_eval", "eval"),
    ("isl_pw_qpolynomial_isa_qpolynomial", "is_qpolynomial"),
    ("isl_pw_qpolynomial_mul", "checked_mul"),
    ("isl_pw_qpolynomial_n_piece", "num_pieces"),
    ("isl_pw_qpolynomial_neg", "checked_neg"),
    ("isl_pw_qpolynomial_sub", "checked_sub"),
    ("isl_qpolynomial_add", "checked_add"),
    ("isl_qpolynomial_dim", "get_dim"),
    ("isl_qpolynomial_mul", "checked_mul"),
    ("isl_set_card", "cardinality"),
    ("isl_space_params", "into_params"),
    ("isl_qpolynomial_sub", "checked_sub"),
    ("isl_vec_size", "size"),
];

const KNOWN_METHOD_RENAMES: &[(&str, &str)] =
    &[("2exp", "exp2"), ("match", "matches"), ("mod", "modulo")];

const KNOWN_ARG_RENAMES: &[(&str, &str)] = &[("type", "dim_type"), ("mod", "modulo")];

const OPTIONAL_PROJECT_SUFFIXES: &[&str] = &[
    "get_aff",
    "get_bound",
    "get_dim_id",
    "get_div",
    "get_plain_multi_val_if_fixed",
    "get_stride",
    "get_tuple_id",
    "plain_get_val_if_fixed",
];

const OPTIONAL_STRING_SUFFIXES: &[&str] = &["get_dim_name", "get_tuple_name"];

const OPTIONAL_SIZE_SUFFIXES: &[&str] = &["find_dim_by_id", "find_dim_by_name"];

const OPTIONAL_TRANSFORM_FUNCTIONS: &[&str] = &[
    "isl_local_space_domain",
    "isl_local_space_flatten_domain",
    "isl_local_space_flatten_range",
    "isl_local_space_lifting",
    "isl_local_space_range",
    "isl_local_space_wrap",
];

pub fn generate(
    crate_dir: &Path,
    header_roots: &[PathBuf],
    out_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let declarations = parse_headers(header_roots)?;
    let available_sys_symbols = available_sys_symbols(crate_dir)?;
    let generated_dir = out_dir.join("generated");
    fs::create_dir_all(&generated_dir).map_err(|err| {
        Error::new(format!(
            "failed to create {}: {err}",
            generated_dir.display()
        ))
    })?;

    let mut outputs = Vec::with_capacity(TYPE_CONFIGS.len());
    for config in TYPE_CONFIGS {
        let rendered = render_type(
            crate_dir,
            config,
            &declarations,
            available_sys_symbols.as_ref(),
        )?;
        let output_path = generated_dir.join(format!("{}.rs", config.output));
        fs::write(&output_path, rendered).map_err(|err| {
            Error::new(format!("failed to write {}: {err}", output_path.display()))
        })?;
        outputs.push(output_path);
    }
    Ok(outputs)
}

fn parse_headers(header_roots: &[PathBuf]) -> Result<Vec<FunctionDecl>> {
    let mut declarations = Vec::new();
    let mut seen = BTreeSet::new();
    for root in header_roots {
        collect_declarations(root, &mut declarations, &mut seen)?;
    }
    Ok(declarations)
}

fn collect_declarations(
    dir: &Path,
    declarations: &mut Vec<FunctionDecl>,
    seen: &mut BTreeSet<String>,
) -> Result<()> {
    for entry in fs::read_dir(dir)
        .map_err(|err| Error::new(format!("failed to read {}: {err}", dir.display())))?
    {
        let entry =
            entry.map_err(|err| Error::new(format!("failed to read directory entry: {err}")))?;
        let path = entry.path();
        if path.is_dir() {
            collect_declarations(&path, declarations, seen)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("h") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|err| Error::new(format!("failed to read {}: {err}", path.display())))?;
        let text = strip_comments(&text);
        let stripped = strip_non_declaration_lines(&text);
        let expanded = expand_known_declaration_macros(&text);
        for statement in expanded
            .into_iter()
            .chain(stripped.split(';').map(ToString::to_string))
        {
            let statement = normalize_whitespace(&statement);
            if !statement.contains("isl_") || !statement.contains('(') {
                continue;
            }
            if statement.starts_with("typedef ")
                || statement.starts_with("struct ")
                || statement.starts_with("enum ")
            {
                continue;
            }
            if let Ok(decl) = parse_declaration(&statement)
                && decl.name.starts_with("isl_")
            {
                let key = format!(
                    "{}({})->{}",
                    decl.name,
                    decl.args
                        .iter()
                        .map(|arg| arg.ty.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                    decl.return_ty
                );
                if !seen.insert(key) {
                    continue;
                }
                declarations.push(decl);
            }
        }
    }
    Ok(())
}

fn render_type(
    crate_dir: &Path,
    config: &TypeConfig,
    declarations: &[FunctionDecl],
    available_sys_symbols: Option<&BTreeSet<String>>,
) -> Result<String> {
    let existing_methods = existing_method_names(crate_dir, config)?;
    let mut methods = Vec::new();
    for decl in declarations {
        if let Some(available_sys_symbols) = available_sys_symbols
            && !available_sys_symbols.contains(&decl.name)
        {
            continue;
        }
        if config.exclude.contains(&decl.name.as_str()) {
            continue;
        }
        if matches_utility_function(config, decl) {
            continue;
        }
        if let Some(method) = classify_method(config, decl)? {
            methods.push(method);
        }
    }

    let methods = finalize_methods(methods, &existing_methods)?;

    let rust_type = format_ident!("{}", config.rust_type);
    let c_type = format_ident!("{}", config.c_type);
    let method_tokens = methods.iter().map(|method| &method.tokens);

    let handle_tokens = if config.emit_handle {
        match config.printer {
            PrinterKind::Direct => quote! {
                crate::impl_isl_handle!(#rust_type, #c_type);
            },
            PrinterKind::Printer => quote! {
                crate::impl_isl_handle!([printer] #rust_type, #c_type);
            },
            PrinterKind::None => quote! {
                crate::impl_isl_handle!([noprint] #rust_type, #c_type);
            },
        }
    } else {
        quote! {}
    };
    let file_tokens = quote! {
        #handle_tokens

        #[allow(clippy::too_many_arguments)]
        impl<'a> #rust_type<'a> {
            #(#method_tokens)*
        }
    };
    let file = syn::parse2::<syn::File>(file_tokens)
        .map_err(|err| Error::new(format!("failed to render {}: {err}", config.rust_type)))?;

    let mut output = String::new();
    output.push_str(&format!(
        "// Generated from isl headers for `{}`.\n\
         // The naming rules follow isl/interface/generator.cc.\n\
         // Do not edit this file directly.\n\n",
        config.rust_type
    ));
    output.push_str(&prettyplease::unparse(&file));
    Ok(output)
}

#[derive(Debug)]
struct RenderedMethod {
    sort_key: String,
    tokens: TokenStream,
}

#[derive(Debug, Clone, Copy)]
enum MethodKind {
    CtorCtx,
    CtorOwned,
    Transform(Option<&'static str>),
    TransformOpt(Option<&'static str>),
    Project(&'static str),
    ProjectOpt(&'static str),
    Flag,
    Size,
    SizeOpt,
    Str,
    StrOpt,
}

#[derive(Debug, Clone)]
struct ClassifiedMethod {
    name: String,
    raw_suffix: String,
    overload_disambiguator: Option<String>,
    arg_family_disambiguator: Option<String>,
    decl: FunctionDecl,
    kind: MethodKind,
}

#[derive(Debug, Clone)]
struct MethodNaming {
    name: String,
    raw_suffix: String,
    overload_disambiguator: Option<String>,
}

#[derive(Debug, Clone)]
struct NormalizedSuffix {
    name: String,
    disambiguator: Option<String>,
}

fn classify_method(config: &TypeConfig, decl: &FunctionDecl) -> Result<Option<ClassifiedMethod>> {
    if decl.args.iter().any(arg_is_unsupported) {
        return Ok(None);
    }

    let return_object = object_c_type(&decl.return_ty);
    let first_arg = decl.args.first();
    let Some(first_arg) = first_arg else {
        return Ok(None);
    };

    let naming = method_naming(config, decl)?;
    let kind = if is_ctx(&first_arg.ty)
        && return_object == Some(config.c_type)
        && decl.return_annotation == Some(Annotation::Give)
    {
        MethodKind::CtorCtx
    } else if return_object == Some(config.c_type)
        && first_arg.annotation == Some(Annotation::Take)
        && object_c_type(&first_arg.ty).is_some()
        && object_c_type(&first_arg.ty) != Some(config.c_type)
    {
        MethodKind::CtorOwned
    } else if object_c_type(&first_arg.ty) == Some(config.c_type)
        && first_arg.annotation == Some(Annotation::Take)
        && decl.return_annotation == Some(Annotation::Give)
        && return_object == Some(config.c_type)
    {
        if is_optional_transform(decl) {
            MethodKind::TransformOpt(None)
        } else {
            MethodKind::Transform(None)
        }
    } else if object_c_type(&first_arg.ty) == Some(config.c_type)
        && first_arg.annotation == Some(Annotation::Take)
        && decl.return_annotation == Some(Annotation::Give)
    {
        let Some(target) = return_object.and_then(rust_type_for_c) else {
            return Ok(None);
        };
        if is_optional_transform(decl) {
            MethodKind::TransformOpt(Some(target))
        } else {
            MethodKind::Transform(Some(target))
        }
    } else if object_c_type(&first_arg.ty) == Some(config.c_type)
        && first_arg.annotation == Some(Annotation::Keep)
        && decl.return_annotation == Some(Annotation::Give)
    {
        let Some(target) = return_object.and_then(rust_type_for_c) else {
            return Ok(None);
        };
        if is_optional_project(decl) {
            MethodKind::ProjectOpt(target)
        } else {
            MethodKind::Project(target)
        }
    } else if object_c_type(&first_arg.ty) == Some(config.c_type)
        && first_arg.annotation == Some(Annotation::Keep)
        && decl.return_ty == "isl_bool"
    {
        MethodKind::Flag
    } else if object_c_type(&first_arg.ty) == Some(config.c_type)
        && first_arg.annotation == Some(Annotation::Keep)
        && (decl.return_ty == "isl_size" || decl.return_ty == "int")
    {
        if is_optional_size(decl) {
            MethodKind::SizeOpt
        } else if decl.return_ty == "isl_size" {
            MethodKind::Size
        } else {
            return Ok(None);
        }
    } else if object_c_type(&first_arg.ty) == Some(config.c_type)
        && first_arg.annotation == Some(Annotation::Keep)
        && is_string(&decl.return_ty)
    {
        if is_optional_string(decl) {
            MethodKind::StrOpt
        } else {
            MethodKind::Str
        }
    } else {
        return Ok(None);
    };

    Ok(Some(ClassifiedMethod {
        name: naming.name,
        raw_suffix: naming.raw_suffix,
        overload_disambiguator: naming.overload_disambiguator,
        arg_family_disambiguator: arg_family_disambiguator(decl),
        decl: decl.clone(),
        kind,
    }))
}

fn finalize_methods(
    methods: Vec<ClassifiedMethod>,
    existing_methods: &BTreeSet<String>,
) -> Result<Vec<RenderedMethod>> {
    let mut grouped = BTreeMap::<String, Vec<ClassifiedMethod>>::new();
    for method in methods {
        grouped.entry(method.name.clone()).or_default().push(method);
    }

    let mut rendered = Vec::new();
    let mut taken_names = existing_methods.clone();
    for (base_name, mut group) in grouped {
        group.sort_by(|lhs, rhs| lhs.decl.name.cmp(&rhs.decl.name));

        let canonical_index = if group.len() == 1 {
            (!taken_names.contains(&base_name)).then_some(0)
        } else if taken_names.contains(&base_name) {
            None
        } else {
            let unsuffixed = group
                .iter()
                .enumerate()
                .filter_map(|(index, method)| {
                    method.overload_disambiguator.is_none().then_some(index)
                })
                .collect::<Vec<_>>();
            if unsuffixed.len() == 1 {
                Some(unsuffixed[0])
            } else {
                None
            }
        };

        for (index, method) in group.into_iter().enumerate() {
            let final_name = if canonical_index == Some(index) {
                base_name.clone()
            } else {
                unique_method_name(&base_name, &method, &taken_names)
            };

            if existing_methods.contains(&final_name) {
                continue;
            }

            taken_names.insert(final_name.clone());
            rendered.push(render_method(&final_name, &method)?);
        }
    }

    rendered.sort_by(|lhs, rhs| lhs.sort_key.cmp(&rhs.sort_key));
    rendered.dedup_by(|lhs, rhs| {
        lhs.sort_key == rhs.sort_key && lhs.tokens.to_string() == rhs.tokens.to_string()
    });
    Ok(rendered)
}

fn render_method(method_name: &str, method: &ClassifiedMethod) -> Result<RenderedMethod> {
    let tokens = match method.kind {
        MethodKind::CtorCtx => render_ctor_ctx(method_name, &method.decl)?,
        MethodKind::CtorOwned => render_ctor_owned(method_name, &method.decl)?,
        MethodKind::Transform(target) => render_transform(method_name, &method.decl, target)?,
        MethodKind::TransformOpt(target) => {
            render_transform_opt(method_name, &method.decl, target)?
        }
        MethodKind::Project(target) => render_project(method_name, &method.decl, target)?,
        MethodKind::ProjectOpt(target) => render_project_opt(method_name, &method.decl, target)?,
        MethodKind::Flag => render_flag(method_name, &method.decl)?,
        MethodKind::Size => render_size(method_name, &method.decl)?,
        MethodKind::SizeOpt => render_size_opt(method_name, &method.decl)?,
        MethodKind::Str => render_str(method_name, &method.decl)?,
        MethodKind::StrOpt => render_str_opt(method_name, &method.decl)?,
    };
    Ok(RenderedMethod {
        sort_key: format!("{}::{}", method_name, method.decl.name),
        tokens,
    })
}

fn unique_method_name(
    base_name: &str,
    method: &ClassifiedMethod,
    taken_names: &BTreeSet<String>,
) -> String {
    let mut candidates = Vec::new();
    if let Some(disambiguator) = method
        .overload_disambiguator
        .as_deref()
        .or(method.arg_family_disambiguator.as_deref())
    {
        candidates.push(format!("{base_name}_{disambiguator}"));
    }
    if method.raw_suffix != base_name {
        candidates.push(method.raw_suffix.clone());
    }

    for candidate in candidates {
        if !taken_names.contains(&candidate) {
            return candidate;
        }
    }

    let mut index = 2usize;
    loop {
        let fallback = format!("{base_name}_{index}");
        if !taken_names.contains(&fallback) {
            return fallback;
        }
        index += 1;
    }
}

fn arg_family_disambiguator(decl: &FunctionDecl) -> Option<String> {
    let mut families = Vec::new();
    for arg in decl.args.iter().skip(1) {
        if let Some(type_suffix) = overload_type_suffix(arg) {
            families.push(type_suffix.trim_start_matches('_').to_string());
        }
    }
    (!families.is_empty()).then(|| families.join("_"))
}

fn existing_method_names(crate_dir: &Path, config: &TypeConfig) -> Result<BTreeSet<String>> {
    let source_path = crate_dir.join("src").join(format!("{}.rs", config.source));
    let text = fs::read_to_string(&source_path).map_err(|err| {
        Error::new(format!(
            "failed to read existing wrapper source {}: {err}",
            source_path.display()
        ))
    })?;

    let mut names = BTreeSet::new();
    let file = syn::parse_file(&text).map_err(|err| {
        Error::new(format!(
            "failed to parse existing wrapper source {}: {err}",
            source_path.display()
        ))
    })?;
    for item in file.items {
        let syn::Item::Impl(item_impl) = item else {
            continue;
        };
        if item_impl.trait_.is_some() {
            continue;
        }
        if !impl_targets_type(item_impl.self_ty.as_ref(), config.rust_type) {
            continue;
        }
        for impl_item in item_impl.items {
            match impl_item {
                syn::ImplItem::Fn(method) => {
                    names.insert(method.sig.ident.to_string());
                }
                syn::ImplItem::Macro(item_macro) => {
                    collect_macro_method_names(&item_macro.mac, &mut names)?;
                }
                _ => {}
            }
        }
    }

    Ok(names)
}

fn impl_targets_type(self_ty: &syn::Type, rust_type: &str) -> bool {
    match self_ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident == rust_type)
            .unwrap_or(false),
        _ => false,
    }
}

fn collect_macro_method_names(mac: &syn::Macro, names: &mut BTreeSet<String>) -> Result<()> {
    let path = mac
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
        .unwrap_or_default();
    let tokens = mac.tokens.to_string();
    match path.as_str() {
        "isl_ctor" | "isl_transform" | "isl_project" | "isl_transform_opt" | "isl_project_opt" => {
            let regex = Regex::new(r"^(?:\[[^\]]+\]\s*)?([A-Za-z_][A-Za-z0-9_]*)\s*,").unwrap();
            if let Some(capture) = regex.captures(&tokens) {
                names.insert(capture[1].to_string());
            }
        }
        "isl_flag" | "isl_size" | "isl_str" | "isl_size_opt" | "isl_str_opt" => {
            let regex = Regex::new(r"=>\s*([A-Za-z_][A-Za-z0-9_]*)").unwrap();
            if let Some(capture) = regex.captures(&tokens) {
                names.insert(capture[1].to_string());
            }
        }
        _ => {}
    }
    Ok(())
}

fn available_sys_symbols(crate_dir: &Path) -> Result<Option<BTreeSet<String>>> {
    let target_dir = crate_dir
        .parent()
        .map(|parent| parent.join("target"))
        .ok_or_else(|| Error::new("crate_dir has no parent"))?;
    if !target_dir.exists() {
        return Ok(None);
    }

    let mut bindings_files = Vec::new();
    collect_bindings_files(&target_dir, &mut bindings_files)?;
    if bindings_files.is_empty() {
        return Ok(None);
    }

    let symbol_regex = Regex::new(r"pub fn (isl_[A-Za-z0-9_]+)\(")
        .map_err(|err| Error::new(format!("failed to compile symbol regex: {err}")))?;
    let mut symbols = BTreeSet::new();
    for bindings_file in bindings_files {
        let text = fs::read_to_string(&bindings_file).map_err(|err| {
            Error::new(format!(
                "failed to read barvinok-sys bindings {}: {err}",
                bindings_file.display()
            ))
        })?;
        for capture in symbol_regex.captures_iter(&text) {
            symbols.insert(capture[1].to_string());
        }
    }

    Ok(Some(symbols))
}

fn collect_bindings_files(dir: &Path, bindings_files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)
        .map_err(|err| Error::new(format!("failed to read {}: {err}", dir.display())))?
    {
        let entry =
            entry.map_err(|err| Error::new(format!("failed to read directory entry: {err}")))?;
        let path = entry.path();
        if path.is_dir() {
            collect_bindings_files(&path, bindings_files)?;
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("bindings.rs")
            && path.to_string_lossy().contains("barvinok-sys-")
        {
            bindings_files.push(path);
        }
    }
    Ok(())
}

fn method_naming(config: &TypeConfig, decl: &FunctionDecl) -> Result<MethodNaming> {
    if let Some((_, rename)) = KNOWN_FUNCTION_RENAMES
        .iter()
        .find(|(candidate, _)| *candidate == decl.name)
    {
        return Ok(MethodNaming {
            name: (*rename).to_string(),
            raw_suffix: decl.name.clone(),
            overload_disambiguator: None,
        });
    }

    let raw_suffix = method_suffix(config, decl)?;
    let overload_suffix = if decl.is_overload {
        normalize_overload_suffixes(&raw_suffix, &decl.args)
    } else {
        NormalizedSuffix {
            name: raw_suffix.clone(),
            disambiguator: None,
        }
    };

    if overload_suffix.name.is_empty() {
        return Err(Error::new(format!(
            "empty method name after overload normalization for `{}`",
            decl.name
        )));
    }

    let constructor_candidate = decl.is_constructor
        || KNOWN_CONSTRUCTOR_RENAMES
            .iter()
            .any(|(candidate, _)| *candidate == overload_suffix.name)
        || overload_suffix.name.ends_with("_alloc");

    let name = if constructor_candidate {
        if let Some((_, rename)) = KNOWN_CONSTRUCTOR_RENAMES
            .iter()
            .find(|(candidate, _)| *candidate == overload_suffix.name)
        {
            (*rename).to_string()
        } else if overload_suffix.name.ends_with("_alloc") {
            overload_suffix.name[..overload_suffix.name.len() - "_alloc".len()].to_string()
        } else {
            sanitize_method_name(&overload_suffix.name).to_string()
        }
    } else {
        sanitize_method_name(&overload_suffix.name).to_string()
    };

    if name.is_empty() {
        return Err(Error::new(format!(
            "empty method name after sanitization for `{}`",
            decl.name
        )));
    }
    Ok(MethodNaming {
        name,
        raw_suffix,
        overload_disambiguator: overload_suffix.disambiguator,
    })
}

fn normalize_overload_suffixes(name: &str, args: &[FunctionArg]) -> NormalizedSuffix {
    let mut stripped = name.to_string();
    let mut removed_suffixes = Vec::new();
    for arg in args.iter().rev() {
        if let Some(type_suffix) = overload_type_suffix(arg)
            && stripped.ends_with(type_suffix)
        {
            stripped.truncate(stripped.len() - type_suffix.len());
            removed_suffixes.push(type_suffix.trim_start_matches('_').to_string());
        }
    }
    removed_suffixes.reverse();
    NormalizedSuffix {
        name: stripped,
        disambiguator: (!removed_suffixes.is_empty()).then(|| removed_suffixes.join("_")),
    }
}

fn method_suffix(config: &TypeConfig, decl: &FunctionDecl) -> Result<String> {
    let prefix = format!("isl_{}_", config.c_type);
    decl.name
        .strip_prefix(&prefix)
        .map(ToString::to_string)
        .ok_or_else(|| Error::new(format!("{} does not start with {}", decl.name, prefix)))
}

fn overload_type_suffix(arg: &FunctionArg) -> Option<&'static str> {
    if let Some(c_object) = object_c_type(&arg.ty) {
        return match c_object {
            "aff" => Some("_aff"),
            "basic_map" => Some("_basic_map"),
            "basic_set" => Some("_basic_set"),
            "id" => Some("_id"),
            "map" => Some("_map"),
            "multi_aff" => Some("_multi_aff"),
            "multi_id" => Some("_multi_id"),
            "multi_pw_aff" => Some("_multi_pw_aff"),
            "multi_union_pw_aff" => Some("_multi_union_pw_aff"),
            "multi_val" => Some("_multi_val"),
            "point" => Some("_point"),
            "pw_aff" => Some("_pw_aff"),
            "pw_multi_aff" => Some("_pw_multi_aff"),
            "set" => Some("_set"),
            "space" => Some("_space"),
            "union_map" => Some("_union_map"),
            "union_pw_aff" => Some("_union_pw_aff"),
            "union_pw_multi_aff" => Some("_union_pw_multi_aff"),
            "union_set" => Some("_union_set"),
            "val" => Some("_val"),
            _ => None,
        };
    }

    match arg.ty.as_str() {
        "unsigned" | "unsigned int" => Some("_ui"),
        _ => None,
    }
}

fn render_ctor_ctx(method_name: &str, decl: &FunctionDecl) -> Result<TokenStream> {
    let method_ident = format_ident!("{}", method_name);
    let sys_fn_ident = format_ident!("{}", decl.name);
    let mut args = Vec::new();
    for arg in decl.args.iter().skip(1) {
        args.push(render_arg(arg)?);
    }
    Ok(if args.is_empty() {
        quote! { crate::isl_ctor!([ctx] #method_ident, #sys_fn_ident); }
    } else {
        quote! { crate::isl_ctor!([ctx] #method_ident, #sys_fn_ident, #(#args),*); }
    })
}

fn render_ctor_owned(method_name: &str, decl: &FunctionDecl) -> Result<TokenStream> {
    let method_ident = format_ident!("{}", method_name);
    let sys_fn_ident = format_ident!("{}", decl.name);
    let mut rendered = Vec::new();
    let first = decl.args.first().unwrap();
    rendered.push(render_ctor_first_arg(first)?);
    for arg in decl.args.iter().skip(1) {
        rendered.push(render_arg(arg)?);
    }
    Ok(quote! {
        crate::isl_ctor!(#method_ident, #sys_fn_ident, #(#rendered),*);
    })
}

fn render_transform(
    method_name: &str,
    decl: &FunctionDecl,
    target: Option<&str>,
) -> Result<TokenStream> {
    let method_ident = format_ident!("{}", method_name);
    let sys_fn_ident = format_ident!("{}", decl.name);
    let mut args = Vec::new();
    for arg in decl.args.iter().skip(1) {
        args.push(render_arg(arg)?);
    }
    Ok(if let Some(target) = target {
        let target_ty = rust_type_with_lifetime(target)
            .ok_or_else(|| Error::new(format!("unsupported transform target `{target}`")))?;
        if args.is_empty() {
            quote! { crate::isl_transform!([into(#target_ty)] #method_ident, #sys_fn_ident); }
        } else {
            quote! { crate::isl_transform!([into(#target_ty)] #method_ident, #sys_fn_ident, #(#args),*); }
        }
    } else if args.is_empty() {
        quote! { crate::isl_transform!(#method_ident, #sys_fn_ident); }
    } else {
        quote! { crate::isl_transform!(#method_ident, #sys_fn_ident, #(#args),*); }
    })
}

fn render_transform_opt(
    method_name: &str,
    decl: &FunctionDecl,
    target: Option<&str>,
) -> Result<TokenStream> {
    let method_ident = format_ident!("{}", method_name);
    let sys_fn_ident = format_ident!("{}", decl.name);
    let mut args = Vec::new();
    for arg in decl.args.iter().skip(1) {
        args.push(render_arg(arg)?);
    }
    Ok(if let Some(target) = target {
        let target_ty = rust_type_with_lifetime(target)
            .ok_or_else(|| Error::new(format!("unsupported transform target `{target}`")))?;
        if args.is_empty() {
            quote! { crate::isl_transform_opt!([into(#target_ty)] #method_ident, #sys_fn_ident); }
        } else {
            quote! { crate::isl_transform_opt!([into(#target_ty)] #method_ident, #sys_fn_ident, #(#args),*); }
        }
    } else if args.is_empty() {
        quote! { crate::isl_transform_opt!(#method_ident, #sys_fn_ident); }
    } else {
        quote! { crate::isl_transform_opt!(#method_ident, #sys_fn_ident, #(#args),*); }
    })
}

fn render_project(method_name: &str, decl: &FunctionDecl, target: &str) -> Result<TokenStream> {
    let method_ident = format_ident!("{}", method_name);
    let sys_fn_ident = format_ident!("{}", decl.name);
    let target_ty = rust_type_with_lifetime(target)
        .ok_or_else(|| Error::new(format!("unsupported project target `{target}`")))?;
    let mut args = Vec::new();
    for arg in decl.args.iter().skip(1) {
        args.push(render_arg(arg)?);
    }
    Ok(if args.is_empty() {
        quote! { crate::isl_project!([into(#target_ty)] #method_ident, #sys_fn_ident); }
    } else {
        quote! { crate::isl_project!([into(#target_ty)] #method_ident, #sys_fn_ident, #(#args),*); }
    })
}

fn render_project_opt(method_name: &str, decl: &FunctionDecl, target: &str) -> Result<TokenStream> {
    let method_ident = format_ident!("{}", method_name);
    let sys_fn_ident = format_ident!("{}", decl.name);
    let target_ty = rust_type_with_lifetime(target)
        .ok_or_else(|| Error::new(format!("unsupported project target `{target}`")))?;
    let mut args = Vec::new();
    for arg in decl.args.iter().skip(1) {
        args.push(render_arg(arg)?);
    }
    Ok(if args.is_empty() {
        quote! { crate::isl_project_opt!([into(#target_ty)] #method_ident, #sys_fn_ident); }
    } else {
        quote! { crate::isl_project_opt!([into(#target_ty)] #method_ident, #sys_fn_ident, #(#args),*); }
    })
}

fn render_flag(method_name: &str, decl: &FunctionDecl) -> Result<TokenStream> {
    let method_ident = format_ident!("{}", method_name);
    let sys_fn_ident = format_ident!("{}", decl.name);
    let mut args = Vec::new();
    for arg in decl.args.iter().skip(1) {
        args.push(render_arg(arg)?);
    }
    Ok(if args.is_empty() {
        quote! { crate::isl_flag!(#sys_fn_ident => #method_ident); }
    } else {
        quote! { crate::isl_flag!(#sys_fn_ident => #method_ident, #(#args),*); }
    })
}

fn render_size(method_name: &str, decl: &FunctionDecl) -> Result<TokenStream> {
    let method_ident = format_ident!("{}", method_name);
    let sys_fn_ident = format_ident!("{}", decl.name);
    let mut args = Vec::new();
    for arg in decl.args.iter().skip(1) {
        args.push(render_arg(arg)?);
    }
    Ok(if args.is_empty() {
        quote! { crate::isl_size!(#sys_fn_ident => #method_ident); }
    } else {
        quote! { crate::isl_size!(#sys_fn_ident => #method_ident, #(#args),*); }
    })
}

fn render_size_opt(method_name: &str, decl: &FunctionDecl) -> Result<TokenStream> {
    let method_ident = format_ident!("{}", method_name);
    let sys_fn_ident = format_ident!("{}", decl.name);
    let mut args = Vec::new();
    for arg in decl.args.iter().skip(1) {
        args.push(render_arg(arg)?);
    }
    Ok(if args.is_empty() {
        quote! { crate::isl_size_opt!(#sys_fn_ident => #method_ident); }
    } else {
        quote! { crate::isl_size_opt!(#sys_fn_ident => #method_ident, #(#args),*); }
    })
}

fn render_str(method_name: &str, decl: &FunctionDecl) -> Result<TokenStream> {
    let method_ident = format_ident!("{}", method_name);
    let sys_fn_ident = format_ident!("{}", decl.name);
    let mut args = Vec::new();
    for arg in decl.args.iter().skip(1) {
        args.push(render_arg(arg)?);
    }
    Ok(if args.is_empty() {
        quote! { crate::isl_str!(#sys_fn_ident => #method_ident); }
    } else {
        quote! { crate::isl_str!(#sys_fn_ident => #method_ident, #(#args),*); }
    })
}

fn render_str_opt(method_name: &str, decl: &FunctionDecl) -> Result<TokenStream> {
    let method_ident = format_ident!("{}", method_name);
    let sys_fn_ident = format_ident!("{}", decl.name);
    let mut args = Vec::new();
    for arg in decl.args.iter().skip(1) {
        args.push(render_arg(arg)?);
    }
    Ok(if args.is_empty() {
        quote! { crate::isl_str_opt!(#sys_fn_ident => #method_ident); }
    } else {
        quote! { crate::isl_str_opt!(#sys_fn_ident => #method_ident, #(#args),*); }
    })
}

fn render_ctor_first_arg(arg: &FunctionArg) -> Result<TokenStream> {
    let arg_name = sanitize_arg_name(&arg.name);
    if arg_name.is_empty() {
        return Err(Error::new(format!(
            "empty constructor argument name for type `{}`",
            arg.ty
        )));
    }
    let object = object_c_type(&arg.ty)
        .and_then(rust_type_for_c)
        .ok_or_else(|| {
            Error::new(format!(
                "unsupported constructor argument type `{}`",
                arg.ty
            ))
        })?;
    let arg_ident = format_ident!("{}", arg_name);
    let object_path = rust_type_path(object).ok_or_else(|| {
        Error::new(format!(
            "unsupported constructor argument type `{}`",
            arg.ty
        ))
    })?;
    Ok(quote! { #arg_ident: #object_path<'a> })
}

fn render_arg(arg: &FunctionArg) -> Result<TokenStream> {
    let rust_name = sanitize_arg_name(&arg.name);
    if rust_name.is_empty() {
        return Err(Error::new(format!(
            "empty argument name for `{}` of type `{}`",
            arg.name, arg.ty
        )));
    }
    let arg_ident = format_ident!("{}", rust_name);

    if let Some(c_object) = object_c_type(&arg.ty) {
        let rust_object = rust_type_for_c(c_object)
            .ok_or_else(|| Error::new(format!("unsupported object type `{}`", arg.ty)))?;
        let rust_object_path = rust_type_path(rust_object)
            .ok_or_else(|| Error::new(format!("unsupported object type `{}`", arg.ty)))?;
        return Ok(match arg.annotation {
            Some(Annotation::Take) => quote! { [managed] #arg_ident: #rust_object_path<'a> },
            Some(Annotation::Keep) => quote! { [ref] #arg_ident: &#rust_object_path<'a> },
            Some(Annotation::Give) | None => {
                return Err(Error::new(format!(
                    "unexpected object annotation for `{}`",
                    arg.name
                )));
            }
        });
    }

    if is_string(&arg.ty) {
        return Ok(quote! { [str] #arg_ident: &str });
    }

    match arg.ty.as_str() {
        "enum isl_dim_type" => Ok(quote! { [cast(u32)] #arg_ident: DimType }),
        "unsigned" | "unsigned int" => Ok(quote! { [trivial] #arg_ident: u32 }),
        "int" => Ok(quote! { [trivial] #arg_ident: i32 }),
        "long" => Ok(quote! { [trivial] #arg_ident: i64 }),
        "unsigned long" => Ok(quote! { [trivial] #arg_ident: u64 }),
        "size_t" => Ok(quote! { [trivial] #arg_ident: usize }),
        other => Err(Error::new(format!("unsupported scalar type `{other}`"))),
    }
}

fn sanitize_method_name(name: &str) -> &str {
    KNOWN_METHOD_RENAMES
        .iter()
        .find_map(|(candidate, replacement)| (*candidate == name).then_some(*replacement))
        .unwrap_or(name)
}

fn sanitize_arg_name(name: &str) -> &str {
    KNOWN_ARG_RENAMES
        .iter()
        .find_map(|(candidate, replacement)| (*candidate == name).then_some(*replacement))
        .unwrap_or(name)
}

fn has_suffix_in(decl: &FunctionDecl, suffixes: &[&str]) -> bool {
    suffixes.iter().any(|suffix| decl.name.ends_with(suffix))
}

fn is_optional_project(decl: &FunctionDecl) -> bool {
    has_suffix_in(decl, OPTIONAL_PROJECT_SUFFIXES)
}

fn is_optional_string(decl: &FunctionDecl) -> bool {
    has_suffix_in(decl, OPTIONAL_STRING_SUFFIXES)
}

fn is_optional_size(decl: &FunctionDecl) -> bool {
    has_suffix_in(decl, OPTIONAL_SIZE_SUFFIXES)
}

fn is_optional_transform(decl: &FunctionDecl) -> bool {
    OPTIONAL_TRANSFORM_FUNCTIONS.contains(&decl.name.as_str())
}

fn matches_utility_function(config: &TypeConfig, decl: &FunctionDecl) -> bool {
    let prefix = format!("isl_{}_", config.c_type);
    if !decl.name.starts_with(&prefix) {
        return true;
    }
    let suffix = decl.name.trim_start_matches(&prefix);
    matches!(
        suffix,
        "copy" | "free" | "get_ctx" | "to_str" | "dump" | "print_internal"
    )
}

fn arg_is_unsupported(arg: &FunctionArg) -> bool {
    if arg.ty.contains("(*)")
        || arg.ty.contains("(*")
        || arg.ty.contains("**")
        || arg.ty.contains("FILE *")
        || arg.ty.contains("mpz_t")
        || arg.ty.contains("mpq_t")
        || arg.ty == "void *"
        || arg.ty == "const void *"
    {
        return true;
    }
    if let Some(object) = object_c_type(&arg.ty) {
        return object != "ctx" && rust_type_for_c(object).is_none();
    }
    if arg.ty.contains('*') && !is_ctx(&arg.ty) && !is_string(&arg.ty) {
        return true;
    }
    false
}

fn rust_type_for_c(c_type: &str) -> Option<&'static str> {
    C_TO_RUST
        .iter()
        .find_map(|(candidate, rust)| (*candidate == c_type).then_some(*rust))
}

fn rust_type_path(rust_type: &str) -> Option<TokenStream> {
    match rust_type {
        "Affine" => Some(quote!(crate::aff::Affine)),
        "AffineList" => Some(quote!(crate::list::AffineList)),
        "BasicMap" => Some(quote!(crate::map::BasicMap)),
        "BasicSet" => Some(quote!(crate::set::BasicSet)),
        "BasicSetList" => Some(quote!(crate::list::BasicSetList)),
        "Constraint" => Some(quote!(crate::constraint::Constraint)),
        "ConstraintList" => Some(quote!(crate::list::ConstraintList)),
        "Ident" => Some(quote!(crate::ident::Ident)),
        "IdentList" => Some(quote!(crate::list::IdentList)),
        "LocalSpace" => Some(quote!(crate::local_space::LocalSpace)),
        "Map" => Some(quote!(crate::map::Map)),
        "Matrix" => Some(quote!(crate::mat::Matrix)),
        "MultiAffine" => Some(quote!(crate::multi_aff::MultiAffine)),
        "MultiId" => Some(quote!(crate::multi_id::MultiId)),
        "MultiPiecewiseAffine" => Some(quote!(crate::multi_pw_aff::MultiPiecewiseAffine)),
        "MultiUnionPiecewiseAffine" => {
            Some(quote!(crate::multi_union_pw_aff::MultiUnionPiecewiseAffine))
        }
        "MultiValue" => Some(quote!(crate::multi_val::MultiValue)),
        "PiecewiseQuasiPolynomial" => Some(quote!(crate::polynomial::PiecewiseQuasiPolynomial)),
        "PiecewiseAffine" => Some(quote!(crate::pw_aff::PiecewiseAffine)),
        "PiecewiseAffineList" => Some(quote!(crate::list::PiecewiseAffineList)),
        "PiecewiseMultiAffine" => Some(quote!(crate::pw_multi_aff::PiecewiseMultiAffine)),
        "Point" => Some(quote!(crate::point::Point)),
        "QuasiPolynomial" => Some(quote!(crate::polynomial::QuasiPolynomial)),
        "Set" => Some(quote!(crate::set::Set)),
        "SetList" => Some(quote!(crate::list::SetList)),
        "Space" => Some(quote!(crate::space::Space)),
        "Term" => Some(quote!(crate::polynomial::Term)),
        "UnionMap" => Some(quote!(crate::union_map::UnionMap)),
        "UnionPiecewiseAffine" => Some(quote!(crate::union_pw_aff::UnionPiecewiseAffine)),
        "UnionPiecewiseAffineList" => Some(quote!(crate::list::UnionPiecewiseAffineList)),
        "UnionPiecewiseMultiAffine" => {
            Some(quote!(crate::union_pw_multi_aff::UnionPiecewiseMultiAffine))
        }
        "UnionSet" => Some(quote!(crate::union_set::UnionSet)),
        "Value" => Some(quote!(crate::value::Value)),
        "ValueList" => Some(quote!(crate::list::ValueList)),
        "Vector" => Some(quote!(crate::vec::Vector)),
        _ => None,
    }
}

fn rust_type_with_lifetime(rust_type: &str) -> Option<TokenStream> {
    let rust_type_path = rust_type_path(rust_type)?;
    Some(quote!(#rust_type_path<'a>))
}

fn object_c_type(ty: &str) -> Option<&str> {
    let trimmed = ty.trim();
    let trimmed = trimmed.strip_prefix("const ").unwrap_or(trimmed);
    let trimmed = trimmed.strip_suffix('*')?.trim();
    trimmed.strip_prefix("isl_")
}

fn is_ctx(ty: &str) -> bool {
    ty == "isl_ctx *"
}

fn is_string(ty: &str) -> bool {
    ty == "const char *" || ty == "char *"
}

fn strip_comments(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn expand_known_declaration_macros(input: &str) -> Vec<String> {
    let mut declarations = Vec::new();
    let multi_decl = Regex::new(r"(?m)^\s*ISL_DECLARE_MULTI\(([^)]+)\)\s*$").unwrap();
    for captures in multi_decl.captures_iter(input) {
        declarations.extend(expand_isl_declare_multi(captures[1].trim()));
    }
    declarations
}

fn expand_isl_declare_multi(base: &str) -> Vec<String> {
    let multi = format!("isl_multi_{base}");
    let element = format!("isl_{base}");
    let list = format!("isl_{base}_list");
    vec![
        format!("__isl_export __isl_give isl_space *{multi}_get_space(__isl_keep {multi} *multi)"),
        format!("__isl_give isl_space *{multi}_get_domain_space(__isl_keep {multi} *multi)"),
        format!("__isl_export __isl_give {list} *{multi}_get_list(__isl_keep {multi} *multi)"),
        format!(
            "__isl_constructor __isl_give {multi} *{multi}_from_{base}_list(__isl_take isl_space *space, __isl_take {list} *list)"
        ),
        format!(
            "__isl_export isl_bool {multi}_plain_is_equal(__isl_keep {multi} *multi1, __isl_keep {multi} *multi2)"
        ),
        format!("__isl_export isl_size {multi}_size(__isl_keep {multi} *multi)"),
        format!(
            "__isl_export __isl_give {element} *{multi}_get_at(__isl_keep {multi} *multi, int pos)"
        ),
        format!("__isl_give {element} *{multi}_get_{base}(__isl_keep {multi} *multi, int pos)"),
        format!(
            "__isl_export __isl_give {multi} *{multi}_set_at(__isl_take {multi} *multi, int pos, __isl_take {element} *el)"
        ),
        format!(
            "__isl_give {multi} *{multi}_set_{base}(__isl_take {multi} *multi, int pos, __isl_take {element} *el)"
        ),
        format!(
            "__isl_give {multi} *{multi}_range_splice(__isl_take {multi} *multi1, unsigned pos, __isl_take {multi} *multi2)"
        ),
        format!("__isl_give {multi} *{multi}_flatten_range(__isl_take {multi} *multi)"),
        format!(
            "__isl_export __isl_give {multi} *{multi}_flat_range_product(__isl_take {multi} *multi1, __isl_take {multi} *multi2)"
        ),
        format!(
            "__isl_export __isl_give {multi} *{multi}_range_product(__isl_take {multi} *multi1, __isl_take {multi} *multi2)"
        ),
        format!("__isl_give {multi} *{multi}_factor_range(__isl_take {multi} *multi)"),
        format!("isl_bool {multi}_range_is_wrapping(__isl_keep {multi} *multi)"),
        format!("__isl_give {multi} *{multi}_range_factor_domain(__isl_take {multi} *multi)"),
        format!("__isl_give {multi} *{multi}_range_factor_range(__isl_take {multi} *multi)"),
        format!(
            "__isl_give {multi} *{multi}_align_params(__isl_take {multi} *multi, __isl_take isl_space *model)"
        ),
        format!("__isl_give {multi} *{multi}_from_range(__isl_take {multi} *multi)"),
    ]
}

fn strip_non_declaration_lines(input: &str) -> String {
    input
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !(trimmed.starts_with('#')
                || trimmed.starts_with("ISL_")
                || trimmed == "extern \"C\" {"
                || trimmed == "{"
                || trimmed == "}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_declaration(input: &str) -> std::result::Result<FunctionDecl, String> {
    let open = input.find('(').ok_or_else(|| "missing `(`".to_string())?;
    let close = input.rfind(')').ok_or_else(|| "missing `)`".to_string())?;
    let prefix = input[..open].trim();
    let args = input[open + 1..close].trim();
    let flags = extract_flags(prefix);

    let name_start = prefix
        .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .map(|idx| idx + 1)
        .unwrap_or(0);
    let name = prefix[name_start..].trim().to_string();
    let return_prefix = prefix[..name_start].trim();
    let (return_ty, return_annotation) = strip_annotations(return_prefix);

    let args = if args == "void" || args.is_empty() {
        Vec::new()
    } else {
        split_top_level(args, ',')
            .into_iter()
            .map(|arg| parse_argument(&arg))
            .collect::<std::result::Result<Vec<_>, _>>()?
    };

    Ok(FunctionDecl {
        name,
        return_ty,
        return_annotation,
        args,
        is_constructor: flags.is_constructor,
        is_overload: flags.is_overload,
    })
}

fn parse_argument(input: &str) -> std::result::Result<FunctionArg, String> {
    let trimmed = input.trim();
    let name_start = trimmed
        .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .map(|idx| idx + 1)
        .ok_or_else(|| format!("missing argument name in `{trimmed}`"))?;
    let name = trimmed[name_start..].trim().to_string();
    let (ty, annotation) = strip_annotations(trimmed[..name_start].trim());
    Ok(FunctionArg {
        name,
        ty,
        annotation,
    })
}

#[derive(Debug, Default, Clone, Copy)]
struct Flags {
    is_constructor: bool,
    is_overload: bool,
}

fn extract_flags(input: &str) -> Flags {
    Flags {
        is_constructor: input.contains("__isl_constructor"),
        is_overload: input.contains("__isl_overload"),
    }
}

fn strip_annotations(input: &str) -> (String, Option<Annotation>) {
    let annotation = if input.contains("__isl_take") {
        Some(Annotation::Take)
    } else if input.contains("__isl_keep") {
        Some(Annotation::Keep)
    } else if input.contains("__isl_give") {
        Some(Annotation::Give)
    } else {
        None
    };

    let stripped = normalize_whitespace(
        &input
            .replace("__isl_take", "")
            .replace("__isl_keep", "")
            .replace("__isl_give", "")
            .replace("__isl_null", "")
            .replace("__isl_constructor", "")
            .replace("__isl_export", "")
            .replace("__isl_overload", ""),
    );
    (stripped, annotation)
}

fn split_top_level(input: &str, separator: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0usize;
    let mut angle_depth = 0usize;

    for ch in input.chars() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '<' => angle_depth += 1,
            '>' => angle_depth = angle_depth.saturating_sub(1),
            _ => {}
        }

        if ch == separator && paren_depth == 0 && angle_depth == 0 {
            parts.push(current.trim().to_string());
            current.clear();
            continue;
        }
        current.push(ch);
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn strips_upstream_overload_suffixes() {
        let decl = FunctionDecl {
            name: "isl_space_add_named_tuple_id_ui".into(),
            return_ty: "isl_space *".into(),
            return_annotation: Some(Annotation::Give),
            args: vec![
                FunctionArg {
                    name: "space".into(),
                    ty: "isl_space *".into(),
                    annotation: Some(Annotation::Take),
                },
                FunctionArg {
                    name: "id".into(),
                    ty: "isl_id *".into(),
                    annotation: Some(Annotation::Take),
                },
                FunctionArg {
                    name: "dim".into(),
                    ty: "unsigned int".into(),
                    annotation: None,
                },
            ],
            is_constructor: false,
            is_overload: true,
        };

        let naming = method_naming(&TYPE_CONFIGS[0], &decl).unwrap();
        assert_eq!(naming.name, "add_named_tuple");
        assert_eq!(naming.overload_disambiguator.as_deref(), Some("id_ui"));
    }

    #[test]
    fn disambiguates_overload_groups_without_canonical_base() {
        let group = vec![
            ClassifiedMethod {
                name: "preimage_domain".into(),
                raw_suffix: "preimage_domain_multi_aff".into(),
                overload_disambiguator: Some("multi_aff".into()),
                arg_family_disambiguator: Some("multi_aff".into()),
                decl: FunctionDecl {
                    name: "isl_map_preimage_domain_multi_aff".into(),
                    return_ty: "isl_map *".into(),
                    return_annotation: Some(Annotation::Give),
                    args: vec![
                        FunctionArg {
                            name: "map".into(),
                            ty: "isl_map *".into(),
                            annotation: Some(Annotation::Take),
                        },
                        FunctionArg {
                            name: "ma".into(),
                            ty: "isl_multi_aff *".into(),
                            annotation: Some(Annotation::Take),
                        },
                    ],
                    is_constructor: false,
                    is_overload: true,
                },
                kind: MethodKind::Transform(None),
            },
            ClassifiedMethod {
                name: "preimage_domain".into(),
                raw_suffix: "preimage_domain_pw_multi_aff".into(),
                overload_disambiguator: Some("pw_multi_aff".into()),
                arg_family_disambiguator: Some("pw_multi_aff".into()),
                decl: FunctionDecl {
                    name: "isl_map_preimage_domain_pw_multi_aff".into(),
                    return_ty: "isl_map *".into(),
                    return_annotation: Some(Annotation::Give),
                    args: vec![
                        FunctionArg {
                            name: "map".into(),
                            ty: "isl_map *".into(),
                            annotation: Some(Annotation::Take),
                        },
                        FunctionArg {
                            name: "pma".into(),
                            ty: "isl_pw_multi_aff *".into(),
                            annotation: Some(Annotation::Take),
                        },
                    ],
                    is_constructor: false,
                    is_overload: true,
                },
                kind: MethodKind::Transform(None),
            },
        ];

        let methods = finalize_methods(group, &BTreeSet::new()).unwrap();
        let names = methods
            .iter()
            .map(|method| method.sort_key.split("::").next().unwrap().to_string())
            .collect::<BTreeSet<_>>();
        assert!(names.contains("preimage_domain_multi_aff"));
        assert!(names.contains("preimage_domain_pw_multi_aff"));
        assert!(!names.contains("preimage_domain"));
    }

    #[test]
    fn parses_headers_and_renders_generated_modules() {
        let root = workspace_root();
        let crate_dir = root.join("barvinok");
        let header_roots = vec![
            root.join("barvinok-sys")
                .join("barvinok")
                .join("isl")
                .join("include")
                .join("isl"),
            root.join("barvinok-sys").join("barvinok").join("barvinok"),
        ];
        let out_dir = root.join("target").join("barvinok-gen-test");
        let _ = fs::remove_dir_all(&out_dir);
        let outputs = generate(&crate_dir, &header_roots, &out_dir).unwrap();
        assert!(outputs.iter().any(|path| path.ends_with("space.rs")));
        assert!(outputs.iter().any(|path| path.ends_with("basic_map.rs")));
        assert!(outputs.iter().any(|path| path.ends_with("basic_set.rs")));
        assert!(outputs.iter().any(|path| path.ends_with("local_space.rs")));
        assert!(outputs.iter().any(|path| path.ends_with("value.rs")));
        assert!(outputs.iter().any(|path| path.ends_with("vec.rs")));
        let space = fs::read_to_string(out_dir.join("generated").join("space.rs")).unwrap();
        let aff = fs::read_to_string(out_dir.join("generated").join("aff.rs")).unwrap();
        let basic_map = fs::read_to_string(out_dir.join("generated").join("basic_map.rs")).unwrap();
        let local_space =
            fs::read_to_string(out_dir.join("generated").join("local_space.rs")).unwrap();
        let map = fs::read_to_string(out_dir.join("generated").join("map.rs")).unwrap();
        let multi_aff = fs::read_to_string(out_dir.join("generated").join("multi_aff.rs")).unwrap();
        let multi_pw_aff =
            fs::read_to_string(out_dir.join("generated").join("multi_pw_aff.rs")).unwrap();
        let multi_val = fs::read_to_string(out_dir.join("generated").join("multi_val.rs")).unwrap();
        let value = fs::read_to_string(out_dir.join("generated").join("value.rs")).unwrap();
        let vector = fs::read_to_string(out_dir.join("generated").join("vec.rs")).unwrap();
        let normalized: String = space.chars().filter(|ch| !ch.is_whitespace()).collect();
        let value_normalized: String = value.chars().filter(|ch| !ch.is_whitespace()).collect();
        let basic_map_normalized: String =
            basic_map.chars().filter(|ch| !ch.is_whitespace()).collect();
        let local_space_normalized: String = local_space
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect();
        let map_normalized: String = map.chars().filter(|ch| !ch.is_whitespace()).collect();
        let aff_normalized: String = aff.chars().filter(|ch| !ch.is_whitespace()).collect();
        let multi_aff_normalized: String =
            multi_aff.chars().filter(|ch| !ch.is_whitespace()).collect();
        let multi_pw_aff_normalized: String = multi_pw_aff
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect();
        let multi_val_normalized: String =
            multi_val.chars().filter(|ch| !ch.is_whitespace()).collect();
        let vector_normalized: String = vector.chars().filter(|ch| !ch.is_whitespace()).collect();
        assert!(normalized.contains("impl<'a>Space<'a>{"));
        assert!(normalized.contains("crate::isl_ctor!([ctx]new,isl_space_alloc"));
        assert!(aff_normalized.contains("checked_add"));
        assert!(basic_map_normalized.contains("cardinality"));
        assert!(local_space_normalized.contains("crate::isl_project_opt!"));
        assert!(local_space_normalized.contains("crate::isl_transform_opt!"));
        assert!(map_normalized.contains("cardinality"));
        assert!(
            multi_aff_normalized.contains("crate::isl_ctor!(from_list,isl_multi_aff_from_aff_list")
        );
        assert!(multi_aff_normalized.contains("crate::isl_size!(isl_multi_aff_size=>size"));
        assert!(multi_aff_normalized.contains(
            "crate::isl_project!([into(crate::list::AffineList<'a>)]get_list,isl_multi_aff_get_list"
        ));
        assert!(multi_pw_aff_normalized.contains("crate::isl_project!([into(crate::list::PiecewiseAffineList<'a>)]get_list,isl_multi_pw_aff_get_list"));
        assert!(multi_val_normalized.contains(
            "crate::isl_project!([into(crate::list::ValueList<'a>)]get_list,isl_multi_val_get_list"
        ));
        assert!(normalized.contains("matches"));
        assert!(value_normalized.contains("crate::isl_ctor!([ctx]from_str,isl_val_read_from_str"));
        assert!(value_normalized.contains("crate::isl_ctor!([ctx]zero,isl_val_zero"));
        assert!(vector_normalized.contains("impl<'a>Vector<'a>{"));
    }

    #[test]
    fn sanitizes_rust_keywords() {
        assert_eq!(sanitize_arg_name("type"), "dim_type");
        assert_eq!(sanitize_arg_name("mod"), "modulo");
        assert_eq!(sanitize_method_name("match"), "matches");
        assert_eq!(sanitize_method_name("mod"), "modulo");
    }

    #[test]
    fn strips_isl_declare_macros_before_first_declaration() {
        let input = r#"
            ISL_DECLARE_MULTI(val)
            ISL_DECLARE_MULTI_ZERO(val)
            __isl_export __isl_give isl_val *isl_val_zero(isl_ctx *ctx);
        "#;
        let stripped = strip_non_declaration_lines(&strip_comments(input));
        let declarations = stripped
            .split(';')
            .map(normalize_whitespace)
            .filter(|statement| statement.contains("isl_") && statement.contains('('))
            .filter_map(|statement| parse_declaration(&statement).ok())
            .collect::<Vec<_>>();

        assert!(declarations.iter().any(|decl| decl.name == "isl_val_zero"));
    }

    #[test]
    fn expands_isl_declare_multi_macros() {
        let input = r#"
            ISL_DECLARE_MULTI(aff)
        "#;
        let expanded = expand_known_declaration_macros(input);
        let expanded = expanded
            .into_iter()
            .map(|statement| normalize_whitespace(&statement))
            .collect::<Vec<_>>();

        assert!(
            expanded
                .iter()
                .any(|statement| statement.contains("isl_multi_aff_get_list"))
        );
        assert!(
            expanded
                .iter()
                .any(|statement| statement.contains("isl_multi_aff_from_aff_list"))
        );
        assert!(
            expanded
                .iter()
                .any(|statement| statement.contains("isl_multi_aff_set_at"))
        );
    }
}
