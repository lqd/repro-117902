fn main() {
    let texture_descriptor = repro_117902::TextureDescriptor {
        ballast: [0u64; 4096],
        format: repro_117902::TextureFormat::Rgba8UnormSrgb,
    };

    let context = repro_117902::Global;
    repro_117902::device_create_texture(&context, &texture_descriptor);
}
