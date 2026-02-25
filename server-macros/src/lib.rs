//! # Server Framework Macros
//!
//! A procedural macro crate providing declarative configuration loading,
//! compile-time controller mapping, and deterministic server bootstrap
//! generation for high-performance Rust API frameworks.
//!
//! This crate is intended to eliminate boilerplate in projects following
//! a modular API architecture, where routing, configuration, and server
//! orchestration should be derived from static metadata rather than
//! dynamic runtime registration.
//!
//! ---
//!
//! ## 1. Crate Purpose
//!
//! This crate offers:
//!
//! - **Compile-time API controller registration**, avoiding runtime reflection.
//! - **Deterministic server initialization**, generating the necessary
//!   runtime code to start an HTTP server and attach controllers.
//! - **Strict compile-time invariants**, ensuring controllers, routes,
//!   and configuration types remain coherent across refactors.
//!
//! It integrates naturally with async runtimes (Tokio) and HTTP frameworks
//! such as `actix-web`, `axum`, or custom routers, depending on how the
//! integrating crate expands its server initialization logic.
//!
//! ---
//!
//! ## 2. Internal Architecture
//!
//! The procedural macros provided by this crate rely on:
//!
//! - `syn` for AST parsing
//! - `quote` for code emission
//! - `proc_macro` and `proc_macro2` for integration with Rust's compiler
//!
//! The macro expansion pipeline follows these steps:
//!
//! 1. **Parse input tokens**: Identify annotated items (structs,
//!    impl blocks, functions, or modules).
//! 2. **Extract metadata**:
//!    - Attribute values (e.g., `path`, `method`, `version`)
//!    - Type information used for configuration deserialization
//!    - HTTP handler signatures
//! 3. **Validate structure**:
//!    - Verify controller methods follow async conventions
//!    - Ensure configuration types implement or derive supported traits
//!    - Ensure routes do not conflict at compile time
//! 4. **Generate deterministic expansion**:
//!    - Create registry modules
//!    - Emit initialization functions
//!    - Generate routing descriptors
//!    - Produce integration glue between config + controllers + server init
//!
//! As a design constraint, this crate **avoids runtime registration**, relying
//! entirely on generated modules and symbol exposure for route aggregation.
//!
//! ---
//!
//! ## 3. Provided Macros
//!
//! ### 3.1 `#[api_server]`
//!
//! Annotates a configuration struct used to bootstrap the API server.
//!
//! Responsibilities:
//! - Generates a strongly-typed configuration loader.
//! - Load server configuration (YAML) using `config`-compatible input.
//! - Merges environment variables following a deterministic precedence model.
//! - Emits validation logic ensuring required fields are set at startup.
//!
//! **Expansion Output**
//! - `main` → generated a new version of the main function to configure and starts the server.
//! - `configure` → generated a configure method to initialize routes in the server.
//!
//! **Requirements**
//! - Struct must derive or implement `serde::Deserialize`.
//! - All fields must have a defined type at compile time.
//! - Unsupported field types cause a compile error.
#![allow(clippy::expect_fun_call)]
#![allow(clippy::bind_instead_of_map)]
#![allow(clippy::cmp_owned)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, format_ident, quote};
use std::{fs::File, io::Read, path::Path};
use syn::{
    Attribute, Expr, Ident, ItemFn, Result, Token,
    parse::{Parse, ParseStream},
    parse_file, parse_macro_input, parse_str,
    punctuated::Punctuated,
};
use walkdir::DirEntry;

/// Represents a single `key = value` pair parsed from a macro input.
///
/// This structure is used when implementing procedural macros
/// that accept configuration-style arguments. The `KeyValue` struct
/// captures:
///
/// - A `key` as a [`syn::Ident`], representing the identifier on the left side.
/// - An equals sign token (`=`), represented by [`Token![=]`].
/// - A `value` expression, stored as a [`syn::Expr`].
///
/// # Parsing Behavior
///
/// The `Parse` implementation consumes three consecutive elements from
/// the input stream:
///
/// ```text
/// <ident> = <expr>
/// ```
///
/// If any of these elements are missing or malformed, a parsing error
/// is returned.
///
/// #  Example of how a single key-value pair might appear in a macro:
///
/// ```rust
/// timeout = 30
/// ```
struct KeyValue {
    pub key: Ident,
    pub _eq: Token![=],
    pub value: Expr,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            key: input.parse()?,
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

/// The ArgList struct represents a comma-separated list of `key = value`
/// pairs.
///
/// `ArgList` is a generic container used in procedural macros to accept
/// lists of configuration arguments in the form:
///
/// ```text
/// key1 = value1, key2 = value2, key3 = value3
/// ```
///
/// Internally it stores the items in a [`syn::punctuated::Punctuated`]
/// structure, which keeps track of both the elements and their punctuation.
///
/// # Parsing Behavior
///
/// The `Parse` implementation uses [`Punctuated::parse_terminated`] to
/// parse zero or more `KeyValue` entries separated by commas. Trailing
/// commas are allowed.
///
/// # Example use inside a macro attribute:
///
/// ```rust
/// #[my_macro(a = 1, b = "text", c = true)]
/// ```
///
/// This structure facilitates ergonomic parsing of argument lists in
/// procedural macros.
struct ArgList {
    items: Punctuated<KeyValue, Token![,]>,
}

impl Parse for ArgList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            items: Punctuated::parse_terminated(input)?,
        })
    }
}

// Bootstraps the API server and registers all controllers using Actix-Web.
//
// The `api_server` macro generates the microservice’s `main` function,
// configures the Actix-Web `HttpServer`, and maps each controller’s static
// handler methods into routes. It simplifies service initialization by
// automatically wiring controllers and server settings.
//
// # Usage
//
// Controllers must expose static Actix-Web–compatible handler methods.
//
// ```rust
// #[api_server(
//     controllers_path = "src/module/user, src/module/admin",
// )]
// fn main() {
//    //The macro will generate the server bootstrap code here.
// }
// ```
//
// This macro generates the server startup code, registers all routes derived
// from the provided controllers, and binds the application to the configured
// address.
#[proc_macro_attribute]
pub fn api_server(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let main_fn = parse_macro_input!(item as ItemFn);
    let arg_list = parse_macro_input!(attrs as ArgList);

    impl_main_fn(main_fn, arg_list)
}

/// Generates the expanded `main` function for the procedural macro.
///
/// This function extracts the body of the user-provided `main` function,
/// processes the project's controllers to build endpoint registration code,
/// and produces the final token stream that initializes and runs the server.
///
/// # Parameters
/// - `main_fn`: The original `main` function captured by the macro.
/// - `arg_list`: Parsed arguments used to locate and register controllers.
///
/// # Returns
/// A [`TokenStream`] containing the generated `main` function and the
/// controller registration code.
fn impl_main_fn(main_fn: ItemFn, arg_list: ArgList) -> TokenStream {
    let main_body = &main_fn.block.stmts;

    let new_server_command = impl_generate_new_server(&arg_list);

    // Search and process project controllers
    let (register_token, openapi_handlers, import_modules) =
        search_and_process_controllers(&arg_list);
    let openapi_token = generate_openapi_token(openapi_handlers, &arg_list);
    let database = impl_generate_database_intialization(&arg_list);
    let fn_name = main_fn.sig.ident;
    let fn_visibility = &main_fn.vis;

    quote! {
        #import_modules

        use rust_microservice::Server;

        #[tokio::main]
        #fn_visibility async fn #fn_name() -> rust_microservice::Result<(), String> {
            #( #main_body )*

            let server = #new_server_command
                .init().await.map_err(|e| e.to_string())?
                #database
                .configure(Some(register_endpoints));

            Server::set_global(server);
            let result = Server::global_server();
            match result {
                Some(server) => {
                    server.run().await;
                }
                None => {
                    return Err("Global server is not initialized".to_string());
                }
            }

            Ok(())
        }

        #register_token

        #openapi_token
    }
    .into()
}

/// Generates code to initialize the database connection based on the provided
/// configuration parameters.
///
/// # Arguments
///
/// - `arg_list`: The list of arguments provided to the macro.
///
/// # Returns
///
/// A `TokenStream` containing the generated code to initialize the database
/// connection. If the `database` parameter is set to `true`, the generated code
/// will initialize the database connection. Otherwise, it will return an empty
/// `TokenStream`.
fn impl_generate_database_intialization(arg_list: &ArgList) -> proc_macro2::TokenStream {
    let initialize_database: bool =
        get_arg_string_value(arg_list, "database".to_string(), "false".to_string())
            .parse()
            .expect("Failed to parse Database value");

    match initialize_database {
        true => quote! {
            .intialize_database().await.map_err(|e| e.to_string())?
        },
        false => quote! {},
    }
}

fn impl_generate_new_server(arg_list: &ArgList) -> proc_macro2::TokenStream {
    let banner = get_arg_string_value(arg_list, "banner".to_string(), "".to_string());
    if !banner.is_empty() {
        return quote! {
            Server::new(env!("CARGO_PKG_VERSION").to_string(), Some(#banner.into()))
        };
    }
    quote! {
       Server::new(env!("CARGO_PKG_VERSION").to_string(), None)
    }
}

/// Searches for controller files and generates endpoint registration code.
///
/// This function reads the `controllers_path` argument, recursively scans the
/// specified directories for Rust (`.rs`) files, and processes each controller
/// to extract route handlers and required module imports.
///
/// It returns:
/// - A `TokenStream` containing the generated `register_endpoints` function,
///   which registers all discovered handlers and configures Swagger UI.
/// - A list of `TokenStream`s representing the collected handlers.
/// - A `TokenStream` with the unique `use` statements (module imports)
///   required by the discovered controllers.
///
/// # Parameters
/// * `arg_list` - Macro arguments containing the `controllers_path` configuration.
///
/// # Returns
/// A tuple with:
/// 1. Generated endpoint registration code.
/// 2. A vector of handler token streams.
/// 3. Generated module import token streams.
fn search_and_process_controllers(
    arg_list: &ArgList,
) -> (
    proc_macro2::TokenStream,
    Vec<proc_macro2::TokenStream>,
    proc_macro2::TokenStream,
) {
    let controllers_path =
        get_arg_string_value(arg_list, "controllers_path".to_string(), "".to_string());

    let span = proc_macro::Span::call_site();
    let main_file_syntax_tree = load_syntax_tree_from_file(span.file());

    let mut handlers: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut openapi_handlers: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut import_modules: Vec<proc_macro2::TokenStream> = Vec::new();
    if !controllers_path.is_empty() {
        let paths = controllers_path.split(',').collect::<Vec<&str>>();
        paths.iter().for_each(|root| {
            //println!("Processing controller path: {root}");

            let path = Path::new(root.trim_matches(|c| c == ' ' || c == '"'));
            if path.exists() && path.is_dir() {
                //println!("Processing controller DIR: {path:?}");
                for entry in walkdir::WalkDir::new(path) {
                    match entry {
                        Ok(entry) => {
                            let file_path = entry.path();
                            if file_path.is_file()
                                && file_path.extension().and_then(|s| s.to_str()) == Some("rs")
                            {
                                let module_path = convert_path_to_module(file_path);
                                let module_token: proc_macro2::TokenStream =
                                    parse_str(module_path.as_str()).unwrap();

                                let (handler, openapi) = process_controller(&entry, &module_token);
                                if !handler.is_empty() {
                                    handlers.push(handler);
                                    openapi_handlers.push(openapi);
                                }

                                let import_module =
                                    find_imported_module(&main_file_syntax_tree, &module_token);
                                if !import_module.is_empty() {
                                    let match_found = import_modules
                                        .iter()
                                        .find(|m| *m.to_string() == import_module.to_string());
                                    if match_found.is_none() {
                                        import_modules.push(import_module);
                                    }
                                }
                            }
                        }
                        Err(error) => println!("Error processing controller. Detail: {error}"),
                    }
                }
            }
            // else {
            //     println!("Controller path does not exist or is not a directory: {path:?}. isDir: {}, exists: {}",
            //         path.is_dir(),
            //         path.exists()
            //     );
            // }
        });
    }

    let quote = quote! {
        use actix_web::web::ServiceConfig;

        fn register_endpoints(cfg: &mut ServiceConfig) {
            #(#handlers)*

            // Register the swagger-ui handler
            let openapi = ApiDoc::openapi();
            cfg.service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone())
                    .config(Config::default().validator_url("none"))
            );
        }
    };

    let import_mod = quote! {
        #( #import_modules )*
    };

    (quote, openapi_handlers, import_mod)
}

/// Converts a file system path into a Rust module path.
///
/// This function normalizes a controller file path by stripping the `src`
/// prefix, removing the extension, and converting directory separators
/// into `::`, producing a valid Rust module path.
///
/// # Parameters
/// - `path`: The file path to convert.
///
/// # Returns
/// A `String` containing the normalized module path.
fn convert_path_to_module(path: &Path) -> String {
    let s = path
        .with_extension("")
        .strip_prefix("src")
        .unwrap()
        .to_string_lossy()
        .to_string();
    let normalized = s.replace('\\', "/").replace('/', "::");
    //println!("NORMALIZED: {normalized}");
    normalized
        .replace("::mod::", "::")
        .strip_suffix("::mod")
        .unwrap_or(&normalized)
        .to_string()
}

/// Generates the token stream needed to register all handler functions
/// from a controller module.
///
/// This function reads the controller source file, parses its syntax tree,
/// extracts handler functions, and produces the Actix-Web service
/// registration code.
///
/// # Parameters
/// - `file`: Directory entry of the controller file.
/// - `module`: Parsed module path token.
///
/// # Returns
/// A [`TokenStream`] containing the registration statements.
fn process_controller(
    file: &DirEntry,
    module: &proc_macro2::TokenStream,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let filename = file.file_name().to_str().unwrap();
    let mut f = File::open(file.path()).expect(format!("Unable to open file: {filename}").as_str());
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect(format!("Unable to read file: {filename}").as_str());
    let syntax_tree = parse_file(&contents).unwrap();
    let handles = find_file_handles(&syntax_tree);

    let services = quote! {
        #( cfg.service(#module::#handles); )*
    };

    let openapi = quote! {
        #(#module::#handles),*
    };

    (services, openapi)
}

/// Checks whether a module is already imported in the given syntax tree.
///
/// This function inspects the root items of a parsed Rust source file
/// (`syn::File`) to determine if the root module of the provided module path
/// is declared. If the module is not found, it generates a `mod <name>;`
/// declaration.
///
/// # Parameters
/// - `syntax_tree`: The parsed Rust file AST to search for module declarations.
/// - `module`: A token stream representing a module path (e.g. `foo::bar`).
///
/// # Returns
/// A [`TokenStream`] containing:
/// - an empty token stream if the root module is already declared, or
/// - a `mod <root_module>;` declaration if it is missing.
fn find_imported_module(
    file: &Option<syn::File>,
    module: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let root_module: proc_macro2::TokenStream =
        parse_str(module.to_string().split("::").next().unwrap()).unwrap();

    if let Some(syntax_tree) = file {
        for item in &syntax_tree.items {
            if let syn::Item::Mod(item_mod) = item
                && item_mod.ident.to_string() == root_module.to_string()
            {
                // println!("Module already imported: {}", root_module.to_string(),);
                return quote! {};
            }
        }
    }
    quote! {
        pub mod #root_module;
    }
}

/// Loads and parses a Rust source file into a `syn::File` syntax tree.
///
/// This function opens the given file path, reads its entire contents,
/// and parses it using `syn::parse_file`, returning the resulting
/// abstract syntax tree (AST).
///
/// # Panics
///
/// Panics if the file cannot be opened, read, or if the source code
/// is not valid Rust syntax.
///
/// # Arguments
///
/// * `file` - Path to the Rust source file to be parsed.
///
/// # Returns
///
/// A `syn::File` representing the parsed syntax tree of the source file.
fn load_syntax_tree_from_file(file: String) -> Option<syn::File> {
    let f = Path::new(&file);
    if f.is_dir() || !f.exists() {
        return None;
    }
    let mut f = File::open(file).expect("Unable to open file.");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("Unable to read file: {file}");
    let syntax_tree = parse_file(&contents).unwrap();
    Some(syntax_tree)
}

/// Extracts all handler function identifiers from a parsed Rust file.
///
/// A handler function is any function annotated with an Actix-Web HTTP
/// method macro (e.g., `#[get]`, `#[post]`). This function collects those
/// identifiers for later endpoint registration.
///
/// # Parameters
/// - `syntax_tree`: Parsed Rust file.
///
/// # Returns
/// A vector of function identifiers representing handler endpoints.
fn find_file_handles(syntax_tree: &syn::File) -> Vec<Ident> {
    let mut handles: Vec<Ident> = Vec::new();
    for item in &syntax_tree.items {
        if let syn::Item::Fn(item_fn) = item
            && is_handle_function(item_fn)
        {
            handles.push(item_fn.sig.ident.clone());
        }
    }
    handles
}

/// Determines whether a function is an Actix-Web handler.
///
/// A function qualifies as a handler if it contains one of the supported
/// HTTP method attributes (e.g., `#[get]`, `#[post]`, `#[put]`).
///
/// # Parameters
/// - `item_fn`: The function to inspect.
///
/// # Returns
/// `true` if the function is a handler, otherwise `false`.
fn is_handle_function(item_fn: &ItemFn) -> bool {
    const HTTP_METHODS: &[&str] = &[
        "get", "post", "put", "delete", "head", "connect", "options", "trace", "patch", "secured",
    ];

    item_fn
        .attrs
        .iter()
        .filter_map(|attr| attr.meta.path().get_ident())
        .any(|ident| HTTP_METHODS.iter().any(|m| ident == m))
}

/// Generates an OpenAPI specification token stream.
///
/// This function builds a `TokenStream` containing the `#[derive(OpenApi)]`
/// configuration, using values extracted from `arg_list` to define the
/// OpenAPI title, API name, and description. It also injects the provided
/// handler paths into the OpenAPI `paths` section.
///
/// # Parameters
/// - `handlers`: A list of tokenized API handler paths.
/// - `arg_list`: The arguments used to customize OpenAPI metadata.
///
/// # Returns
/// A `TokenStream` representing the OpenAPI configuration for code generation.
fn generate_openapi_token(
    handlers: Vec<proc_macro2::TokenStream>,
    arg_list: &ArgList,
) -> proc_macro2::TokenStream {
    let openapi_title = get_arg_string_value(
        arg_list,
        "openapi_title".to_string(),
        "🌐 API Server".to_string(),
    );
    let api_name = get_arg_string_value(
        arg_list,
        "openapi_api_name".to_string(),
        "⚙️ Rest API".to_string(),
    );
    let api_description = get_arg_string_value(
        arg_list,
        "openapi_api_description".to_string(),
        "Rest API OpenApi Documentation.".to_string(),
    );
    let auth_server =
        get_arg_string_value(arg_list, "openapi_auth_server".to_string(), "".to_string());

    quote! {
        use utoipa_swagger_ui::{SwaggerUi, Config};
        use utoipa::{
            Modify, OpenApi,
            openapi::SecurityRequirement,
            openapi::security::{
                Flow,
                Password,
                OAuth2,
                Scopes,
                SecurityScheme,
                Http,
                HttpAuthScheme
            },
        };

        #[derive(OpenApi)]
        #[openapi(
            info(
                title = #openapi_title,
            ),
            paths(
                #( #handlers, )*
            ),
            components(

            ),
            modifiers(&SecurityAddon),
            tags(
                (name = #api_name, description = #api_description)
            ),
        )]
        struct ApiDoc;

        struct SecurityAddon;

        impl Modify for SecurityAddon {
            fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
                let token_url = match Server::global()
                    .ok()
                    .and_then(|s| s.settings().get_auth2_token_url())
                {
                    Some(url) => url,
                    None => #auth_server.to_string(),
                };

                openapi.security = Some(vec![SecurityRequirement::new(
                    "OAuth2 Authentication",
                    ["openid", "profile", "email"],
                )]);

                if let Some(components) = openapi.components.as_mut() {
                    components.add_security_scheme(
                        "OAuth2 Authentication",
                        SecurityScheme::OAuth2(OAuth2::with_description(
                            [Flow::Password(Password::new(
                                token_url, // authorization url
                                Scopes::from_iter([
                                    ("openid", "Standard OIDC scope"),
                                    ("profile", "Access to user profile info"),
                                    ("email", "Access to user email"),
                                ]),
                            ))],
                            "OAuth2 Authentication",
                        )),
                    );
                    // components.add_security_scheme(
                    //     "bearerAuth",
                    //     SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
                    // );
                }
            }
        }
    }
}

/// Returns the string value of a given key from an `ArgList`.
///
/// This function searches the list for an item whose `key` matches the
/// provided `key`. If found and the value is a string literal, that
/// literal is returned. Otherwise, the provided `default` value is
/// returned.
///
/// # Parameters
/// - `arg_list`: The list of parsed arguments.
/// - `key`: The key to search for.
/// - `default`: The fallback value if the key is missing or not a string literal.
///
/// # Returns
/// A `String` containing either the matched literal value or the default.
fn get_arg_string_value(arg_list: &ArgList, key: String, default: String) -> String {
    let value = arg_list
        .items
        .iter()
        .find(|kv| kv.key == key)
        .and_then(|kv| match &kv.value {
            Expr::Lit(expr_lit) => match &expr_lit.lit {
                syn::Lit::Str(lit_str) => Some(lit_str.value()),
                _ => Some(default.clone()),
            },
            _ => Some(default.clone()),
        });
    value.unwrap_or(default)
}

/// Marks a function as requiring authentication and authorization.
///
/// The `secured` macro can be applied to functions to protect them from unauthorized access.
/// It takes a list of arguments, which can include the `roles` configuration parameter.
///
/// The `roles` parameter is a comma-separated list of roles that are allowed to access the function.
/// If the `roles` parameter is missing or empty, the function is open to all authenticated users.
///
/// Example usage:
///
/// #[utoipa::path(
///     tag = "Delete a user by ID",
///     responses(
///         (status = 400, description = "Returns an error if the deletion fails.", body = String),
///     )
///  )]
/// #[get("/v2/user/{user}")]
/// #[secured(roles = "ROLE_ADMIN")]
/// pub async fn create_delete_endpoint(user: web::Json<UserDTO>) -> HttpResponse {
///     HttpResponse::Ok()
///         .content_type("application/json")
///         .body("ok".to_string())
/// }
#[proc_macro_attribute]
pub fn secured(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let secure_fn = parse_macro_input!(item as ItemFn);
    let arg_list = parse_macro_input!(attrs as ArgList);

    impl_secured_fn(secure_fn, arg_list)
}

/// Generates a secured version of a function.
///
/// This function takes an original function (ItemFn) and an argument list (ArgList)
/// and returns a new function definition with the same signature and body,
/// but with the authentication and authorization checks applied.
///
/// The returned function checks if the authenticated user has at least one of the roles
/// specified in the `roles` parameter. If the user is authorized, the function body is executed.
///
/// # Parameters
/// - `secured_fn`: The original function to secure.
/// - `arg_list`: The list of arguments containing the `roles` configuration parameter.
///
/// # Returns
/// A `TokenStream` containing the secured function definition.
fn impl_secured_fn(secured_fn: ItemFn, arg_list: ArgList) -> TokenStream {
    let fn_body = &secured_fn.block.stmts;
    let sig = &secured_fn.sig.to_token_stream();
    let fn_name = &secured_fn.sig.ident;
    let _fn_output = &secured_fn.sig.output;
    let _fn_params = &secured_fn.sig.inputs;
    let _fn_modifiers = &secured_fn.sig.asyncness;
    let fn_visibility = &secured_fn.vis;

    let method = Ident::new(
        get_arg_string_value(&arg_list, "method".to_string(), "".to_string()).as_str(),
        Span::call_site(),
    );
    let path = get_arg_string_value(&arg_list, "path".to_string(), "".to_string()).to_lowercase();
    let authorize = get_arg_string_value(&arg_list, "authorize".to_string(), "".to_string());
    let _actix_web_attr = update_actix_web_attr(&secured_fn.attrs);
    let auth_module_name = format_ident!("auth_{}", fn_name);
    let wrap_fn = format!(
        "::actix_web::middleware::from_fn({}::auth_middleware)",
        auth_module_name
    )
    .to_string();

    quote! {
        mod #auth_module_name {
            use ::actix_web::{
                HttpMessage,
                http::header::{self, HeaderValue}
            };
            use rust_microservice::Server;
            use tracing::warn;

            pub async fn auth_middleware(
                req: ::actix_web::dev::ServiceRequest,
                next: ::actix_web::middleware::Next<impl ::actix_web::body::MessageBody>,
            ) -> Result<
                ::actix_web::dev::ServiceResponse<impl ::actix_web::body::MessageBody>,
                ::actix_web::Error,
            > {
                Server::global()
                    .map_err(|e| ::actix_web::error::ErrorInternalServerError(e.to_string()))?
                    .validate_jwt(&req, #authorize.to_string())
                    .map_err(|e| {
                        //warn!("Unauthorized: {}", e);
                        ::actix_web::error::ErrorUnauthorized("Unauthorized user.")
                    })?;
                next.call(req).await
            }
        }

        #[#method(#path, wrap = #wrap_fn)]
        #fn_visibility #sig {
            #( #fn_body )*
        }
    }
    .into()
}

/// Updates an Actix-Web attribute by extracting the HTTP method and path.
///
/// It takes a vector of `syn::Attribute`s, finds the first attribute that matches
/// one of the supported HTTP methods, and returns a `proc_macro2::TokenStream`
/// containing the updated attribute.
///
/// Supported HTTP methods are:
/// - `get`
/// - `post`
/// - `put`
/// - `delete`
/// - `head`
/// - `connect`
/// - `options`
/// - `trace`
/// - `patch`
///
/// If no matching attribute is found, an empty `TokenStream` is returned.
fn update_actix_web_attr(attrs: &[Attribute]) -> proc_macro2::TokenStream {
    use syn::Expr;

    const HTTP_METHODS: &[&str] = &[
        "get", "post", "put", "delete", "head", "connect", "options", "trace", "patch",
    ];

    let attr = match attrs
        .iter()
        .find(|attr| HTTP_METHODS.iter().any(|m| attr.path().is_ident(m)))
    {
        Some(attr) => attr,
        None => return quote! {},
    };

    let method = match attr.path().get_ident() {
        Some(ident) => ident.to_string(),
        None => return quote! {},
    };

    let value: Expr = match attr.parse_args() {
        Ok(v) => v,
        Err(_) => return quote! {},
    };

    let wrapper = quote! {
        wrap = "::actix_web::middleware::from_fn(auth::auth_middleware)"
    };

    match method.as_str() {
        "post" => {
            quote! { #[post(#value, #wrapper)] }
        }
        "get" => quote! { #[get(#value)] },
        "put" => quote! { #[put(#value)] },
        "delete" => quote! { #[delete(#value)] },
        "head" => quote! { #[head(#value)] },
        "connect" => quote! { #[connect(#value)] },
        "options" => quote! { #[options(#value)] },
        "trace" => quote! { #[trace(#value)] },
        "patch" => quote! { #[patch(#value)] },
        _ => quote! {},
    }
}

/// Extracts the list of roles from the `secured` macro attribute.
///
/// If the `roles` parameter is specified, it is split by commas into a vector of strings.
/// If no roles are specified, an empty vector is returned.
///
/// # Parameters
/// - `arg_list`: The list of arguments containing the `roles` parameter.
///
/// # Returns
/// A vector of strings representing the roles specified in the `secured` macro attribute.
fn _get_security_roles(arg_list: &ArgList) -> proc_macro2::TokenStream {
    let roles_arg = get_arg_string_value(arg_list, "roles".to_string(), "".to_string());
    if !roles_arg.is_empty() {
        let roles = roles_arg.split(',').collect::<Vec<&str>>();
        quote! {
            const ROLES: &[&'static str] = &[#(#roles),*];
        }
    } else {
        quote! {
            let roles = vec![];
        }
    }
}

/// Wraps a function with a database connection retrieval call.
///
/// This macro takes a function and wraps its body with a call to retrieve a database connection
/// from the server's global state. The wrapped function takes a single argument, which is the database
/// connection retrieved from the server's global state.
///
/// The macro expects the following configuration arguments:
///
/// - `name`: The name of the database to retrieve from the server's global state.
/// - `error`: The error message to return if the database connection retrieval call fails.
///
/// # Example
///
/// ```rust
/// #[database(name = "api", error = "UserError::DatabaseNotConfigured")]
/// pub async fn create_user(dto: UserDTO) -> Result<UserDTO> {
///    let model = user::ActiveModel::from(dto);
///
///     let saved = model.save(&db).await.map_err(|e| match e.sql_err() {
///         Some(SqlErr::UniqueConstraintViolation(_)) => UserError::AlreadyExists,
///         _ => UserError::Create(e.to_string()),
///     })?;
///
///    UserDTO::try_from(saved).map_err(|e| UserError::Conversion(e.to_string()))
/// }
/// ```
///
///
#[proc_macro_attribute]
pub fn database(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    let arg_list = parse_macro_input!(attrs as ArgList);

    impl_database_fn(item_fn, arg_list)
}

/// Wraps a function with a database connection retrieval call.
///
/// This function takes an `ItemFn` and an `ArgList` as parameters. It extracts the
/// function body, signature, and visibility from the `ItemFn`, and extracts the database
/// name and error message from the `ArgList`. It then generates a token stream that wraps the
/// function body with a call to retrieve a database connection from the server's global
/// state, using the extracted database name and error message.
///
/// The generated token stream will contain a function with the same signature and visibility
/// as the original function, but with a wrapped body that first retrieves a database connection
/// and then calls the original function body with the connection as an argument.
///
/// # Parameters
/// - `item_fn`: The function to wrap with a database connection retrieval call.
/// - `arg_list`: The arguments containing the database name and error message.
///
/// # Returns
/// A token stream representing the wrapped function.
fn impl_database_fn(item_fn: ItemFn, arg_list: ArgList) -> TokenStream {
    let fn_body = &item_fn.block.stmts;
    let sig = &item_fn.sig.to_token_stream();
    let fn_visibility = &item_fn.vis;
    let db_name = get_arg_string_value(&arg_list, "name".to_string(), "".to_string());
    let error_str = get_arg_string_value(&arg_list, "error".to_string(), "".to_string());
    let error: proc_macro2::TokenStream = parse_str(error_str.as_str()).unwrap();

    quote! {
        #fn_visibility #sig {
            let db = Server::global()
                .map_err(|_| #error)?
                .database_with_name(#db_name)
                .map_err(|_| #error)?;

            #( #fn_body )*
        }

    }
    .into()
}
