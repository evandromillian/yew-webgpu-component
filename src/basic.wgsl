struct VertexOut {
    @builtin(position) position : vec4<f32>,
};

@vertex
fn vertex_main(@location(0) position: vec4<f32>) -> VertexOut
{
    var output : VertexOut;
    output.position = position;
    return output;
} 

@fragment
fn fragment_main(fragData: VertexOut) -> @location(0) vec4<f32>
{
    return vec4(1.0, 0.6, 0.2, 1.0);
}