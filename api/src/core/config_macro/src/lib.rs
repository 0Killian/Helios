use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, Meta, parse_macro_input};

/// A procedural macro for generating configuration structs that load values from environment variables.
///
/// This macro generates a `from_env(prefix: &str, default: Option<&str>)` method that automatically loads configuration
/// values from environment variables. It constructs the environment variable name
/// as `{prefix}_{ENV_SUFFIX}`.
///
/// # Syntax
/// ```rust
/// #[config]
/// pub struct MyConfig {
///     #[env("ENV_SUFFIX", default = "default_value")]
///     pub field_name: FieldType,
///
///     #[env("ENV_SUFFIX")]
///     pub required_field: FieldType,
///
///     #[env("ENV_PREFIX")]
///     pub nested_config: NestedConfigType,
/// }
/// ```
///
/// # Field Types
/// - **Environment fields with default**: `#[env("ENV_SUFFIX", default = "default")]`
///   - Creates env var: `{prefix}_{ENV_SUFFIX}`
///   - Uses default if env var is not set
/// - **Required environment fields**: `#[env("ENV_SUFFIX")]`
///   - Creates env var: `{prefix}_{ENV_SUFFIX}`
///   - Panics if env var is missing
///
/// # Examples
///
/// **Simple Configuration:**
/// ```rust
/// #[config]
/// pub struct DatabaseConfig {
///     #[env("HOST", default = "localhost")]
///     pub host: String,
///
///     #[env("PORT", default = "5432")]
///     pub port: u16,
///
///     #[env("PASSWORD")]
///     pub password: String,
/// }
///
/// // Usage: DatabaseConfig::from_env("DB")
/// // Environment variables: DB_HOST, DB_PORT, DB_PASSWORD
/// ```
///
/// **Nested Configuration:**
/// ```rust
/// #[config]
/// pub struct AppConfig {
///     #[env("DEBUG", default = "false")]
///     pub debug: bool,
///     #[env("DATABASE")]
///     pub database: DatabaseConfig,
///     #[env("SERVER")]
///     pub server: ServerConfig,
/// }
///
/// // Usage: AppConfig::from_env("APP")
/// // Environment variables:
/// // - APP_DEBUG
/// // - APP_DATABASE_HOST, APP_DATABASE_PORT, APP_DATABASE_PASSWORD (from DatabaseConfig with DATABASE prefix)
/// // - APP_SERVER_HOST, APP_SERVER_PORT (from ServerConfig with SERVER prefix)
/// ```
#[proc_macro_attribute]
pub fn config(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let struct_vis = &input.vis;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let mut field_definitions = Vec::new();
    let mut field_initializations = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let field_vis = &field.vis;

        field_definitions.push(quote! {
            #field_vis #field_name: #field_type
        });

        if let Some(env_attr) = get_env_attribute(field) {
            let (env_suffix, default) = parse_env_attribute(&env_attr);

            let default_value = if let Some(default_value) = default {
                quote! { Some(#default_value) }
            } else {
                quote! { None }
            };

            field_initializations.push(quote! {
                #field_name: {
                    let env_var_name = format!("{}_{}", prefix, #env_suffix);
                    #field_type::from_env(&env_var_name, #default_value)
                }
            });
        }
    }

    let expanded = quote! {
        #struct_vis struct #struct_name {
            #(#field_definitions,)*
        }

        impl #struct_name {
            pub fn from_env(prefix: &str, _: Option<&str>) -> Self {
                Self {
                    #(#field_initializations,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_env_attribute(field: &Field) -> Option<&syn::Attribute> {
    field.attrs.iter().find(|attr| attr.path().is_ident("env"))
}

fn parse_env_attribute(attr: &syn::Attribute) -> (String, Option<String>) {
    let mut env_suffix = None;
    let mut default = None;

    match &attr.meta {
        Meta::List(meta_list) => {
            let tokens_str = meta_list.tokens.to_string();
            let parts: Vec<&str> = tokens_str.split(',').collect();

            // First part should be the env variable suffix (a string literal)
            if let Some(first_part) = parts.first() {
                let trimmed = first_part.trim();
                if trimmed.starts_with('"') && trimmed.ends_with('"') {
                    env_suffix = Some(trimmed[1..trimmed.len() - 1].to_string());
                }
            }

            // Look for default parameter in remaining parts
            for part in parts.iter().skip(1) {
                let trimmed = part.trim();
                if trimmed.starts_with("default") {
                    if let Some(equals_pos) = trimmed.find('=') {
                        let default_part = trimmed[equals_pos + 1..].trim();
                        if default_part.starts_with('"') && default_part.ends_with('"') {
                            default = Some(default_part[1..default_part.len() - 1].to_string());
                        }
                    }
                }
            }
        }
        _ => panic!("env attribute must be a list"),
    }

    (
        env_suffix
            .expect("env attribute must have a string literal for the environment variable suffix"),
        default,
    )
}
