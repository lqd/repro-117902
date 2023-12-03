fn main() {
    let texture_descriptor = wgpu::TextureDescriptor {
        ballast: [0u64; 4096],
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
    };

    let context = wgpu::Global;
    wgpu::device_create_texture(&context, &texture_descriptor);
}
