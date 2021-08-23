use imgui::Ui;
use skulpin::skia_safe::Canvas;

use super::Editor;
use crate::{
    tool_behaviors::ToolBehavior,
    tools::{tool_enum_to_tool, EditorEvent, Tool, ToolEnum},
    user_interface::Interface,
};

impl Editor {
    /// This is the function that powers the editor. Tools recieve events from the Editor and then use them to modify state.
    /// Adding new events is as simple as adding a new anonymous struct to EditorEvent and a call to this function in the appropriate
    /// place.Tools can then implement behavior for that event in their handle_event implementation.
    pub fn dispatch_editor_event(&mut self, i: &mut Interface, event: EditorEvent) {
        self.behavior_finished = false;
        if let Some(behavior) = self.tool_behaviors.pop() {
            let mut active_behavior = dyn_clone::clone_box(&*behavior);
            active_behavior.event(self, i, event);

            if !self.behavior_finished {
                self.tool_behaviors.push(active_behavior)
            }
        } else {
            let mut active_tool = dyn_clone::clone_box(&*self.active_tool);
            active_tool.event(self, i, event);
            self.active_tool = active_tool;
        }
    }

    pub fn dispatch_tool_draw(&self, i: &Interface, canvas: &mut Canvas) {
        for behavior in self.tool_behaviors.iter().rev() {
            behavior.draw(self, i, canvas);
        }
        self.active_tool.draw(self, i, canvas)
    }

    pub fn dispatch_tool_ui(&mut self, i: &mut Interface, ui: &mut Ui) {
        let mut active_tool = dyn_clone::clone_box(&*self.active_tool);
        active_tool.ui(self, i, ui);
        self.active_tool = active_tool;
    }

    /// Get the active tool by enum.
    pub fn get_tool(&self) -> ToolEnum {
        self.active_tool_enum
    }

    /// Get a mutable copy of the current tool as a boxed dyn Tool. This is used in event handling.
    pub fn get_tool_mut(&mut self) -> &mut Box<dyn Tool> {
        &mut self.active_tool
    }

    pub fn reset_tool(&mut self) {
        self.end_layer_modification();
        self.clear_behaviors();
        self.active_tool = tool_enum_to_tool(self.active_tool_enum);
    }

    /// Pops the current behavior off the behavior stack. ToolBehavior should call this when it has finished.
    pub fn pop_behavior(&mut self) {
        self.tool_behaviors.pop();

        // if the behavior is not on the stack when we call this we set this flag to tell the dispatch_event function
        // not to put the behavior back on the stack. The flag is cleared at the start of dispatch_event.
        self.behavior_finished = true;
    }

    /// Use this to push multiple behaviors on the stack for multi-stage editing. Behaviors should be pushed in the
    /// reverse order to their intended execution.
    pub fn push_behavior(&mut self, behavior: Box<dyn ToolBehavior>) {
        self.tool_behaviors.push(behavior);
    }

    /// This is the primary way of setting the editor's current behavior. You should call push_behavior only for
    /// multi-stage edits. This clears all behaviors, ends any current modifications, and puts the new behavior
    /// as the only behavior on the stack.
    pub fn set_behavior(&mut self, behavior: Box<dyn ToolBehavior>) {
        self.tool_behaviors = vec![]; // this is called so infequently this should be fine
        self.push_behavior(behavior);
    }

    pub fn clear_behaviors(&mut self) {
        self.tool_behaviors = vec![];
    }

    /// Set the active tool by enum. When adding your own tools make sure to add them to ToolEnum.
    pub fn set_tool(&mut self, tool: ToolEnum) {
        if self.active_tool_enum == tool {
            return;
        };

        self.end_layer_modification();
        self.active_tool_enum = tool;
        self.active_tool = tool_enum_to_tool(tool);
    }
}
