// Sizes
pub static OUTLINE_STROKE_THICKNESS: f32 = 1.5 * PEN_SIZE;
pub static GUIDELINE_THICKNESS: f32 = OUTLINE_STROKE_THICKNESS;
pub static POINT_STROKE_THICKNESS: f32 = 3. * PEN_SIZE;
pub static DIRECTION_STROKE_THICKNESS: f32 = 2. * PEN_SIZE;
pub static HANDLE_STROKE_THICKNESS: f32 = 2.5 * PEN_SIZE;
pub static POINT_RADIUS: f32 = 5. * PEN_SIZE;
pub static HANDLE_RADIUS: f32 = 2.5 * PEN_SIZE;
pub static HANDLEBAR_THICKNESS: f32 = 3. * PEN_SIZE;

// Colors
pub static OUTLINE_FILL: u32 = 0xaa_000000;
pub static OUTLINE_STROKE: u32 = 0xff_000000;
pub static POINT_SQUARE_FILL: u32 = 0xff_6ae755;
pub static POINT_SQUARE_STROKE: u32 = 0xff_208e53;
// One and two refers to number of Bezier handles.
pub static POINT_ONE_FILL: u32 = 0xff_44cf8c;
pub static POINT_ONE_STROKE: u32 = 0xff_1d8a84;
pub static POINT_TWO_FILL: u32 = 0xff_579aff;
pub static POINT_TWO_STROKE: u32 = 0xff_4428ec;
pub static HANDLE_FILL: u32 = 0xff_ff57ee;
pub static HANDLE_STROKE: u32 = 0xff_b928ec;
pub static HANDLEBAR_STROKE: u32 = 0xff_7e28ec;

pub static SELECTED_FILL: u32 = 0xff_ffed50;
pub static SELECTED_STROKE: u32 = 0xff_ffa115;

pub static LBEARING_STROKE: u32 = 0xff_7d7d7d;
pub static RBEARING_STROKE: u32 = LBEARING_STROKE;

// Math
pub const PI: f32 = std::f32::consts::PI;
pub const DEGREES_IN_RADIANS: f32 = PI / 180.0;

// Misc.
pub const PEN_SIZE: f32 = 1.0;
