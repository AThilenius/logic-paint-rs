use crate::{
    canvas::{CanvasData, MetalLayer, SiLayer},
    input::{ActiveTools, CanvasInput, ToolType},
};
use bevy::prelude::*;

pub fn handle_canvas_input(
    mut canvas_query: Query<(&mut CanvasData, &CanvasInput)>,
    active_tool: Res<ActiveTools>,
) {
    for (mut canvas, canvas_input) in canvas_query.iter_mut() {
        if !canvas_input.left_pressed && !canvas_input.right_pressed {
            return;
        }

        if let Some(pos) = canvas_input.mouse_position {
            update_cell(&mut canvas, &active_tool, pos, canvas_input.left_pressed);
        }
    }
}

fn update_cell(canvas: &mut CanvasData, active_tool: &ActiveTools, pos: IVec2, left_click: bool) {
    let mut cell = canvas.get_cell_mut(pos);
    match (active_tool.tool_type, left_click) {
        (ToolType::None, _) => {}
        (ToolType::NType, true) => {
            cell.si = match cell.si {
                SiLayer::None | SiLayer::N => SiLayer::N,
                SiLayer::P => SiLayer::NOnP,
                SiLayer::NOnP => SiLayer::NOnP,
                SiLayer::POnN => SiLayer::POnN,
            };
        }
        (ToolType::NType, false) => {
            cell.si = match cell.si {
                SiLayer::None | SiLayer::N => {
                    cell.metal = match cell.metal {
                        MetalLayer::None => MetalLayer::None,
                        MetalLayer::Metal | MetalLayer::MetalAndVia => MetalLayer::Metal,
                    };
                    SiLayer::None
                }
                SiLayer::P => SiLayer::P,
                SiLayer::NOnP => {
                    cell.metal = match cell.metal {
                        MetalLayer::None => MetalLayer::None,
                        MetalLayer::Metal | MetalLayer::MetalAndVia => MetalLayer::Metal,
                    };
                    SiLayer::P
                }
                SiLayer::POnN => SiLayer::POnN,
            };
        }
        (ToolType::PType, true) => {
            cell.si = match cell.si {
                SiLayer::None | SiLayer::P => SiLayer::P,
                SiLayer::N => SiLayer::POnN,
                SiLayer::NOnP => SiLayer::NOnP,
                SiLayer::POnN => SiLayer::POnN,
            };
        }
        (ToolType::PType, false) => {
            cell.si = match cell.si {
                SiLayer::None | SiLayer::P => {
                    cell.metal = match cell.metal {
                        MetalLayer::None => MetalLayer::None,
                        MetalLayer::Metal | MetalLayer::MetalAndVia => MetalLayer::Metal,
                    };
                    SiLayer::None
                }
                SiLayer::N => SiLayer::N,
                SiLayer::NOnP => SiLayer::NOnP,
                SiLayer::POnN => {
                    cell.metal = match cell.metal {
                        MetalLayer::None => MetalLayer::None,
                        MetalLayer::Metal | MetalLayer::MetalAndVia => MetalLayer::Metal,
                    };
                    SiLayer::N
                }
            };
        }
        (ToolType::Metal, true) => {
            cell.metal = match cell.metal {
                MetalLayer::None | MetalLayer::Metal => MetalLayer::Metal,
                MetalLayer::MetalAndVia => MetalLayer::MetalAndVia,
            };
        }
        (ToolType::Metal, false) => {
            cell.metal = MetalLayer::None;
        }
        (ToolType::Via, true) => {
            cell.metal = match (cell.metal, cell.si) {
                (MetalLayer::Metal, si) if si != SiLayer::None => MetalLayer::MetalAndVia,
                (MetalLayer::MetalAndVia, _) => MetalLayer::MetalAndVia,
                _ => cell.metal,
            };
        }
        (ToolType::Via, false) => {
            cell.metal = match cell.metal {
                MetalLayer::Metal | MetalLayer::MetalAndVia => MetalLayer::Metal,
                MetalLayer::None => MetalLayer::None,
            };
        }
    };
}
