fn main() {
    let texture_descriptor = wgpu_types::TextureDescriptor {
        ballast: [0u64; 4096],
        format: wgpu_types::TextureFormat::Rgba8UnormSrgb,
    };

    let context = wgpu::Global;
    wgpu::device_create_texture(&context, &texture_descriptor);
}
