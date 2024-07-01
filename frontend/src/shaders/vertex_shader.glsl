#version 300 es
precision mediump float;

in vec2 position;

uniform vec2 resolution;
uniform vec2 camera_pos;
uniform float camera_zoom;

void main() {
  vec2 view_position = (position - camera_pos) * camera_zoom;
  vec2 normalized_position = view_position / resolution;
  vec2 screen_position = vec2(-1.0 + normalized_position.x * 2.0, 1.0 - normalized_position.y * 2.0);
  gl_Position = vec4(screen_position, 0.0, 1.0);
}