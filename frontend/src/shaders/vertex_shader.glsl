#version 300 es
precision mediump float;

in vec2 position;

uniform vec2 resolution;

void main() {
  vec2 screen_position = position / vec2(500,300);
  gl_Position = vec4(screen_position, 0.0, 1.0);
}