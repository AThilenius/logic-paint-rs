/* tslint:disable */
/* eslint-disable */
export function upc_normalize(a: number): number;
export function upc_denormalize(a: number): number;
export function __wbg_normalizedcell_free(a: number, b: number): void;
export function __wbg_get_normalizedcell_metal(a: number): number;
export function __wbg_set_normalizedcell_metal(a: number, b: number): void;
export function __wbg_get_normalizedcell_si(a: number): number;
export function __wbg_set_normalizedcell_si(a: number, b: number): void;
export function __wbg_placement_free(a: number, b: number): void;
export function __wbg_get_placement_up(a: number): number;
export function __wbg_set_placement_up(a: number, b: number): void;
export function __wbg_get_placement_right(a: number): number;
export function __wbg_set_placement_right(a: number, b: number): void;
export function __wbg_get_placement_down(a: number): number;
export function __wbg_set_placement_down(a: number, b: number): void;
export function __wbg_get_placement_left(a: number): number;
export function __wbg_set_placement_left(a: number, b: number): void;
export function __wbg_upc_free(a: number, b: number): void;
export function __wbg_viewport_free(a: number, b: number): void;
export function viewport_new(a: number): number;
export function viewport_draw(a: number, b: number, c: number, d: number): void;
export function __wbg_camera_free(a: number, b: number): void;
export function __wbg_get_camera_translation(a: number): number;
export function __wbg_set_camera_translation(a: number, b: number): void;
export function __wbg_get_camera_scale(a: number): number;
export function __wbg_set_camera_scale(a: number, b: number): void;
export function __wbg_get_camera_size(a: number): number;
export function __wbg_set_camera_size(a: number, b: number): void;
export function camera_new_translation_scale(a: number, b: number): number;
export function camera_project_screen_point_to_world(a: number, b: number): number;
export function camera_project_screen_point_to_cell(a: number, b: number): number;
export function camera_project_cell_coord_to_screen_point(a: number, b: number): number;
export function __wbg_buffer_free(a: number, b: number): void;
export function buffer_new(): number;
export function buffer_get_cell(a: number, b: number): number;
export function buffer_set_cell(a: number, b: number, c: number): void;
export function buffer_clone_selection(a: number, b: number, c: number): number;
export function buffer_paste_at(a: number, b: number, c: number): void;
export function buffer_rotate_to_new(a: number): number;
export function buffer_mirror_to_new(a: number): number;
export function buffer_fix_all_cells(a: number): void;
export function buffer_cell_count(a: number): number;
export function __wbg_pin_free(a: number, b: number): void;
export function __wbg_get_pin_cell_coord(a: number): number;
export function __wbg_set_pin_cell_coord(a: number, b: number): void;
export function __wbg_get_pin_trigger(a: number): number;
export function __wbg_set_pin_trigger(a: number, b: number): void;
export function __wbg_get_pin_si_output_high(a: number): number;
export function __wbg_set_pin_si_output_high(a: number, b: number): void;
export function __wbg_get_pin_si_input_high(a: number): number;
export function __wbg_set_pin_si_input_high(a: number, b: number): void;
export function __wbg_socket_free(a: number, b: number): void;
export function __wbg_get_socket_always_update(a: number): number;
export function __wbg_set_socket_always_update(a: number, b: number): void;
export function pin_new(a: number, b: number): number;
export function socket_new(a: number, b: number, c: number, d: number): number;
export function buffer_to_base64_string(a: number, b: number): void;
export function buffer_to_bytes(a: number, b: number): void;
export function buffer_from_base64_string(a: number, b: number, c: number): void;
export function buffer_from_bytes(a: number, b: number, c: number): void;
export function __wbg_boolstate_free(a: number, b: number): void;
export function __wbg_get_boolstate_clicked(a: number): number;
export function __wbg_set_boolstate_clicked(a: number, b: number): void;
export function __wbg_get_boolstate_down(a: number): number;
export function __wbg_set_boolstate_down(a: number, b: number): void;
export function __wbg_get_boolstate_released(a: number): number;
export function __wbg_set_boolstate_released(a: number, b: number): void;
export function __wbg_keystate_free(a: number, b: number): void;
export function __wbg_get_keystate_key_code(a: number, b: number): void;
export function __wbg_set_keystate_key_code(a: number, b: number, c: number): void;
export function __wbg_get_keystate_key(a: number, b: number): void;
export function __wbg_set_keystate_key(a: number, b: number, c: number): void;
export function __wbg_get_keystate_state(a: number): number;
export function __wbg_set_keystate_state(a: number, b: number): void;
export function __wbg_drag_free(a: number, b: number): void;
export function __wbg_get_drag_start(a: number): number;
export function __wbg_set_drag_start(a: number, b: number): void;
export function __wbg_get_drag_initial_impulse_vertical(a: number): number;
export function __wbg_set_drag_initial_impulse_vertical(a: number, b: number): void;
export function __wbg_iostate_free(a: number, b: number): void;
export function __wbg_get_iostate_hovered(a: number): number;
export function __wbg_set_iostate_hovered(a: number, b: number): void;
export function __wbg_get_iostate_primary(a: number): number;
export function __wbg_set_iostate_primary(a: number, b: number): void;
export function __wbg_get_iostate_secondary(a: number): number;
export function __wbg_set_iostate_secondary(a: number, b: number): void;
export function __wbg_get_iostate_drag(a: number): number;
export function __wbg_set_iostate_drag(a: number, b: number): void;
export function __wbg_get_iostate_keys(a: number, b: number): void;
export function __wbg_set_iostate_keys(a: number, b: number, c: number): void;
export function __wbg_get_iostate_screen_point(a: number): number;
export function __wbg_set_iostate_screen_point(a: number, b: number): void;
export function __wbg_get_iostate_cell(a: number): number;
export function __wbg_set_iostate_cell(a: number, b: number): void;
export function __wbg_get_iostate_scroll_delta_y(a: number): number;
export function __wbg_set_iostate_scroll_delta_y(a: number, b: number): void;
export function iostate_new(): number;
export function iostate_event_key_down(a: number, b: number): void;
export function iostate_event_key_up(a: number, b: number): void;
export function iostate_event_mouse(a: number, b: number, c: number): void;
export function iostate_event_mouse_presence(a: number, b: number): void;
export function iostate_event_wheel(a: number, b: number): void;
export function iostate_get_key(a: number, b: number, c: number): number;
export function iostate_get_key_code(a: number, b: number, c: number): number;
export function iostate_get_drag_path(a: number, b: number): void;
export function buffer_draw_si(a: number, b: number, c: number, d: number, e: number): void;
export function buffer_draw_metal(a: number, b: number, c: number, d: number): void;
export function buffer_clear_si(a: number, b: number, c: number, d: number): void;
export function buffer_clear_metal(a: number, b: number, c: number, d: number): void;
export function buffer_draw_via(a: number, b: number): void;
export function buffer_clear_selection(a: number, b: number): void;
export function buffer_clear_selection_border(a: number, b: number): void;
export function buffer_draw_si_link(a: number, b: number, c: number, d: number): void;
export function buffer_draw_metal_link(a: number, b: number, c: number): void;
export function buffer_clear_cell_si(a: number, b: number): void;
export function buffer_clear_cell_metal(a: number, b: number): void;
export function __wbg_cellcoord_free(a: number, b: number): void;
export function __wbg_get_cellcoord_0(a: number): number;
export function __wbg_set_cellcoord_0(a: number, b: number): void;
export function cellcoord__wasm_ctor(a: number, b: number): number;
export function __wbg_editor_free(a: number, b: number): void;
export function __wbg_get_editor_buffer(a: number): number;
export function __wbg_set_editor_buffer(a: number, b: number): void;
export function __wbg_get_editor_mask(a: number): number;
export function __wbg_set_editor_mask(a: number, b: number): void;
export function __wbg_get_editor_selection(a: number): number;
export function __wbg_set_editor_selection(a: number, b: number): void;
export function __wbg_get_editor_cursor_coord(a: number): number;
export function __wbg_set_editor_cursor_coord(a: number, b: number): void;
export function __wbg_get_editor_cursor_style(a: number, b: number): void;
export function __wbg_set_editor_cursor_style(a: number, b: number, c: number): void;
export function __wbg_editordispatchresult_free(a: number, b: number): void;
export function __wbg_get_editordispatchresult_buffer_persist(a: number): number;
export function __wbg_set_editordispatchresult_buffer_persist(a: number, b: number): void;
export function __wbg_get_editordispatchresult_tools_persist(a: number, b: number): void;
export function __wbg_set_editordispatchresult_tools_persist(a: number, b: number, c: number): void;
export function __wbg_toolpersist_free(a: number, b: number): void;
export function __wbg_get_toolpersist_tool_name(a: number, b: number): void;
export function __wbg_set_toolpersist_tool_name(a: number, b: number, c: number): void;
export function __wbg_get_toolpersist_serialized_state(a: number, b: number): void;
export function __wbg_set_toolpersist_serialized_state(a: number, b: number, c: number): void;
export function editor_new(): number;
export function editor_dispatch_event(a: number, b: number, c: number): number;
export function import_legacy_blueprint(a: number, b: number, c: number): void;
export function __wbg_compilerresults_free(a: number, b: number): void;
export function __wbg_atom_free(a: number, b: number): void;
export function __wbg_get_atom_coord(a: number): number;
export function __wbg_set_atom_coord(a: number, b: number): void;
export function __wbg_get_atom_part(a: number): number;
export function __wbg_set_atom_part(a: number, b: number): void;
export function compilerresults_from_buffer(a: number): number;
export function compilerresults_get_trace_atoms(a: number, b: number, c: number): void;
export function __wbg_mask_free(a: number, b: number): void;
export function __wbg_selection_free(a: number, b: number): void;
export function __wbg_get_selection_lower_left(a: number): number;
export function __wbg_set_selection_lower_left(a: number, b: number): void;
export function __wbg_get_selection_upper_right(a: number): number;
export function __wbg_set_selection_upper_right(a: number, b: number): void;
export function main(): void;
export function __wbg_get_vec2_x(a: number): number;
export function __wbg_set_vec2_x(a: number, b: number): void;
export function __wbg_get_vec2_y(a: number): number;
export function __wbg_set_vec2_y(a: number, b: number): void;
export function vec2_wasm_bindgen_ctor(a: number, b: number): number;
export function __wbg_vec3a_free(a: number, b: number): void;
export function __wbg_get_vec3a_x(a: number): number;
export function __wbg_set_vec3a_x(a: number, b: number): void;
export function __wbg_get_vec3a_y(a: number): number;
export function __wbg_set_vec3a_y(a: number, b: number): void;
export function __wbg_get_vec3a_z(a: number): number;
export function __wbg_set_vec3a_z(a: number, b: number): void;
export function vec3a_wasm_bindgen_ctor(a: number, b: number, c: number): number;
export function __wbg_get_vec4_w(a: number): number;
export function __wbg_set_vec4_w(a: number, b: number): void;
export function vec4_wasm_bindgen_ctor(a: number, b: number, c: number, d: number): number;
export function __wbg_ivec2_free(a: number, b: number): void;
export function __wbg_get_ivec2_x(a: number): number;
export function __wbg_set_ivec2_x(a: number, b: number): void;
export function __wbg_get_ivec2_y(a: number): number;
export function __wbg_set_ivec2_y(a: number, b: number): void;
export function ivec2_wasm_bindgen_ctor(a: number, b: number): number;
export function __wbg_ivec4_free(a: number, b: number): void;
export function __wbg_get_ivec4_z(a: number): number;
export function __wbg_set_ivec4_z(a: number, b: number): void;
export function __wbg_get_ivec4_w(a: number): number;
export function __wbg_set_ivec4_w(a: number, b: number): void;
export function ivec4_wasm_bindgen_ctor(a: number, b: number, c: number, d: number): number;
export function __wbg_get_vec4_x(a: number): number;
export function __wbg_get_vec4_y(a: number): number;
export function __wbg_get_vec4_z(a: number): number;
export function __wbg_get_ivec4_x(a: number): number;
export function __wbg_get_ivec4_y(a: number): number;
export function __wbg_vec4_free(a: number, b: number): void;
export function __wbg_vec2_free(a: number, b: number): void;
export function __wbg_set_vec4_x(a: number, b: number): void;
export function __wbg_set_vec4_y(a: number, b: number): void;
export function __wbg_set_vec4_z(a: number, b: number): void;
export function __wbg_set_ivec4_x(a: number, b: number): void;
export function __wbg_set_ivec4_y(a: number, b: number): void;
export function __wbg_u64vec2_free(a: number, b: number): void;
export function __wbg_get_u64vec2_x(a: number): number;
export function __wbg_set_u64vec2_x(a: number, b: number): void;
export function __wbg_get_u64vec2_y(a: number): number;
export function __wbg_set_u64vec2_y(a: number, b: number): void;
export function u64vec2_wasm_bindgen_ctor(a: number, b: number): number;
export function __wbg_u64vec4_free(a: number, b: number): void;
export function __wbg_get_u64vec4_z(a: number): number;
export function __wbg_set_u64vec4_z(a: number, b: number): void;
export function __wbg_get_u64vec4_w(a: number): number;
export function __wbg_set_u64vec4_w(a: number, b: number): void;
export function u64vec4_wasm_bindgen_ctor(a: number, b: number, c: number, d: number): number;
export function __wbg_get_u64vec4_x(a: number): number;
export function __wbg_get_u64vec4_y(a: number): number;
export function __wbg_set_u64vec4_x(a: number, b: number): void;
export function __wbg_set_u64vec4_y(a: number, b: number): void;
export function __wbg_dvec2_free(a: number, b: number): void;
export function __wbg_get_dvec2_x(a: number): number;
export function __wbg_set_dvec2_x(a: number, b: number): void;
export function __wbg_get_dvec2_y(a: number): number;
export function __wbg_set_dvec2_y(a: number, b: number): void;
export function dvec2_wasm_bindgen_ctor(a: number, b: number): number;
export function __wbg_dvec4_free(a: number, b: number): void;
export function __wbg_get_dvec4_z(a: number): number;
export function __wbg_set_dvec4_z(a: number, b: number): void;
export function __wbg_get_dvec4_w(a: number): number;
export function __wbg_set_dvec4_w(a: number, b: number): void;
export function dvec4_wasm_bindgen_ctor(a: number, b: number, c: number, d: number): number;
export function __wbg_get_i64vec2_x(a: number): number;
export function __wbg_set_i64vec2_x(a: number, b: number): void;
export function __wbg_get_i64vec2_y(a: number): number;
export function __wbg_set_i64vec2_y(a: number, b: number): void;
export function i64vec2_wasm_bindgen_ctor(a: number, b: number): number;
export function __wbg_u64vec3_free(a: number, b: number): void;
export function __wbg_get_u64vec3_z(a: number): number;
export function __wbg_set_u64vec3_z(a: number, b: number): void;
export function u64vec3_wasm_bindgen_ctor(a: number, b: number, c: number): number;
export function __wbg_get_dvec4_x(a: number): number;
export function __wbg_get_dvec4_y(a: number): number;
export function __wbg_get_u64vec3_x(a: number): number;
export function __wbg_get_u64vec3_y(a: number): number;
export function __wbg_i64vec2_free(a: number, b: number): void;
export function __wbg_set_dvec4_x(a: number, b: number): void;
export function __wbg_set_dvec4_y(a: number, b: number): void;
export function __wbg_set_u64vec3_x(a: number, b: number): void;
export function __wbg_set_u64vec3_y(a: number, b: number): void;
export function __wbg_mat4_free(a: number, b: number): void;
export function __wbg_get_mat4_x_axis(a: number): number;
export function __wbg_set_mat4_x_axis(a: number, b: number): void;
export function __wbg_get_mat4_y_axis(a: number): number;
export function __wbg_set_mat4_y_axis(a: number, b: number): void;
export function __wbg_get_mat4_z_axis(a: number): number;
export function __wbg_set_mat4_z_axis(a: number, b: number): void;
export function __wbg_get_mat4_w_axis(a: number): number;
export function __wbg_set_mat4_w_axis(a: number, b: number): void;
export function mat4_wasm_bindgen_ctor(a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number, p: number): number;
export function __wbg_i16vec2_free(a: number, b: number): void;
export function __wbg_get_i16vec2_x(a: number): number;
export function __wbg_set_i16vec2_x(a: number, b: number): void;
export function __wbg_get_i16vec2_y(a: number): number;
export function __wbg_set_i16vec2_y(a: number, b: number): void;
export function i16vec2_wasm_bindgen_ctor(a: number, b: number): number;
export function __wbg_i16vec3_free(a: number, b: number): void;
export function __wbg_get_i16vec3_z(a: number): number;
export function __wbg_set_i16vec3_z(a: number, b: number): void;
export function i16vec3_wasm_bindgen_ctor(a: number, b: number, c: number): number;
export function __wbg_get_i16vec4_w(a: number): number;
export function __wbg_set_i16vec4_w(a: number, b: number): void;
export function i16vec4_wasm_bindgen_ctor(a: number, b: number, c: number, d: number): number;
export function __wbg_uvec2_free(a: number, b: number): void;
export function __wbg_get_uvec2_x(a: number): number;
export function __wbg_set_uvec2_x(a: number, b: number): void;
export function __wbg_get_uvec2_y(a: number): number;
export function __wbg_set_uvec2_y(a: number, b: number): void;
export function uvec2_wasm_bindgen_ctor(a: number, b: number): number;
export function __wbg_set_i16vec3_x(a: number, b: number): void;
export function __wbg_set_i16vec3_y(a: number, b: number): void;
export function __wbg_set_i16vec4_x(a: number, b: number): void;
export function __wbg_set_i16vec4_y(a: number, b: number): void;
export function __wbg_set_i16vec4_z(a: number, b: number): void;
export function __wbg_get_i16vec3_x(a: number): number;
export function __wbg_get_i16vec3_y(a: number): number;
export function __wbg_get_i16vec4_x(a: number): number;
export function __wbg_get_i16vec4_y(a: number): number;
export function __wbg_get_i16vec4_z(a: number): number;
export function __wbg_i16vec4_free(a: number, b: number): void;
export function __wbg_mat3_free(a: number, b: number): void;
export function __wbg_get_mat3_x_axis(a: number): number;
export function __wbg_set_mat3_x_axis(a: number, b: number): void;
export function __wbg_get_mat3_y_axis(a: number): number;
export function __wbg_set_mat3_y_axis(a: number, b: number): void;
export function __wbg_get_mat3_z_axis(a: number): number;
export function __wbg_set_mat3_z_axis(a: number, b: number): void;
export function mat3_wasm_bindgen_ctor(a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number): number;
export function __wbg_ivec3_free(a: number, b: number): void;
export function __wbg_get_ivec3_x(a: number): number;
export function __wbg_set_ivec3_x(a: number, b: number): void;
export function __wbg_get_ivec3_y(a: number): number;
export function __wbg_set_ivec3_y(a: number, b: number): void;
export function __wbg_get_ivec3_z(a: number): number;
export function __wbg_set_ivec3_z(a: number, b: number): void;
export function ivec3_wasm_bindgen_ctor(a: number, b: number, c: number): number;
export function __wbg_dmat2_free(a: number, b: number): void;
export function __wbg_get_dmat2_x_axis(a: number): number;
export function __wbg_set_dmat2_x_axis(a: number, b: number): void;
export function __wbg_get_dmat2_y_axis(a: number): number;
export function __wbg_set_dmat2_y_axis(a: number, b: number): void;
export function dmat2_wasm_bindgen_ctor(a: number, b: number, c: number, d: number): number;
export function __wbg_dmat3_free(a: number, b: number): void;
export function __wbg_get_dmat3_x_axis(a: number): number;
export function __wbg_set_dmat3_x_axis(a: number, b: number): void;
export function __wbg_get_dmat3_y_axis(a: number): number;
export function __wbg_set_dmat3_y_axis(a: number, b: number): void;
export function __wbg_get_dmat3_z_axis(a: number): number;
export function __wbg_set_dmat3_z_axis(a: number, b: number): void;
export function dmat3_wasm_bindgen_ctor(a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number): number;
export function __wbg_dquat_free(a: number, b: number): void;
export function __wbg_get_dquat_x(a: number): number;
export function __wbg_set_dquat_x(a: number, b: number): void;
export function __wbg_get_dquat_y(a: number): number;
export function __wbg_set_dquat_y(a: number, b: number): void;
export function __wbg_get_dquat_z(a: number): number;
export function __wbg_set_dquat_z(a: number, b: number): void;
export function __wbg_get_dquat_w(a: number): number;
export function __wbg_set_dquat_w(a: number, b: number): void;
export function __wbg_dvec3_free(a: number, b: number): void;
export function dvec3_wasm_bindgen_ctor(a: number, b: number, c: number): number;
export function __wbg_uvec3_free(a: number, b: number): void;
export function __wbg_get_uvec3_x(a: number): number;
export function __wbg_set_uvec3_x(a: number, b: number): void;
export function __wbg_get_uvec3_y(a: number): number;
export function __wbg_set_uvec3_y(a: number, b: number): void;
export function __wbg_get_uvec3_z(a: number): number;
export function __wbg_set_uvec3_z(a: number, b: number): void;
export function uvec3_wasm_bindgen_ctor(a: number, b: number, c: number): number;
export function __wbg_uvec4_free(a: number, b: number): void;
export function __wbg_get_uvec4_w(a: number): number;
export function __wbg_set_uvec4_w(a: number, b: number): void;
export function uvec4_wasm_bindgen_ctor(a: number, b: number, c: number, d: number): number;
export function __wbg_get_dvec3_x(a: number): number;
export function __wbg_get_dvec3_y(a: number): number;
export function __wbg_get_dvec3_z(a: number): number;
export function __wbg_get_uvec4_x(a: number): number;
export function __wbg_get_uvec4_y(a: number): number;
export function __wbg_get_uvec4_z(a: number): number;
export function __wbg_set_dvec3_x(a: number, b: number): void;
export function __wbg_set_dvec3_y(a: number, b: number): void;
export function __wbg_set_dvec3_z(a: number, b: number): void;
export function __wbg_set_uvec4_x(a: number, b: number): void;
export function __wbg_set_uvec4_y(a: number, b: number): void;
export function __wbg_set_uvec4_z(a: number, b: number): void;
export function __wbg_mat2_free(a: number, b: number): void;
export function __wbg_get_mat2_x_axis(a: number): number;
export function __wbg_set_mat2_x_axis(a: number, b: number): void;
export function __wbg_get_mat2_y_axis(a: number): number;
export function __wbg_set_mat2_y_axis(a: number, b: number): void;
export function mat2_wasm_bindgen_ctor(a: number, b: number, c: number, d: number): number;
export function __wbg_mat3a_free(a: number, b: number): void;
export function __wbg_get_mat3a_x_axis(a: number): number;
export function __wbg_set_mat3a_x_axis(a: number, b: number): void;
export function __wbg_get_mat3a_y_axis(a: number): number;
export function __wbg_set_mat3a_y_axis(a: number, b: number): void;
export function __wbg_get_mat3a_z_axis(a: number): number;
export function __wbg_set_mat3a_z_axis(a: number, b: number): void;
export function mat3a_wasm_bindgen_ctor(a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number): number;
export function __wbg_dmat4_free(a: number, b: number): void;
export function __wbg_get_dmat4_x_axis(a: number): number;
export function __wbg_set_dmat4_x_axis(a: number, b: number): void;
export function __wbg_get_dmat4_y_axis(a: number): number;
export function __wbg_set_dmat4_y_axis(a: number, b: number): void;
export function __wbg_get_dmat4_z_axis(a: number): number;
export function __wbg_set_dmat4_z_axis(a: number, b: number): void;
export function __wbg_get_dmat4_w_axis(a: number): number;
export function __wbg_set_dmat4_w_axis(a: number, b: number): void;
export function dmat4_wasm_bindgen_ctor(a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number, p: number): number;
export function __wbg_u16vec2_free(a: number, b: number): void;
export function __wbg_get_u16vec2_x(a: number): number;
export function __wbg_set_u16vec2_x(a: number, b: number): void;
export function __wbg_get_u16vec2_y(a: number): number;
export function __wbg_set_u16vec2_y(a: number, b: number): void;
export function u16vec2_wasm_bindgen_ctor(a: number, b: number): number;
export function __wbg_u16vec3_free(a: number, b: number): void;
export function __wbg_get_u16vec3_z(a: number): number;
export function __wbg_set_u16vec3_z(a: number, b: number): void;
export function u16vec3_wasm_bindgen_ctor(a: number, b: number, c: number): number;
export function __wbg_get_u16vec4_w(a: number): number;
export function __wbg_set_u16vec4_w(a: number, b: number): void;
export function u16vec4_wasm_bindgen_ctor(a: number, b: number, c: number, d: number): number;
export function __wbg_set_u16vec3_x(a: number, b: number): void;
export function __wbg_set_u16vec3_y(a: number, b: number): void;
export function __wbg_set_u16vec4_x(a: number, b: number): void;
export function __wbg_set_u16vec4_y(a: number, b: number): void;
export function __wbg_set_u16vec4_z(a: number, b: number): void;
export function __wbg_get_u16vec3_x(a: number): number;
export function __wbg_get_u16vec3_y(a: number): number;
export function __wbg_get_u16vec4_x(a: number): number;
export function __wbg_get_u16vec4_y(a: number): number;
export function __wbg_get_u16vec4_z(a: number): number;
export function __wbg_u16vec4_free(a: number, b: number): void;
export function __wbg_vec3_free(a: number, b: number): void;
export function __wbg_get_vec3_x(a: number): number;
export function __wbg_set_vec3_x(a: number, b: number): void;
export function __wbg_get_vec3_y(a: number): number;
export function __wbg_set_vec3_y(a: number, b: number): void;
export function __wbg_get_vec3_z(a: number): number;
export function __wbg_set_vec3_z(a: number, b: number): void;
export function vec3_wasm_bindgen_ctor(a: number, b: number, c: number): number;
export function __wbg_quat_free(a: number, b: number): void;
export function __wbg_get_quat_x(a: number): number;
export function __wbg_set_quat_x(a: number, b: number): void;
export function __wbg_get_quat_y(a: number): number;
export function __wbg_set_quat_y(a: number, b: number): void;
export function __wbg_get_quat_z(a: number): number;
export function __wbg_set_quat_z(a: number, b: number): void;
export function __wbg_get_quat_w(a: number): number;
export function __wbg_set_quat_w(a: number, b: number): void;
export function __wbg_i64vec3_free(a: number, b: number): void;
export function __wbg_get_i64vec3_x(a: number): number;
export function __wbg_set_i64vec3_x(a: number, b: number): void;
export function __wbg_get_i64vec3_y(a: number): number;
export function __wbg_set_i64vec3_y(a: number, b: number): void;
export function __wbg_get_i64vec3_z(a: number): number;
export function __wbg_set_i64vec3_z(a: number, b: number): void;
export function i64vec3_wasm_bindgen_ctor(a: number, b: number, c: number): number;
export function __wbg_i64vec4_free(a: number, b: number): void;
export function __wbg_get_i64vec4_w(a: number): number;
export function __wbg_set_i64vec4_w(a: number, b: number): void;
export function i64vec4_wasm_bindgen_ctor(a: number, b: number, c: number, d: number): number;
export function __wbg_get_i64vec4_x(a: number): number;
export function __wbg_get_i64vec4_y(a: number): number;
export function __wbg_get_i64vec4_z(a: number): number;
export function __wbg_set_i64vec4_x(a: number, b: number): void;
export function __wbg_set_i64vec4_y(a: number, b: number): void;
export function __wbg_set_i64vec4_z(a: number, b: number): void;
export const memory: WebAssembly.Memory;
export function __wbindgen_export_1(a: number, b: number): number;
export function __wbindgen_export_2(a: number, b: number, c: number, d: number): number;
export function __wbindgen_add_to_stack_pointer(a: number): number;
export function __wbindgen_export_3(a: number, b: number, c: number): void;
export function __wbindgen_export_4(a: number): void;
export function __wbindgen_thread_destroy(a: number, b: number, c: number): void;
export function __wbindgen_start(a: number): void;
