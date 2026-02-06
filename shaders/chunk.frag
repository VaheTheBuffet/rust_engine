#version 450 core

layout(location = 0) flat in float shading;
layout(location = 1) flat in int voxel_id;
layout(location = 2) flat in int face_id;
layout(location = 3) in vec3 vertex_pos;

layout(location = 0) out vec4 FragColor;

const vec2 face_texture_offset[6] = vec2[] (
    vec2(2,0), vec2(0,0), vec2(1,0), vec2(1,0), vec2(1,0), vec2(1,0)
);

vec2 uv[6] = vec2[6](
    fract(vertex_pos.xz), vec2(0,1)+vec2(1,-1)*fract(vertex_pos.xz),
    vec2(1,1)+vec2(-1,-1)*fract(vertex_pos.zy), vec2(0,1)+vec2(1,-1)*fract(vertex_pos.zy),
    vec2(0,1)+vec2(1,-1)*fract(vertex_pos.xy), (1,1)+(-1,-1)*fract(vertex_pos.xy)
);

//#define TESTING
#ifdef TESTING
layout(binding = 1) uniform sampler2D test;
#else
layout(binding = 1) uniform sampler2DArray tex_array;
#endif

void main()
{
    vec2 uv_coords = (uv[face_id] + face_texture_offset[face_id]) * vec2(1/3.0,1);
#ifdef TESTING
    FragColor = texture(test, uv_coords);
#else
    FragColor = texture(tex_array, vec3(uv_coords, voxel_id));
#endif

    FragColor *= shading;
};