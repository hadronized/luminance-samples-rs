extern crate luminance; // core library
extern crate luminance_gl; // OpenGL backend
extern crate gl; // needed to create an OpenGL context
extern crate glfw; // window stuff

use glfw::{Action, Context, Key};
// Currently, anything that doesn’t require backend or several implementation (i.e. which can be
// shared between backends) is found in the luminance crate. In our case, we need only need the
// tessellation mode.
use luminance::Mode;
// We use the OpenGL 3.3 backend as it’s the only one available currently. All the types we’re using
// from this crate are also defined in the luminance crate. However, keep in mind that luminance’s
// types are a bit more generic – as they accept an extra template argument for the backend type.
// The types defined in backends are just aliases to the luminance’s ones, substituting the backend
// type variable with the correct one (in our case, GL33).
use luminance_gl::gl33::{Framebuffer, Pipeline, Program, RenderCommand, ShadingCommand, Stage,
                         Tessellation};
use std::os::raw::c_void;
use std::thread;
use std::time::Duration;

fn main() {
  // Initialize GLFW.
  let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

  // OpenGL hints. Those are mandatory, because they have to match with luminance_gl::gl33.
  glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
  glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
  glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));

  // Create a non-fullscreen window.
  let (mut window, events) = glfw.create_window(800, 600, "luminance app", glfw::WindowMode::Windowed).expect("Failed to create GLFW window.");
  // This line is used to bring the OpenGL context and attach it to that window. You can have several
  // OpenGL contexts around in theory but keep in mind it wasn’t tested (yet?) with luminance. If
  // you need this property, please let me know.
  window.make_current();

  // Trick part of the new version of GLFW: you now have to state which events you want to listen
  // to! Because this is just a sample program, we’ll just listen to the key state changes so that
  // we can capture a press on escape.
  window.set_key_polling(true);

  // Provide the pointer address loader for OpenGL functions.
  gl::load_with(|s| window.get_proc_address(s) as *const c_void);

  // A vertex shader is a piece of code that runs on the GPU for each vertex you stream to.
  //
  // It is required to have a vertex shader to build a shader program.
  let vs = Stage::new(VS_SRC).unwrap();
  // A fragment shader is a piece of code that runs on the GPU for each covered pixel after
  // rasterization. When you render tessellation, it always covers a part of the screen (the
  // whole, a part of it or nothing). For each pixel covered by that tessellation (if any), the
  // fragment shader is executed and pixel data are flowing into it.
  //
  // It is required to have a fragment shader to build a shader program.
  let fs = Stage::new(FS_SRC).unwrap();
  // A shader program represents all stages used to transform and rasterize tessellation. Here,
  // the None value stand for, respectively, the tessellation shader stages pair and the geometry
  // shader. You’d have used Some(…) if you wanted one of those. More information in the luminance
  // documentation.
  //
  // The last argument is a closure used to build a uniform interface. That interface is attached to
  // the shader program and strongly types it. It is used to pass values to your shader stages so
  // that you can customize and make the stages act like functions. In our case, because we don’t
  // actually need that, we just pass a closure that doesn’t read its argument and returns unit.
  // The type of `program` is then:
  //
  //     program: Program<()>
  let program = Program::new(None, &vs, None, &fs, |_| { Ok(()) }).unwrap();

  // We’ll be rendering a simple triangle as tessellation. We ask luminance to use the
  // Mode::Triangle mode for the render and we pass the vertices in. The last argument (None)
  // represents an optional slice of indexes. If you pass any, you enable indexed rendering.
  let triangle = Tessellation::new(Mode::Triangle, &TRI_VERTICES, None);

  // Renders are performed into framebuffers. The default framebuffer is the backbuffer of the
  // window. You’re strongly advised to use the same dimension as the one you used when you created
  // the window.
  let back_buffer = Framebuffer::default((800, 600));
  // A shading command represents data sent to the GPU to shade things. In our case, we just need to
  // shade our triangle with the shader program we created just above. The second argument gives you
  // access to the uniform interface, but remember, ours is just (), so we can just pass a no-op
  // closure. The last argument is a list of things we want to shade. Here, we only shade a
  // triangle through a render command. The first argument is the blending configuration. None means
  // no blending. The second one is whether we should perform a depth test while executing that
  // render command. The third one is exactly the same as the second argument of the shading
  // command – we pass in a no-op closure. The fourth argument is the tessellation to render; our
  // triangle. The next argument is the number of instances we want. We just want a triangle, so we
  // set it to 1. Finally, the latest argument is the size of the rasterized points and lines.
  // Because we’re not rendering points nor lines, we set it to None.
  let shading_cmd = ShadingCommand::new(&program, |_|{}, vec![
    RenderCommand::new(None, true, |_|{}, &triangle, 1, None)
  ]);
  // A pipeline gathers several shading commands, executes them and outputs the result into a given
  // framebuffer. The first argument is the framebuffer to render into, the second one is the color
  // to clear the framebuffer with before rendering and the last argument is the list of shading
  // commands.
  let pipeline = Pipeline::new(&back_buffer, [0., 0., 0., 1.], vec![&shading_cmd]);

  // Let’s just loop until the window closes.
  while !window.should_close() {
    glfw.poll_events();

    for (_, event) in glfw::flush_messages(&events) {
      match event {
        glfw::WindowEvent::Key(key, _, action, _) if key == Key::Escape && action == Action::Release => {
          window.set_should_close(true);
        },
        _ => {}
      }
    }

    // Run our pipeline. Here, our pipeline is static. That means it’s created once (before the
    // loop) and kept around until the program dies. Keep in mind that shading commands, render
    // commands and pipelines can be created on the fly, for each frame. So you can have dynamic
    // pipelines if you want to. However, for the sake of this sample, such a configuration is not
    // required. Keep it simple and stupid. That’s the most important thing in life.
    pipeline.run();

    // Swap the back buffer with the front buffer, making your render appear on screen.
    window.swap_buffers();
    // This is a bit brutal. In a good and correct production application, you’d compute the time it
    // took to go through an iteration of the loop and sleep until the required amount of time to
    // meet the whished FPS. Some people don’t actually sleep and let the while goes crazy and rely
    // on the fact your graphic driver will block screen updates if you try to render too quickly.
    // That is because it takes time to the screen to update its pixels.
    //
    // In our case, we just want the program to sleep the most. Also, because we just render a
    // non-moving triangle, we could have just made a single render before the loop.
    thread::sleep(Duration::from_millis(100));
  }
}

// The vertex shader source code. If you’re not comfortable with it, you should have a look to
// GLSL tutorials and the OpenGL specifications.
const VS_SRC: &'static str = "\
layout (location = 0) in vec2 co;\n\
layout (location = 1) in vec3 color;\n\

out vec3 v_color;\n\

void main() {\n\
  gl_Position = vec4(co, 0., 1.);\n\
  v_color = color;\n\
}";

// The fragment shader source code.
const FS_SRC: &'static str = "\
in vec3 v_color;\n\

out vec4 frag;\n\

void main() {\n\
  frag = vec4(v_color, 1.);\n\
  frag = pow(frag, vec4(1./2.2));\n\
}";

// The vertices used to represent the triangle. As you can see, it’s an array of three elements: the
// three vertices of a triangle. If you look at the type, you can see that the vertex type is just
// a pair: ([f32; 2], [f32; 3]). You read it as (position, color). Quite easy, right? luminance
// accepts a shit load of types for representing vertices – you can even create your own. The single
// pitfall with that is the fact that the vertex type has to match with the input statements you
// have in your shaders. Be careful then.
const TRI_VERTICES: [([f32; 2], [f32; 3]); 3] = [
  ([-0.5, -0.5], [1., 0., 0.]),
  ([ 0.5, -0.5], [0., 1., 0.]),
  ([ 0.0,  0.5], [0., 0., 1.])
];
