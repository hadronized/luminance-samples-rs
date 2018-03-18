//! This program shows how to use the `TessRender::one_slice` function to use a single tessellations
//! to render several objects.
//!
//! Press <space> to switch between direct tessellation and indexed tessellation.
extern crate luminance;
extern crate luminance_glfw;

use std::thread; // used to sleep
use std::time::Duration;
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::{RenderState, pipeline};
use luminance::shader::program::Program;
use luminance::tess::{Mode, Tess, TessRender};
use luminance_glfw::{Action, Device, GLFWDevice, Key, WindowDim, WindowEvent, WindowOpt};

#[derive(Copy, Clone, Debug)]
enum Demo {
  Direct,
  Indexed
}

impl Demo {
  fn toggle(self) -> Self {
    match self {
      Demo::Direct => Demo::Indexed,
      Demo::Indexed => Demo::Direct,
    }
  }
}

fn main() {
  let mut dev =
    GLFWDevice::new(WindowDim::Windowed(800, 600),
                    "luminance samples",
                    WindowOpt::default()
                    ).expect("device creation");

  let (program, _) = Program::<Vertex, (), ()>::from_strings(None, &VS, None, &FS).expect("program");
  let direct_triangles = Tess::new(Mode::TriangleFan, &TRI_VERTICES[..], None);
  let indexed_triangles = Tess::new(Mode::TriangleFan, &TRI_VERTICES[..], &TRI_INDEXES[..]);

  let mut demo = Demo::Direct;
  println!("now rendering {:?}", demo);

  'app: loop {
    for event in dev.events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        WindowEvent::Key(Key::Space, _, Action::Release, _) => {
          demo = demo.toggle();
          println!("now rendering {:?}", demo);
        }

        _ => ()
      }
    }

    let size = dev.size();

    dev.draw(|| {
      pipeline(&Framebuffer::default(size), [0., 0., 0., 0.], |shd_gate| {
        shd_gate.shade(&program, |rdr_gate, _| {
          rdr_gate.render(RenderState::default(), |tess_gate| {
            match demo {
              Demo::Direct => {
                tess_gate.render(TessRender::one_slice(&direct_triangles, 3, 3));
                tess_gate.render(TessRender::one_slice(&direct_triangles, 0, 3));
              }

              Demo::Indexed => {
                tess_gate.render(TessRender::one_slice(&indexed_triangles, 0, 3));
                tess_gate.render(TessRender::one_slice(&indexed_triangles, 3, 3));
              }
            }
          });
        });
      });
    });

    thread::sleep(Duration::from_millis(100));
  }
}

const VS: &'static str = "\
layout (location = 0) in vec2 co;\n\
layout (location = 1) in vec3 color;\n\

out vec3 v_color;\n\

void main() {\n\
  gl_Position = vec4(co, 0., 1.);\n\
  v_color = color;\n\
}";

const FS: &'static str = "\
in vec3 v_color;\n\

out vec4 frag;\n\

void main() {\n\
  frag = vec4(v_color, 1.);\n\
  frag = pow(frag, vec4(1./2.2));\n\
}";

type Vertex = ([f32; 2], [f32; 3]);

const TRI_VERTICES: [Vertex; 6] = [
  // first triangle – an RGB one
  ([ 0.5, -0.5], [0., 1., 0.]),
  ([ 0.0,  0.5], [0., 0., 1.]),
  ([-0.5, -0.5], [1., 0., 0.]),
  // second triangle, a purple one, positioned differently
  ([-0.5,  0.5], [1., 0.2, 1.]),
  ([ 0.0, -0.5], [0.2, 1., 1.]),
  ([ 0.5,  0.5], [0.2, 0.2, 1.])
];

const TRI_INDEXES: [u32; 6] = [
  // first triangle – an RGB one
  0, 1, 2,
  // second triangle, a purple one, positioned differently
  3, 4, 5
];
