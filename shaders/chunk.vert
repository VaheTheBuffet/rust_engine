#version 450 core

layout (location = 0) in int compressed_data;

layout(std140, binding = 0) uniform UniformBufferObject {
    mat4 m_model;
    mat4 m_view;
    mat4 m_proj;
} ubo;    

layout(location = 0) flat out float shading;
layout(location = 1) flat out int voxel_id;
layout(location = 2) flat out int face_id;
layout(location = 3) out vec3 vertex_pos;

ivec3 pos;

void unpack_data(int compressed_data) {
    int COORD_STRIDE = 6; int COORD_MASK = (1<<COORD_STRIDE)-1;
    int FACE_ID_STRIDE = 3;
    int VOXEL_ID_STRIDE = 4; int VOXEL_ID_MASK = (1<<VOXEL_ID_STRIDE)-1;

    voxel_id = compressed_data & VOXEL_ID_MASK; compressed_data >>= VOXEL_ID_STRIDE;
    int z = compressed_data & COORD_MASK; compressed_data >>= COORD_STRIDE;
    int y = compressed_data & COORD_MASK; compressed_data >>= COORD_STRIDE;
    int x = compressed_data & COORD_MASK; compressed_data >>= COORD_STRIDE;
    face_id = compressed_data;
    pos = ivec3(x,y,z);
}

float get_shading(int n) {
    return float[6](
        1.0, 0.4, 0.6, 0.4, 0.7, 0.5
    )[n];
}

const vec2 uv[4] = vec2[] (
    vec2(0,0), vec2(1,0), vec2(1,1), vec2(0,1)
);

const int uv_indices[6] = int[] (
    3, 2, 1, 1, 0, 3
);

const vec2 face_texture_offset[6] = vec2[] (
    vec2(2,0), vec2(0,0), vec2(1,0), vec2(1,0), vec2(1,0), vec2(1,0)
);

void main()
{
    unpack_data(compressed_data);
    shading = get_shading(face_id);
    vertex_pos = (vec4(pos, 1.0) * ubo.m_model).xyz;
    gl_Position = vec4(pos, 1.0) * ubo.m_model * ubo.m_view * ubo.m_proj;
};