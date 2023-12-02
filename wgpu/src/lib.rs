pub use wgpu_types::Backends;
pub use wgpu_types::TextureDimension;
pub use wgpu_types::TextureFormat;
pub use wgpu_types::TextureUsages;

pub type Label<'a> = Option<&'a str>;
pub type TextureDescriptor<'a> =
    wgpu_types::TextureDescriptor<Label<'a>, &'a [wgpu_types::TextureFormat]>;

pub fn device_create_texture(
    global: &crate::core::global::Global<crate::core::identity::IdentityManagerFactory>,
    device: &crate::core::id::DeviceId,
    desc: &TextureDescriptor,
) {
    let wgt_desc =
        desc.map_label_and_view_formats(|l| l.map(std::borrow::Cow::Borrowed), |v| v.to_vec());
    global.device_create_texture::<crate::core::api::Metal>(*device, &wgt_desc, ());
    global.device_create_texture::<crate::core::api::Gles>(*device, &wgt_desc, ());
}

pub mod core {
    pub(crate) mod device {
        use crate::core::Label;
        use thiserror::Error;
        pub(crate) mod global {
            use crate::core::device::DeviceError;
            use crate::core::global::Global;
            use crate::core::hal_api::HalApi;
            use crate::core::hub::Token;
            use crate::core::id::DeviceId;
            use crate::core::id::{self};
            use crate::core::identity::GlobalIdentityHandlerFactory;
            use crate::core::identity::Input;
            use crate::core::resource::{self};
            use crate::core::LabelHelpers as _;
            impl<G: GlobalIdentityHandlerFactory> Global<G> {
                pub fn device_create_texture<A: HalApi>(
                    &self,
                    device_id: DeviceId,
                    desc: &resource::TextureDescriptor,
                    id_in: Input<G, id::TextureId>,
                ) -> (id::TextureId, Option<resource::CreateTextureError>) {
                    let hub = A::hub(self);
                    let mut token = Token::root();
                    let fid = hub.textures.prepare(id_in);
                    let (_, mut token) = hub.adapters.read(&mut token);
                    let (device_guard, mut token) = hub.devices.read(&mut token);
                    let error = loop {
                        let device = match device_guard.get(device_id) {
                            Ok(device) => device,
                            Err(_) => break DeviceError::Invalid.into(),
                        };
                        let texture = match device.create_texture(desc) {
                            Ok(texture) => texture,
                            Err(error) => break error,
                        };
                        let id = fid.assign(texture, &mut token);
                        return (id.0, None);
                    };
                    let id = fid.assign_error(desc.label.borrow_or_default(), &mut token);
                    (id, Some(error))
                }
            }
        }
        pub(crate) mod resource {
            use super::DeviceDescriptor;
            use crate::core::device::MissingFeatures;
            use crate::core::hal_api::HalApi;
            use crate::core::resource::{self};
            use thiserror::Error;
            use wgpu_types::TextureFormat;
            pub struct Device<A: HalApi> {
                pub(crate) features: wgpu_types::Features,
                _marker: std::marker::PhantomData<A>,
            }
            #[derive(Clone, Debug, Error)]
            #[non_exhaustive]
            pub(crate) enum CreateDeviceError {}
            impl<A: HalApi> Device<A> {
                pub(crate) fn require_features(
                    &self,
                    feature: wgpu_types::Features,
                ) -> Result<(), MissingFeatures> {
                    if self.features.contains(feature) {
                        Ok(())
                    } else {
                        unimplemented!()
                    }
                }
            }
            impl<A: HalApi> Device<A> {
                pub(crate) fn new(desc: &DeviceDescriptor) -> Result<Self, CreateDeviceError> {
                    Ok(Self {
                        features: desc.features,
                        _marker: std::marker::PhantomData,
                    })
                }
                pub(super) fn create_texture(
                    &self,
                    desc: &resource::TextureDescriptor,
                ) -> Result<resource::Texture<A>, resource::CreateTextureError> {
                    let _format_features = self.describe_format_features(desc.format);
                    todo!()
                }
                pub(super) fn describe_format_features(
                    &self,
                    format: TextureFormat,
                ) -> Result<wgpu_types::TextureFormatFeatures, MissingFeatures> {
                    self.require_features(format.required_features())?;
                    Ok(format.guaranteed_format_features(self.features))
                }
            }
        }
        pub(crate) use resource::Device;
        pub(crate) type DeviceDescriptor<'a> = wgpu_types::DeviceDescriptor<Label<'a>>;
        #[derive(Clone, Debug, Error)]
        pub enum DeviceError {
            #[error("Parent device is invalid")]
            Invalid,
        }
        #[derive(Clone, Debug, Error)]
        #[error("Features {0:?} are required but not enabled on the device")]
        pub struct MissingFeatures(pub wgpu_types::Features);
    }
    pub mod global {
        use crate::core::hub::Hubs;
        use crate::core::id;
        use crate::core::identity::GlobalIdentityHandlerFactory;
        use crate::core::instance::Surface;
        use crate::core::registry::Registry;
        pub struct Global<G: GlobalIdentityHandlerFactory> {
            pub(crate) surfaces: Registry<Surface, id::SurfaceId, G>,
            pub(crate) hubs: Hubs<G>,
        }
        impl<G: GlobalIdentityHandlerFactory> Global<G> {
            pub fn new(factory: G, _instance_desc: wgpu_types::InstanceDescriptor) -> Self {
                Self {
                    surfaces: Registry::without_backend(&factory),
                    hubs: Hubs::new(&factory),
                }
            }
        }
    }
    pub(crate) mod hal_api {
        use crate::core::global::Global;
        use crate::core::hub::Hub;
        use crate::core::identity::GlobalIdentityHandlerFactory;
        use wgpu_types::Backend;
        pub trait HalApi: crate::hal::Api {
            const VARIANT: Backend;
            fn hub<'a, G: GlobalIdentityHandlerFactory>(_global: &Global<G>) -> &Hub<Self, G> {
                todo!("weird")
            }
        }
        impl HalApi for crate::hal::api::Empty {
            const VARIANT: Backend = Backend::Empty;
        }

        impl HalApi for crate::hal::api::Metal {
            const VARIANT: Backend = Backend::Metal;
            fn hub<G: GlobalIdentityHandlerFactory>(global: &Global<G>) -> &Hub<Self, G> {
                &global.hubs.metal
            }
        }

        impl HalApi for crate::hal::api::Gles {
            const VARIANT: Backend = Backend::Gl;
        }
    }
    pub(crate) mod hub {
        use crate::core::device::Device;
        use crate::core::hal_api::HalApi;
        use crate::core::id;
        use crate::core::identity::GlobalIdentityHandlerFactory;
        use crate::core::instance::Adapter;
        use crate::core::instance::Surface;
        use crate::core::registry::Registry;
        use crate::core::resource::Texture;
        use std::marker::PhantomData;
        pub(crate) trait Access<A> {}
        pub(crate) enum Root {}
        impl Access<Surface> for Root {}
        impl<A: HalApi> Access<Adapter<A>> for Root {}
        impl<A: HalApi> Access<Device<A>> for Root {}
        impl<A: HalApi> Access<Device<A>> for Adapter<A> {}
        impl<A: HalApi> Access<Texture<A>> for Device<A> {}
        pub(crate) struct Token<'a, T: 'a> {
            level: PhantomData<&'a *const T>,
        }
        impl<'a, T> Token<'a, T> {
            pub(crate) fn new() -> Self {
                Self { level: PhantomData }
            }
        }
        impl Token<'static, Root> {
            pub(crate) fn root() -> Self {
                Self { level: PhantomData }
            }
        }
        pub struct Hub<A: HalApi, F: GlobalIdentityHandlerFactory> {
            pub(crate) adapters: Registry<Adapter<A>, id::AdapterId, F>,
            pub(crate) devices: Registry<Device<A>, id::DeviceId, F>,
            pub(crate) textures: Registry<Texture<A>, id::TextureId, F>,
        }
        impl<A: HalApi, F: GlobalIdentityHandlerFactory> Hub<A, F> {
            fn new(factory: &F) -> Self {
                Self {
                    adapters: Registry::new(A::VARIANT, factory),
                    devices: Registry::new(A::VARIANT, factory),
                    textures: Registry::new(A::VARIANT, factory),
                }
            }
        }
        pub(crate) struct Hubs<F: GlobalIdentityHandlerFactory> {
            pub(crate) metal: Hub<crate::hal::api::Metal, F>,
        }
        impl<F: GlobalIdentityHandlerFactory> Hubs<F> {
            pub(crate) fn new(factory: &F) -> Self {
                Self {
                    metal: Hub::new(factory),
                }
            }
        }
    }
    pub mod id {
        use crate::core::Epoch;
        use crate::core::Index;
        use std::fmt;
        use std::marker::PhantomData;
        use wgpu_types::Backend;

        type IdType = u64;

        type NonZeroId = std::num::NonZeroU64;

        type ZippedIndex = Index;
        const INDEX_BITS: usize = std::mem::size_of::<ZippedIndex>() * 8;
        const EPOCH_BITS: usize = INDEX_BITS - BACKEND_BITS;
        const BACKEND_BITS: usize = 3;
        const BACKEND_SHIFT: usize = INDEX_BITS * 2 - BACKEND_BITS;
        pub const EPOCH_MASK: u32 = (1 << (EPOCH_BITS)) - 1;
        type Dummy = crate::hal::api::Empty;
        #[repr(transparent)]
        pub struct Id<T>(NonZeroId, PhantomData<T>);
        impl<T> Id<T> {
            pub fn backend(self) -> Backend {
                match self.0.get() >> (BACKEND_SHIFT) as u8 {
                    0 => Backend::Empty,
                    2 => Backend::Metal,
                    5 => Backend::Gl,
                    _ => unreachable!(),
                }
            }
        }
        impl<T> Copy for Id<T> {}
        impl<T> Clone for Id<T> {
            fn clone(&self) -> Self {
                unimplemented!()
            }
        }
        impl<T> fmt::Debug for Id<T> {
            fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
                unimplemented!()
            }
        }
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
        pub(crate) struct Valid<I>(pub I);
        pub(crate) trait TypedId: Copy {
            fn zip(index: Index, epoch: Epoch, backend: Backend) -> Self;
            fn unzip(self) -> (Index, Epoch, Backend);
            fn into_raw(self) -> NonZeroId;
        }
        #[allow(trivial_numeric_casts)]
        impl<T> TypedId for Id<T> {
            fn zip(index: Index, epoch: Epoch, backend: Backend) -> Self {
                assert_eq!(0, epoch >> EPOCH_BITS);
                assert_eq!(0, (index as IdType) >> INDEX_BITS);
                let v = index as IdType
                    | ((epoch as IdType) << INDEX_BITS)
                    | ((backend as IdType) << BACKEND_SHIFT);
                Id(NonZeroId::new(v).unwrap(), PhantomData)
            }
            fn unzip(self) -> (Index, Epoch, Backend) {
                (
                    (self.0.get() as ZippedIndex) as Index,
                    (((self.0.get() >> INDEX_BITS) as ZippedIndex) & (EPOCH_MASK as ZippedIndex))
                        as Index,
                    self.backend(),
                )
            }
            fn into_raw(self) -> NonZeroId {
                unimplemented!()
            }
        }
        pub type AdapterId = Id<crate::core::instance::Adapter<Dummy>>;
        pub type SurfaceId = Id<crate::core::instance::Surface>;
        pub type DeviceId = Id<crate::core::device::Device<Dummy>>;
        pub type TextureId = Id<crate::core::resource::Texture<Dummy>>;
    }
    pub mod identity {
        use crate::core::id;
        use crate::core::Epoch;
        use crate::core::Index;
        use parking_lot::Mutex;
        use std::fmt::Debug;
        use wgpu_types::Backend;
        #[derive(Debug, Default)]
        pub struct IdentityManager {
            free: Vec<Index>,
            epochs: Vec<Epoch>,
        }
        impl IdentityManager {
            pub(crate) fn alloc<I: id::TypedId>(&mut self, backend: Backend) -> I {
                match self.free.pop() {
                    Some(index) => I::zip(index, self.epochs[index as usize], backend),
                    None => {
                        let epoch = 1;
                        let id = I::zip(self.epochs.len() as Index, epoch, backend);
                        self.epochs.push(epoch);
                        id
                    }
                }
            }
        }
        pub trait IdentityHandler<I>: Debug {
            type Input: Clone + Debug;
            fn process(&self, _id: Self::Input, _backend: Backend) -> I {
                unimplemented!()
            }
            fn free(&self, _id: I) {
                unimplemented!()
            }
        }
        impl<I: id::TypedId + Debug> IdentityHandler<I> for Mutex<IdentityManager> {
            type Input = ();
            fn process(&self, _id: Self::Input, backend: Backend) -> I {
                self.lock().alloc(backend)
            }
        }
        pub trait IdentityHandlerFactory<I> {
            type Filter: IdentityHandler<I>;
            fn spawn(&self) -> Self::Filter;
        }
        #[derive(Debug)]
        pub struct IdentityManagerFactory;
        impl<I: id::TypedId + Debug> IdentityHandlerFactory<I> for IdentityManagerFactory {
            type Filter = Mutex<IdentityManager>;
            fn spawn(&self) -> Self::Filter {
                Mutex::new(IdentityManager::default())
            }
        }
        pub trait GlobalIdentityHandlerFactory:
            IdentityHandlerFactory<id::AdapterId>
            + IdentityHandlerFactory<id::DeviceId>
            + IdentityHandlerFactory<id::TextureId>
            + IdentityHandlerFactory<id::SurfaceId>
        {
        }
        impl GlobalIdentityHandlerFactory for IdentityManagerFactory {}
        pub type Input<G, I> =
            <<G as IdentityHandlerFactory<I>>::Filter as IdentityHandler<I>>::Input;
    }
    pub mod instance {
        use crate::core::device::Device;
        use crate::core::device::DeviceDescriptor;
        use crate::core::global::Global;
        use crate::core::hal_api::HalApi;
        use crate::core::hub::Token;
        use crate::core::id::AdapterId;
        use crate::core::id::DeviceId;
        use crate::core::identity::GlobalIdentityHandlerFactory;
        use crate::core::identity::Input;
        use crate::core::LabelHelpers;
        use thiserror::Error;
        use wgpu_types::Backend;
        use wgpu_types::Backends;
        pub struct Surface {}
        pub struct Adapter<A: crate::hal::Api> {
            _marker: std::marker::PhantomData<A>,
        }
        impl<A: HalApi> Adapter<A> {
            fn new() -> Self {
                Self {
                    _marker: std::marker::PhantomData,
                }
            }
            fn create_device_from_hal(
                &self,
                desc: &DeviceDescriptor,
            ) -> Result<Device<A>, RequestDeviceError> {
                Device::new(desc).or(Err(RequestDeviceError::OutOfMemory))
            }
            pub(crate) fn create_device(
                &self,
                desc: &DeviceDescriptor,
            ) -> Result<Device<A>, RequestDeviceError> {
                self.create_device_from_hal(desc)
            }
        }
        #[derive(Clone, Debug, Error)]
        #[non_exhaustive]
        pub enum RequestDeviceError {
            #[error("Parent adapter is invalid")]
            InvalidAdapter,
            #[error("Not enough memory left")]
            OutOfMemory,
        }
        pub enum AdapterInputs<'a, I> {
            IdSet(&'a [I], fn(&I) -> Backend),
            Mask(Backends, fn(Backend) -> I),
        }
        impl<I: Clone> AdapterInputs<'_, I> {
            fn find(&self, b: Backend) -> Option<I> {
                match *self {
                    Self::IdSet(ids, ref fun) => ids.iter().find(|id| fun(id) == b).cloned(),
                    Self::Mask(bits, ref fun) => {
                        if bits.contains(b.into()) {
                            Some(fun(b))
                        } else {
                            unimplemented!()
                        }
                    }
                }
            }
        }
        #[derive(Clone, Debug, Error)]
        #[non_exhaustive]
        pub enum RequestAdapterError {
            #[error("No suitable adapter found")]
            NotFound,
        }
        impl<G: GlobalIdentityHandlerFactory> Global<G> {
            fn select<A: HalApi>(&self, new_id: Option<Input<G, AdapterId>>) -> Option<AdapterId> {
                let mut token = Token::root();
                let adapter = Adapter::<A>::new();
                let id = HalApi::hub(self)
                    .adapters
                    .prepare(new_id.unwrap())
                    .assign(adapter, &mut token);
                Some(id.0)
            }
            pub fn request_adapter(
                &self,
                inputs: AdapterInputs<Input<G, AdapterId>>,
            ) -> Result<AdapterId, RequestAdapterError> {
                let mut token = Token::root();
                let (_, _) = self.surfaces.read(&mut token);
                let id_metal = inputs.find(crate::hal::api::Metal::VARIANT);
                if let Some(id) = self.select::<crate::hal::api::Metal>(id_metal) {
                    return Ok(id);
                }
                Err(RequestAdapterError::NotFound)
            }
        }
        impl<G: GlobalIdentityHandlerFactory> Global<G> {
            pub fn adapter_request_device<A: HalApi>(
                &self,
                adapter_id: AdapterId,
                desc: &DeviceDescriptor,
                id_in: Input<G, DeviceId>,
            ) -> (DeviceId, Option<RequestDeviceError>) {
                let hub = A::hub(self);
                let mut token = Token::root();
                let fid = hub.devices.prepare(id_in);
                let error = loop {
                    let (adapter_guard, mut token) = hub.adapters.read(&mut token);
                    let adapter = match adapter_guard.get(adapter_id) {
                        Ok(adapter) => adapter,
                        Err(_) => break RequestDeviceError::InvalidAdapter,
                    };
                    let device = match adapter.create_device(desc) {
                        Ok(device) => device,
                        Err(e) => break e,
                    };
                    let id = fid.assign(device, &mut token);
                    return (id.0, None);
                };
                let id = fid.assign_error(desc.label.borrow_or_default(), &mut token);
                (id, Some(error))
            }
        }
    }
    pub(crate) mod registry {
        use crate::core::hub::Access;
        use crate::core::hub::Token;
        use crate::core::id;
        use crate::core::identity::IdentityHandler;
        use crate::core::identity::IdentityHandlerFactory;
        use crate::core::storage::Storage;
        use parking_lot::RwLock;
        use parking_lot::RwLockReadGuard;
        use std::marker::PhantomData;
        use wgpu_types::Backend;
        #[derive(Debug)]
        pub(crate) struct Registry<T, I: id::TypedId, F: IdentityHandlerFactory<I>> {
            identity: F::Filter,
            pub(crate) data: RwLock<Storage<T, I>>,
            backend: Backend,
        }
        impl<T, I: id::TypedId, F: IdentityHandlerFactory<I>> Registry<T, I, F> {
            pub(crate) fn new(backend: Backend, factory: &F) -> Self {
                Self {
                    identity: factory.spawn(),
                    data: RwLock::new(Storage {
                        map: Vec::new(),
                        _phantom: PhantomData,
                    }),
                    backend,
                }
            }
            pub(crate) fn without_backend(factory: &F) -> Self {
                Self {
                    identity: factory.spawn(),
                    data: RwLock::new(Storage {
                        map: Vec::new(),
                        _phantom: PhantomData,
                    }),
                    backend: Backend::Empty,
                }
            }
        }
        #[must_use]
        pub(crate) struct FutureId<'a, I: id::TypedId, T> {
            id: I,
            data: &'a RwLock<Storage<T, I>>,
        }
        impl<I: id::TypedId + Copy, T> FutureId<'_, I, T> {
            pub(crate) fn assign<'a, A: Access<T>>(
                self,
                value: T,
                _: &'a mut Token<A>,
            ) -> id::Valid<I> {
                self.data.write().insert(self.id, value);
                id::Valid(self.id)
            }
            pub(crate) fn assign_error<'a, A: Access<T>>(
                self,
                _label: &str,
                _: &'a mut Token<A>,
            ) -> I {
                unimplemented!()
            }
        }
        impl<T, I: id::TypedId + Copy, F: IdentityHandlerFactory<I>> Registry<T, I, F> {
            pub(crate) fn prepare(
                &self,
                id_in: <F::Filter as IdentityHandler<I>>::Input,
            ) -> FutureId<I, T> {
                FutureId {
                    id: self.identity.process(id_in, self.backend),
                    data: &self.data,
                }
            }
            pub(crate) fn read<'a, A: Access<T>>(
                &'a self,
                _token: &'a mut Token<A>,
            ) -> (RwLockReadGuard<'a, Storage<T, I>>, Token<'a, T>) {
                (self.data.read(), Token::new())
            }
        }
    }
    pub mod resource {
        use crate::core::device::DeviceError;
        use crate::core::Label;
        use thiserror::Error;
        pub type TextureDescriptor<'a> =
            wgpu_types::TextureDescriptor<Label<'a>, Vec<wgpu_types::TextureFormat>>;
        #[derive(Debug)]
        pub struct Texture<A: crate::hal::Api> {
            _marker: std::marker::PhantomData<A>,
        }
        #[derive(Clone, Debug, Error)]
        #[non_exhaustive]
        pub enum CreateTextureError {
            #[error(transparent)]
            Device(#[from] DeviceError),
        }
    }
    pub(crate) mod storage {
        use crate::core::id;
        use crate::core::Epoch;
        use std::marker::PhantomData;
        #[derive(Debug)]
        pub(crate) enum Element<T> {
            Vacant,
            Occupied(T, Epoch),
        }
        #[derive(Clone, Debug)]
        pub(crate) struct InvalidId;
        #[derive(Debug)]
        pub(crate) struct Storage<T, I: id::TypedId> {
            pub(crate) map: Vec<Element<T>>,
            pub(crate) _phantom: PhantomData<I>,
        }
        impl<T, I: id::TypedId> Storage<T, I> {
            pub(crate) fn get(&self, id: I) -> Result<&T, InvalidId> {
                let (index, _, _) = id.unzip();
                let (result, _) = match self.map.get(index as usize) {
                    Some(&Element::Occupied(ref v, epoch)) => (Ok(v), epoch),
                    Some(&Element::Vacant) => panic!("[{}] does not exist", index),
                    None => return Err(InvalidId),
                };
                result
            }
            fn insert_impl(&mut self, index: usize, element: Element<T>) {
                if index >= self.map.len() {
                    self.map.resize_with(index + 1, || Element::Vacant);
                }
                match std::mem::replace(&mut self.map[index], element) {
                    Element::Vacant => {}
                    _ => panic!("Index {index:?} is already occupied"),
                }
            }
            pub(crate) fn insert(&mut self, id: I, value: T) {
                let (index, epoch, _) = id.unzip();
                self.insert_impl(index as usize, Element::Occupied(value, epoch))
            }
        }
    }
    pub use crate::hal::api;
    use std::borrow::Cow;
    type Index = u32;
    type Epoch = u32;
    pub type Label<'a> = Option<Cow<'a, str>>;
    trait LabelHelpers<'a> {
        fn borrow_option(&'a self) -> Option<&'a str>;
        fn borrow_or_default(&'a self) -> &'a str;
    }
    impl<'a> LabelHelpers<'a> for Label<'a> {
        fn borrow_option(&'a self) -> Option<&'a str> {
            unimplemented!()
        }
        fn borrow_or_default(&'a self) -> &'a str {
            unimplemented!()
        }
    }
    macro_rules ! define_backend_caller { { $ public : ident , $ private : ident , $ feature : literal if $ cfg : meta } => {  # [macro_export] macro_rules ! $ private { ($ call : expr) => ($ call) } # [doc (hidden)] pub use $ private as $ public ; } }
    define_backend_caller! { gfx_if_metal , gfx_if_metal_hidden , "metal" if all (feature = "metal" , any (target_os = "macos" , target_os = "ios")) }
    define_backend_caller! { gfx_if_gles , gfx_if_gles_hidden , "gles" if feature = "gles" }
    #[macro_export]
    macro_rules ! gfx_select { ($ id : expr => $ global : ident .$ method : ident ($ ($ param : expr) , *)) => { match $ id . backend () { wgc::hal :: wgt :: Backend :: Metal => $ crate :: gfx_if_metal ! ($ global .$ method ::< $ crate :: api :: Metal > ($ ($ param) , *)) , wgc::hal :: wgt :: Backend :: Gl => $ crate :: gfx_if_gles ! ($ global .$ method ::< $ crate :: api :: Gles > ($ ($ param) , +)) , other => panic ! ("Unexpected backend {:?}" , other) , } } ; }
}

mod hal {
    pub(crate) mod empty {
        #[derive(Clone)]
        pub struct Api;
        pub struct Context;
        impl crate::hal::Api for Api {
            type Instance = Context;
        }
        impl crate::hal::Instance<Api> for Context {}
    }

    pub(crate) mod gles {
        mod egl {
            pub struct Instance {}
            impl crate::hal::Instance<super::Api> for Instance {}
        }
        use self::egl::Instance;
        #[derive(Clone)]
        pub struct Api;
        impl crate::hal::Api for Api {
            type Instance = Instance;
        }
    }
    pub(crate) mod metal {
        #[derive(Clone)]
        pub struct Api;
        impl crate::hal::Api for Api {
            type Instance = Instance;
        }
        pub struct Instance {}
        impl crate::hal::Instance<Api> for Instance {}
    }
    pub mod api {
        pub use super::empty::Api as Empty;
        pub use super::gles::Api as Gles;
        pub use super::metal::Api as Metal;
    }
    use thiserror::Error;
    use wgpu_types::WasmNotSend;
    use wgpu_types::WasmNotSync;

    #[derive(Clone, Debug, Eq, PartialEq, Error)]
    #[error("Not supported")]
    pub struct InstanceError;
    pub trait Api: Clone + Sized {
        type Instance: Instance<Self>;
    }
    pub trait Instance<A: Api>: Sized + WasmNotSend + WasmNotSync {
        fn init(_desc: &InstanceDescriptor) -> Result<Self, InstanceError> {
            unimplemented!()
        }
        fn enumerate_adapters(&self) -> Vec<ExposedAdapter> {
            unimplemented!()
        }
    }
    #[derive(Clone, Debug)]
    pub struct InstanceDescriptor<'a> {
        pub name: &'a str,
    }
    #[derive(Debug)]
    pub struct ExposedAdapter {}
}
