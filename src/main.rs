use js_sys::Array;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    console, window, GpuAdapter, GpuAdapterInfo, GpuAutoLayoutMode, GpuBufferDescriptor,
    GpuCanvasConfiguration, GpuCanvasContext, GpuColorDict, GpuColorTargetState, GpuDevice,
    GpuDeviceDescriptor, GpuFragmentState, GpuLoadOp, GpuPowerPreference, GpuPrimitiveState,
    GpuPrimitiveTopology, GpuRenderPassColorAttachment, GpuRenderPassDescriptor,
    GpuRenderPipelineDescriptor, GpuRequestAdapterOptions, GpuShaderModuleDescriptor, GpuStoreOp,
    GpuVertexAttribute, GpuVertexBufferLayout, GpuVertexFormat, GpuVertexState, GpuVertexStepMode,
    HtmlCanvasElement,
};
use yew::prelude::*;

#[function_component(WebGpuComponent)]
pub fn webgpu_component() -> Html {
    let canvas_ref = NodeRef::default();

    let canvas_clone = canvas_ref.clone();
    use_effect(move || {
        spawn_local(async move {
            let canvas: HtmlCanvasElement = canvas_clone
                .cast()
                .expect("Failed to cast to canvas element");
            let webgpu_context: GpuCanvasContext = canvas
                .get_context("webgpu")
                .unwrap()
                .unwrap()
                .dyn_into()
                .expect("Failed to get gpu context");

            let gpu = window().unwrap().navigator().gpu();

            let mut adapter_options = GpuRequestAdapterOptions::new();
            adapter_options.power_preference(GpuPowerPreference::HighPerformance);

            // Get GPU Adapter
            let adapter: GpuAdapter =
                JsFuture::from(gpu.request_adapter_with_options(&adapter_options))
                    .await
                    .and_then(JsCast::dyn_into)
                    .expect("Error getting adapter");

            let adapter_info: GpuAdapterInfo = JsFuture::from(adapter.request_adapter_info())
                .await
                .and_then(JsCast::dyn_into)
                .expect("Error getting adapter info");

            console::log_1(&adapter_info.architecture().into());

            // Request GPU Device
            let mapped_desc = GpuDeviceDescriptor::new();
            let device: GpuDevice =
                JsFuture::from(adapter.request_device_with_descriptor(&mapped_desc))
                    .await
                    .and_then(JsCast::dyn_into)
                    .expect("Error requesting device");

            console::log_1(&"Printing device features".into());
            let device_features = device.features();
            for f in device_features.values() {
                console::log_1(&f.unwrap());
            }

            // Configure canvas context
            canvas.set_width(1980);
            canvas.set_height(1080);
            let canvas_format = gpu.get_preferred_canvas_format();

            let canvas_config = GpuCanvasConfiguration::new(&device, canvas_format);
            webgpu_context.configure(&canvas_config);

            // Create vertex buffer
            let vertices: [f32; 12] = [
                -0.7, -0.7, 0.0, 1.0, 0.7, -0.7, 0.0, 1.0, 0.0, 0.7, 0.0, 1.0,
            ];

            // LESSON use size in bytes
            let mut buffer_desc = GpuBufferDescriptor::new((vertices.len() * 4) as f64, 0x0020);
            buffer_desc.mapped_at_creation(true);

            let buffer = device.create_buffer(&buffer_desc);
            let mapped_array = js_sys::Float32Array::new(&buffer.get_mapped_range());
            mapped_array.copy_from(&vertices);
            buffer.unmap();

            // Compile shaders
            let shader_code = include_str!("basic.wgsl");
            let module_desc = GpuShaderModuleDescriptor::new(shader_code);

            let shader_module = device.create_shader_module(&module_desc);

            // Create pipeline
            let auto_layout = wasm_bindgen::JsValue::from(GpuAutoLayoutMode::Auto);

            let vertex_attribute = GpuVertexAttribute::new(GpuVertexFormat::Float32x4, 0 as f64, 0);
            let vertex_attributes = Array::new();
            vertex_attributes.push(&vertex_attribute);

            let mut buffer_layout = GpuVertexBufferLayout::new(16.0, &vertex_attributes);
            buffer_layout.step_mode(GpuVertexStepMode::Vertex);
            let buffer_layouts = Array::new();
            buffer_layouts.push(&buffer_layout);

            let mut vertex_state = GpuVertexState::new("vertex_main", &shader_module);
            vertex_state.buffers(&buffer_layouts);

            let mapped_color_state = GpuColorTargetState::new(canvas_format);
            let color_states = Array::new();
            color_states.push(&mapped_color_state);

            let frag_state = GpuFragmentState::new("fragment_main", &shader_module, &color_states);

            let mut primitive_state = GpuPrimitiveState::new();
            primitive_state.topology(GpuPrimitiveTopology::TriangleList);

            let mut pipeline_desc = GpuRenderPipelineDescriptor::new(&auto_layout, &vertex_state);
            pipeline_desc.fragment(&frag_state);
            pipeline_desc.primitive(&primitive_state);

            let pipeline = device.create_render_pipeline(&pipeline_desc);

            // Create render pass for rendering
            let back_buffer = webgpu_context.get_current_texture().create_view();
            let mut attachment = GpuRenderPassColorAttachment::new(
                GpuLoadOp::Clear,
                GpuStoreOp::Store,
                &back_buffer,
            );
            attachment.clear_value(&GpuColorDict::new(1.0, 0.1, 0.0, 1.0));
            let attachments = Array::new();
            attachments.push(&attachment);

            let encoder = device.create_command_encoder();

            let pass_desc = GpuRenderPassDescriptor::new(&attachments);
            let render_pass = encoder.begin_render_pass(&pass_desc);

            render_pass.set_pipeline(&pipeline);
            render_pass.set_vertex_buffer(0, &buffer);
            render_pass.draw(3);

            render_pass.end();
            let encoder_buffers = Array::new();
            encoder_buffers.push(&encoder.finish().into());

            device.queue().submit(&encoder_buffers);
        });

        // Cleanup function
        || {
            // ... cleanup any resources or listeners ...
        }
    });

    html! {
        <canvas ref={canvas_ref.clone()} style="width: 100%" />
    }
}

fn main() {
    yew::Renderer::<WebGpuComponent>::new().render();
}
