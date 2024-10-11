//! Characteristic attribute parsing and handling.
//!
//! This module contains the parsing and handling of the characteristic attribute.
//! The characteristic attribute is used to define a characteristic in a service.
//! A characteristic is a data value that can be read, written, or notified.

use crate::uuid::Uuid;
use darling::Error;
use darling::FromMeta;
use proc_macro2::Span;
use syn::parse::Result;
use syn::spanned::Spanned as _;
use syn::Field;
use syn::LitStr;

#[derive(Debug)]
pub(crate) struct Characteristic {
    pub name: String,
    pub ty: syn::Type,
    pub args: CharacteristicArgs,
    pub span: Span,
    pub vis: syn::Visibility,
}

impl Characteristic {
    pub fn new(field: &Field, args: CharacteristicArgs) -> Self {
        Self {
            name: field.ident.as_ref().expect("Field had no Identity").to_string(),
            ty: field.ty.clone(),
            args,
            span: field.ty.span(),
            vis: field.vis.clone(),
        }
    }
}

/// Descriptor attribute arguments.
///
/// Descriptors are optional and can be used to add additional metadata to the characteristic.
#[derive(Debug, FromMeta)]
pub(crate) struct DescriptorArgs {
    /// The UUID of the descriptor.
    _uuid: Uuid,
    /// The value of the descriptor.
    #[darling(default)]
    _value: Option<syn::Expr>,
}

/// Characteristic attribute arguments
#[derive(Debug, FromMeta, Default)]
pub(crate) struct CharacteristicArgs {
    /// The UUID of the characteristic.
    pub uuid: Option<Uuid>,
    /// If true, the characteristic can be read.
    #[darling(default)]
    pub read: bool,
    /// If true, the characteristic can be written.
    #[darling(default)]
    pub write: bool,
    /// If true, the characteristic can be written without a response.
    #[darling(default)]
    pub write_without_response: bool,
    /// If true, the characteristic can send notifications.
    #[darling(default)]
    pub notify: bool,
    /// If true, the characteristic can send indications.
    #[darling(default)]
    pub indicate: bool,
    /// The initial value of the characteristic.
    /// This is optional and can be used to set the initial value of the characteristic.
    #[darling(default)]
    pub value: Option<syn::Expr>,
    /// Callback to be called when a write request is received
    #[darling(default)]
    pub on_write: Option<syn::Ident>,
    /// Callback to be called when a read request is received
    #[darling(default)]
    pub on_read: Option<syn::Ident>,
    /// Indicates that a characteristic is managed by the application. This includes allocation of memory and interaction with that memory.
    #[darling(default)]
    pub app_managed: bool,
    /// Descriptors for the characteristic.
    /// Descriptors are optional and can be used to add additional metadata to the characteristic.
    #[darling(default, multiple)]
    pub _descriptor: Vec<DescriptorArgs>,
}

impl CharacteristicArgs {
    /// Parse the arguments of a characteristic attribute
    pub fn parse(attribute: &syn::Attribute) -> Result<Self> {
        let mut args = CharacteristicArgs::default();
        attribute.parse_nested_meta(|meta| {
            match meta.path.get_ident().ok_or(Error::custom("no ident"))?.to_string().as_str() {
                "uuid" => {
                    let value = meta
                    .value()
                    .map_err(|_| Error::custom("uuid must be followed by '= [data]'.  i.e. uuid = '0x2A37'".to_string()))?;
                    let uuid_string: LitStr = value.parse()?;
                    args.uuid = Some(Uuid::from_string(uuid_string.value().as_str())?);
                },
                "read" => args.read = true,
                "write" => args.write = true,
                "write_without_response" => args.write_without_response = true,
                "notify" => args.notify = true,
                "indicate" => args.indicate = true,
                "value" => {
                    let value = meta
                    .value()
                    .map_err(|_| Error::custom("value must be followed by '= [data]'.  i.e. value = 'hello'".to_string()))?;
                    args.value = Some(value.parse()?);
                },
                "on_write" => {
                    let value = meta.value().map_err(|_| Error::custom("on_write must be followed by '= [callback]'. i.e. on_write = characterisic_on_write".to_string()))?;
                    args.on_write = Some(value.parse()?);
                }
                "on_read" => {
                    let value = meta.value().map_err(|_| Error::custom("on_read must be followed by '= [callback]'. i.e. on_read = characteristic_on_read".to_string()))?;
                    args.on_read = Some(value.parse()?);
                }
                "app_managed" => args.app_managed = true,
                other => return Err(
                    meta.error(
                        format!(
                            "Unsupported characteristic property: '{other}'.\nSupported properties are: uuid, read, write, write_without_response, notify, indicate, value, on_read, on_write, app_managed"
                        ))),
            };
            Ok(())
        })?;
        if args.uuid.is_none() {
            return Err(Error::custom("Characteristic must have a UUID").into());
        }
        Ok(args)
    }
}
