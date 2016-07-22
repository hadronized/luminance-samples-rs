extern crate luminance;
extern crate luminance_gl;
extern crate gl;
extern crate glfw;

use glfw::{Action, Context, Key};
use luminance::{FragmentShader, Mode, VertexShader, Vertex};
use luminance_gl::gl33::{Framebuffer, Pipeline, Program, RenderCommand, ShadingCommand, Stage,
                         Tessellation};
use std::os::raw::c_void;
use std::thread;
use std::time::Duration;

fn main() {
  let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

  // OpenGL hint
  glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
  glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
  glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));

  let (mut window, events) = glfw.create_window(800, 600, "luminance app", glfw::WindowMode::Windowed).expect("Failed to create GLFW window.");

  window.make_current();
  window.set_key_polling(true);

  gl::load_with(|s| window.get_proc_address(s) as *const c_void);

  let vs = Stage::<VertexShader>::new(VS_SRC).unwrap();
  let fs = Stage::<FragmentShader>::new(FS_SRC).unwrap();
  let program = Program::new(None, &vs, None, &fs, |_| { Ok(()) }).unwrap();

  let triangle = Tessellation::new(Mode::Triangle, &TRI_VERTICES, None);

  let back_buffer = Framebuffer::default((800, 600));
  let shading_cmd = ShadingCommand::new(&program, |_|{}, vec![
    RenderCommand::new(None, true, |_|{}, &triangle, 1, None)
  ]);
  let pipeline = Pipeline::new(&back_buffer, [0., 0., 0., 1.], vec![&shading_cmd]);

  let vf = <([f32; 2], [f32; 3]) as Vertex>::vertex_format();
  println!("{:?}", vf);

  'app_loop: while !window.should_close() {
    glfw.poll_events();

    for (_, event) in glfw::flush_messages(&events) {
      match event {
        glfw::WindowEvent::Key(key, _, action, _) if key == Key::Escape && action == Action::Release => {
          break 'app_loop;
        },
        _ => {}
      }
    }

    pipeline.run();

    window.swap_buffers();
    thread::sleep(Duration::from_millis(100));
  }
}

const VS_SRC: &'static str = "\
layout (location = 0) in vec2 co;\n\
layout (location = 1) in vec3 color;\n\

out vec3 v_color;\n\

void main() {\n\
  gl_Position = vec4(co, 0., 1.);\n\
  v_color = color;\n\
}";

const FS_SRC: &'static str = "\
in vec3 v_color;\n\

out vec4 frag;\n\

void main() {\n\
  frag = vec4(v_color, 1.);\n\
  frag = pow(frag, vec4(1./2.2));\n\
}";

const TRI_VERTICES: [([f32; 2], [f32; 3]); 3] = [
  ([-0.5, -0.5], [1., 0., 0.]),
  ([ 0.5, -0.5], [0., 1., 0.]),
  ([ 0.0,  0.5], [0., 0., 1.])
];
