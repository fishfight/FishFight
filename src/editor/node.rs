use macroquad::experimental::scene::{Node, RefMut};

use super::{Editor, EditorInputScheme};

/// Used to interface with macroquad. Necessary because using `node: RefMut<Self>` really limits
/// what can be done in regards to borrowing.
pub struct EditorNode {
    editor: Editor,

    accept_mouse_input: bool,
    accept_kb_input: bool,
    input_scheme: EditorInputScheme,
}

impl EditorNode {
    pub fn new(editor: Editor, input_scheme: EditorInputScheme) -> Self {
        Self {
            editor,
            accept_mouse_input: true,
            accept_kb_input: true,
            input_scheme,
        }
    }
}

impl Node for EditorNode {
    fn fixed_update(mut node: RefMut<Self>) {
        let input = node
            .input_scheme
            .collect_input(node.accept_kb_input, node.accept_mouse_input);

        node.editor.process_input(&input);
    }

    fn draw(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        node.editor.draw_level();

        egui_macroquad::ui(|egui_ctx| {
            node.editor.ui(egui_ctx);
            //node.accept_mouse_input = !egui_ctx.wants_pointer_input();
            node.accept_kb_input = !egui_ctx.wants_keyboard_input();
        });

        egui_macroquad::draw();
    }
}
