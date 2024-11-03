use crate::utils::{inner_vec_type, should_flatten};
use syn::{Field as SynField, Ident, Type, TypePath, Visibility};

pub struct Field<'a> {
    pub field: &'a SynField,
    pub should_flatten: bool,
}

impl Field<'_> {
    pub fn name(&self) -> &Ident {
        self.field.ident.as_ref().unwrap()
    }

    pub fn ty(&self) -> &Type {
        &self.field.ty
    }

    pub fn visibility(&self) -> &Visibility {
        &self.field.vis
    }

    pub fn type_path(&self) -> Option<&TypePath> {
        if let Type::Path(type_path) = self.ty() {
            Some(type_path)
        } else {
            None
        }
    }

    pub fn inner_vec_type(&self) -> Option<&Ident> {
        self.type_path().and_then(inner_vec_type)
    }
}

impl<'a> From<&'a SynField> for Field<'a> {
    fn from(field: &'a SynField) -> Self {
        Self {
            field,
            should_flatten: should_flatten(&field.attrs),
        }
    }
}
