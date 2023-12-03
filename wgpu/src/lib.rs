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
    global.device_create_texture::<crate::core::api::Metal>(*device, &wgt_desc);
}

pub mod core {
    pub(crate) mod device {
        use crate::core::Label;
        pub(crate) mod global {
            use crate::core::global::Global;
            use crate::core::hal_api::HalApi;
            use crate::core::hub::Token;
            use crate::core::id::DeviceId;
            use crate::core::identity::GlobalIdentityHandlerFactory;
            use crate::core::resource::{self};
            impl<G: GlobalIdentityHandlerFactory> Global<G> {
                pub fn device_create_texture<A: HalApi>(
                    &self,
                    device_id: DeviceId,
                    desc: &resource::TextureDescriptor,
                ) {
                    let hub = A::hub(self);
                    let mut token = Token::root();
                    let (_, mut token) = hub.adapters.read(&mut token);
                    let (device_guard, _) = hub.devices.read(&mut token);
                    let device = device_guard.get(device_id);
                    let _ = device.create_texture(desc);
                    todo!()
                }
            }
        }
        pub(crate) mod resource {
            use super::DeviceDescriptor;
            use crate::core::hal_api::HalApi;
            use crate::core::resource::{self};
            pub struct Device<A: HalApi> {
                pub(crate) features: wgpu_types::Features,
                _marker: std::marker::PhantomData<A>,
            }
            impl<A: HalApi> Device<A> {
                #[inline(always)]
                pub(crate) fn new(desc: &DeviceDescriptor) -> Self {
                    Self {
                        features: desc.features,
                        _marker: std::marker::PhantomData,
                    }
                }
                #[inline(never)]
                pub(super) fn create_texture(
                    &self,
                    desc: &resource::TextureDescriptor,
                ) -> resource::Texture<A> {
                    let format = desc.format;
                    std::hint::black_box(format.required_features());
                    format.guaranteed_format_features(self.features);
                    todo!()
                }
            }
        }
        pub(crate) use resource::Device;
        pub(crate) type DeviceDescriptor<'a> = wgpu_types::DeviceDescriptor<Label<'a>>;
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
                unimplemented!()
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
        use std::marker::PhantomData;
        pub(crate) trait Access<A> {}
        pub(crate) enum Root {}
        impl Access<Surface> for Root {}
        impl<A: HalApi> Access<Adapter<A>> for Root {}
        impl<A: HalApi> Access<Device<A>> for Adapter<A> {}
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
        }
        impl<A: HalApi, F: GlobalIdentityHandlerFactory> Hub<A, F> {
            fn new(factory: &F) -> Self {
                Self {
                    adapters: Registry::new(A::VARIANT, factory),
                    devices: Registry::new(A::VARIANT, factory),
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
        use std::sync::Mutex;
        use wgpu_types::Backend;
        #[derive(Default)]
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
        pub trait IdentityHandler<I> {
            type Input: Clone;
            fn process(&self, _id: Self::Input, _backend: Backend) -> I {
                unimplemented!()
            }
            fn free(&self, _id: I) {
                unimplemented!()
            }
        }
        impl<I: id::TypedId> IdentityHandler<I> for Mutex<IdentityManager> {
            type Input = ();
            fn process(&self, _id: Self::Input, backend: Backend) -> I {
                self.lock().unwrap().alloc(backend)
            }
        }
        pub trait IdentityHandlerFactory<I> {
            type Filter: IdentityHandler<I>;
            fn spawn(&self) -> Self::Filter;
        }
        pub struct IdentityManagerFactory;
        impl<I: id::TypedId> IdentityHandlerFactory<I> for IdentityManagerFactory {
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
            #[inline(always)]
            fn create_device_from_hal(
                &self,
                desc: &DeviceDescriptor,
            ) -> Device<A> {
                Device::new(desc)
            }
            pub(crate) fn create_device(
                &self,
                desc: &DeviceDescriptor,
            ) -> Device<A> {
                self.create_device_from_hal(desc)
            }
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
            ) -> AdapterId {
                let mut token = Token::root();
                let (_, _) = self.surfaces.read(&mut token);
                let id_metal = inputs.find(crate::hal::api::Metal::VARIANT);
                self.select::<crate::hal::api::Metal>(id_metal).unwrap()
            }
        }
        impl<G: GlobalIdentityHandlerFactory> Global<G> {
            pub fn adapter_request_device<A: HalApi>(
                &self,
                adapter_id: AdapterId,
                desc: &DeviceDescriptor,
                id_in: Input<G, DeviceId>,
            ) -> DeviceId {
                let hub = A::hub(self);
                let mut token = Token::root();
                let fid = hub.devices.prepare(id_in);
                let (adapter_guard, mut token) = hub.adapters.read(&mut token);
                let adapter = adapter_guard.get(adapter_id);
                let device = adapter.create_device(desc);
                let id = fid.assign(device, &mut token);
                id.0
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
        use std::sync::RwLock;
        use std::sync::RwLockReadGuard;
        use std::marker::PhantomData;
        use wgpu_types::Backend;
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
                self.data.write().unwrap().insert(self.id, value);
                id::Valid(self.id)
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
                (self.data.read().unwrap(), Token::new())
            }
        }
    }
    pub mod resource {
        use crate::core::Label;
        pub type TextureDescriptor<'a> =
            wgpu_types::TextureDescriptor<Label<'a>, Vec<wgpu_types::TextureFormat>>;
        pub struct Texture<A: crate::hal::Api> {
            _marker: std::marker::PhantomData<A>,
        }
    }
    pub(crate) mod storage {
        use crate::core::id;
        use crate::core::Epoch;
        use std::marker::PhantomData;
        pub(crate) enum Element<T> {
            Vacant,
            Occupied(T, Epoch),
        }
        pub(crate) struct Storage<T, I: id::TypedId> {
            pub(crate) map: Vec<Element<T>>,
            pub(crate) _phantom: PhantomData<I>,
        }
        impl<T, I: id::TypedId> Storage<T, I> {
            pub(crate) fn get(&self, id: I) -> &T {
                let (index, _, _) = id.unzip();
                match self.map.get(index as usize).unwrap() {
                    Element::Occupied(ref v, _) => v,
                    Element::Vacant => panic!("[{}] does not exist", index),
                }
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
}

mod hal {
    pub(crate) mod empty {
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
        pub struct Api;
        impl crate::hal::Api for Api {
            type Instance = Instance;
        }
    }
    pub(crate) mod metal {
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

    pub trait Api: Sized {
        type Instance: Instance<Self>;
    }
    pub trait Instance<A: Api>: Sized {}
}
