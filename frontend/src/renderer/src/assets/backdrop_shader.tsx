// yes, glsl fragment shader in ts source, fight me
export const backdrop_shader =  `
#ifdef GL_ES
  precision lowp float;
#endif

uniform vec2 u_mouse;
uniform vec2 u_resolution;

const vec3 black  = vec3(0.086, 0.086, 0.094);
const vec3 blue   = vec3(0.008, 0.455, 0.722) * 0.4;
const vec3 purple = vec3(0.427, 0.090, 0.600) * 0.6;

const vec2 base_blue_pos    = vec2(1.2, 0.6);
const vec2 base_purple_pos  = vec2(-0.7, -0.3);

const float radius = 1.5;

void main() {
  vec2 st = gl_FragCoord.xy / u_resolution;
  vec2 mouse_modifier = (u_mouse / u_resolution) * 2.0 - vec2(1.0, 1.0);

  vec2 blue_pos   = base_blue_pos - (mouse_modifier * 0.01);
  vec2 purple_pos = base_purple_pos - (mouse_modifier * 0.03);

  float blue_alpha    = 1.0 - distance(st, blue_pos) / radius;
  blue_alpha    = blue_alpha > 0.0 ? blue_alpha : 0.0;
  
  float purple_alpha  = 1.0 - distance(st, purple_pos) / (radius * 1.2);
  purple_alpha  = purple_alpha > 0.0 ? purple_alpha : 0.0;

  vec3 mixed_color = purple * purple_alpha + blue * blue_alpha;
  vec3 final_color = vec3(max(mixed_color.x, black.x), max(mixed_color.y, black.y), max(mixed_color.z, black.z));

  gl_FragColor = vec4(final_color, 1.0);
}`;