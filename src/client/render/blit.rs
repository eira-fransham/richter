use crate::client::render::{pipeline::Pipeline, ui::quad::QuadPipeline, GraphicsState};

pub struct BlitPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    bind_group: wgpu::BindGroup,
    format: wgpu::TextureFormat,
    sampler: wgpu::Sampler,
}

impl BlitPipeline {
    pub fn create_bind_group(
        device: &wgpu::Device,
        layouts: &[wgpu::BindGroupLayout],
        sampler: &wgpu::Sampler,
        input: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blit bind group"),
            layout: &layouts[0],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(input),
                },
            ],
        })
    }

    pub fn new(
        device: &wgpu::Device,
        compiler: &mut shaderc::Compiler,
        input: &wgpu::TextureView,
        format: wgpu::TextureFormat,
    ) -> BlitPipeline {
        let (pipeline, bind_group_layouts) = BlitPipeline::create(device, compiler, &[], 1, format);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_max_clamp: 1000.0,
            compare: None,
            anisotropy_clamp: 1,
            ..Default::default()
        });

        let bind_group = Self::create_bind_group(device, &bind_group_layouts, &sampler, input);

        BlitPipeline {
            pipeline,
            bind_group_layouts,
            bind_group,
            format,
            sampler,
        }
    }

    pub fn rebuild(
        &mut self,
        device: &wgpu::Device,
        compiler: &mut shaderc::Compiler,
        input: &wgpu::TextureView,
    ) {
        let layout_refs: Vec<_> = self.bind_group_layouts.iter().collect();
        let pipeline = BlitPipeline::recreate(device, compiler, &layout_refs, 1, self.format);
        self.pipeline = pipeline;
        self.bind_group =
            Self::create_bind_group(device, self.bind_group_layouts(), &self.sampler, input);
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn bind_group_layouts(&self) -> &[wgpu::BindGroupLayout] {
        &self.bind_group_layouts
    }

    pub fn blit<'a>(&'a self, state: &'a GraphicsState, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline());
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, state.quad_pipeline().vertex_buffer().slice(..));
        pass.draw(0..6, 0..1);
    }
}

impl Pipeline for BlitPipeline {
    type VertexPushConstants = ();
    type SharedPushConstants = ();
    type FragmentPushConstants = ();

    type Args = wgpu::TextureFormat;

    fn name() -> &'static str {
        "blit"
    }

    fn bind_group_layout_descriptors() -> Vec<wgpu::BindGroupLayoutDescriptor<'static>> {
        vec![wgpu::BindGroupLayoutDescriptor {
            label: Some("blit bind group"),
            entries: &[
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // blit texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        }]
    }

    fn vertex_shader() -> &'static str {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/blit.vert"))
    }

    fn fragment_shader() -> &'static str {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/blit.frag"))
    }

    fn primitive_state() -> wgpu::PrimitiveState {
        QuadPipeline::primitive_state()
    }

    fn color_target_states_with_args(format: Self::Args) -> Vec<Option<wgpu::ColorTargetState>> {
        QuadPipeline::color_target_states_with_args(format)
    }

    fn depth_stencil_state() -> Option<wgpu::DepthStencilState> {
        None
    }

    fn vertex_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        QuadPipeline::vertex_buffer_layouts()
    }
}