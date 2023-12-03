pub use wgpu_types::TextureFormat;

#[inline(never)]
pub fn device_create_texture(_global: &Global, desc: &wgpu_types::TextureDescriptor) {
    let format = desc.format;
    std::hint::black_box(format.required_features());
    eprintln!("{:?}", format);
    todo!()
}

pub struct Global;
