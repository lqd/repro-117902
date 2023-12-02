use std::borrow::Cow::Borrowed;

pub use wgpu_core as core;
pub use wgpu_hal::wgt::Backends;
pub use wgpu_hal::wgt::TextureDimension;
pub use wgpu_hal::wgt::TextureFormat;
pub use wgpu_hal::wgt::TextureUsages;

pub type Label<'a> = Option<&'a str>;
pub type TextureDescriptor<'a> =
    wgpu_hal::wgt::TextureDescriptor<Label<'a>, &'a [wgpu_hal::wgt::TextureFormat]>;

pub fn device_create_texture(
    global: &wgpu_core::global::Global<wgpu_core::identity::IdentityManagerFactory>,
    device: &wgpu_core::id::DeviceId,
    desc: &TextureDescriptor,
) -> wgpu_core::id::TextureId {
    let wgt_desc = desc.map_label_and_view_formats(|l| l.map(Borrowed), |v| v.to_vec());
    let (id, _) = match device.backend() {
        wgpu_hal::wgt::Backend::Metal => {
            global.device_create_texture::<wgpu_core::api::Metal>(*device, &wgt_desc, ())
        }
        wgpu_hal::wgt::Backend::Gl => {
            global.device_create_texture::<wgpu_core::api::Gles>(*device, &wgt_desc, ())
        }
        other => {
            panic!("Unexpected backend {:?}", other);
        }
    };
    id
}
