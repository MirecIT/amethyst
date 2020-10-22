use crate::{experimental::AssetUuid, Format};
// FIXME
// pub use atelier_importer::SourceFileImporter;
use atelier_importer::{self as importer, ImportedAsset, Importer, ImporterValue, SerdeObj};
use serde::{Deserialize, Serialize};
use std::io::Read;
use type_uuid::TypeUuid;

/// A simple state for Importer to retain the same UUID between imports
/// for all single-asset source files
#[derive(Default, Serialize, Deserialize, TypeUuid)]
#[uuid = "6b00ea4b-f98c-4b43-94e1-e696c96a6b93"]
pub struct SimpleImporterState {
    id: Option<AssetUuid>,
}

/// Wrapper struct to be able to impl Importer for any SimpleFormat
pub struct SimpleImporter<A: 'static, T: Format<A> + TypeUuid>(
    pub T,
    ::std::marker::PhantomData<A>,
);

impl<A: 'static, T: Format<A> + TypeUuid + 'static> From<T> for SimpleImporter<A, T> {
    fn from(fmt: T) -> SimpleImporter<A, T> {
        SimpleImporter(fmt, ::std::marker::PhantomData)
    }
}
impl<A, T: Format<A> + TypeUuid + Send + 'static> TypeUuid for SimpleImporter<A, T>
where
    A: SerdeObj,
{
    const UUID: type_uuid::Bytes = T::UUID;
}

impl<A, T: Format<A> + TypeUuid + Send + 'static> Importer for SimpleImporter<A, T>
where
    A: SerdeObj,
{
    type State = SimpleImporterState;
    type Options = T;

    fn version_static() -> u32
    where
        Self: Sized,
    {
        1
    }
    fn version(&self) -> u32 {
        Self::version_static()
    }

    fn import(
        &self,
        source: &mut dyn Read,
        options: &Self::Options,
        state: &mut Self::State,
    ) -> importer::Result<ImporterValue> {
        if state.id.is_none() {
            state.id = Some(AssetUuid(*uuid::Uuid::new_v4().as_bytes()));
        }
        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes)?;
        let import_result = options
            .import_simple(bytes)
            .map_err(|e| importer::Error::Boxed(e.into_error()))?;
        Ok(ImporterValue {
            assets: vec![ImportedAsset {
                id: state.id.expect("AssetUUID not generated"),
                search_tags: Vec::new(),
                build_deps: Vec::new(),
                load_deps: Vec::new(),
                asset_data: Box::new(import_result),
                build_pipeline: None,
            }],
        })
    }
}

// Associates the given file extension with a `Format` implementation
//
// The `AssetDaemon` will automatically re-import the asset when a file of that format is created
// or modified.
//
// # Parameters
//
// * `ext`: File extension including the leading `.`, such as `".ron"`.
// * `format`: Type that implements the `Format` trait.
//
// # Examples
//
// ```rust,ignore
// amethyst_assets::register_importer!(".ron", Ron);
// ```
#[macro_export]
macro_rules! register_importer {
    ($ext:literal, $format:ty) => {
        $crate::register_importer!(amethyst_assets; $ext, $format);
    };
    ($krate:ident; $ext:literal, $format:ty) => {
        $crate::inventory::submit!{
            #![crate = $krate]
            $crate::experimental::SourceFileImporter {
                extension: $ext,
                instantiator: || Box::new($crate::experimental::SimpleImporter::from(<$format as Default>::default())),
            }
        }
    };
}
