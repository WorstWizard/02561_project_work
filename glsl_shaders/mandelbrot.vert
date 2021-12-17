#version 450

layout(location = 0) out vec2 complexPos;

vec2 positions[4] = {
    vec2(-1.0,-1.0),
    vec2( 1.0,-1.0),
    vec2(-1.0, 1.0),
    vec2( 1.0, 1.0)
};

float top_y = 0.6;
float bot_y = 0.5;
float left_x = -0.6;
float right_x = -0.5;

vec2 complexPositions[4] = {
    vec2( left_x, top_y),
    vec2(right_x, top_y),
    vec2( left_x, bot_y),
    vec2(right_x, bot_y)
};

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    complexPos = complexPositions[gl_VertexIndex];
}