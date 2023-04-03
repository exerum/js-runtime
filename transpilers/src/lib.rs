pub mod rquickjs;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use core::cell::RefMut;
use crate::rquickjs::{Ctx, Module, Loaded, Result, Script};
/// Reusable trinspiler
pub trait AssetTranspiler {
    fn transpile<'js>(&mut self, ctx: Ctx<'js>, path: &str) -> Result<Module<'js, Loaded<Script>>>;
}

#[derive(Hash, Eq, PartialEq)]
pub enum TKey {
    Name(String),
    Extension(String)
}

#[macro_export]
macro_rules! register {
    ($obj:ident, $name:literal, [$(.$ext:tt),*], $transpiler:ident) => {
        let __t: std::rc::Rc<std::cell::RefCell<dyn $crate::AssetTranspiler>> = std::rc::Rc::new(std::cell::RefCell::new($transpiler::default()));
        for __ext in vec![$(stringify!($ext),)*] {
            $obj.register_transpiler($crate::TKey::Extension(__ext.to_owned()), std::rc::Rc::clone(&__t));
        };
        $obj.register_transpiler($crate::TKey::Name($name.to_owned()), std::rc::Rc::clone(&__t));
    }
}

#[derive(Default)]
pub struct Transpilers {
    inner: HashMap<TKey, Rc<RefCell<dyn AssetTranspiler>>>
}

impl Transpilers {
    pub fn register_transpiler(&mut self, key: TKey, transpiler: Rc<RefCell<dyn AssetTranspiler>>) {
        self.inner.insert(key, transpiler);
    }

    #[inline]
    pub fn by_name(&mut self, name: &str) -> Option<RefMut<'_, (dyn AssetTranspiler + 'static)>>  {
        self.inner
            .get_mut(&TKey::Name(name.to_owned()))
            .map(|t| t.borrow_mut())
    }

    #[inline]
    pub fn by_ext(&mut self, ext: &str) -> Option<RefMut<'_, (dyn AssetTranspiler + 'static)>>  {
        self.inner
            .get_mut(&TKey::Extension(ext.to_owned()))
            .map(|t| t.borrow_mut())
    }
}