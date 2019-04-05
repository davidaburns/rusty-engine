extern crate glfw;
extern crate gl;

use self::glfw::{Context, Key, Action};
use self::gl::types::*;
use std::sync::mpsc::Receiver;
use std::ffi::CString;
use std::ptr;
use std::str;
use std::mem;
use std::os::raw::c_void;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

const VERT_SHADER_SRC: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    void main() {
        gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"#;

const FRAG_SHADER_SRC: &str = r#"
    #version 330 core
    out vec4 FragColor;
    void main() {
        FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    }
"#;

struct GLShader<'a> {
    shader_src: &'a str,
    shader_type: GLenum
}

impl GLShader<'_> {
    unsafe fn compile(&self) -> GLuint {
        let mut success = gl::FALSE as GLint;
        let mut info_log: Vec<u8> = Vec::with_capacity(512);
        let shader = gl::CreateShader(self.shader_type);
        let c_str_source = CString::new(self.shader_src.as_bytes()).unwrap();

        gl::ShaderSource(shader, 1, &c_str_source.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(shader, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::COMPILATION_FAILED\n{}", str::from_utf8(&info_log).unwrap());
        }

        shader
    }
}

struct OpenGLContext {
    shader_program: GLuint,
    vao: u32,
    vbo: u32,
    ebo: u32
}

impl OpenGLContext {
    unsafe fn initialize() -> Self {
        let mut shader_vec: Vec<GLShader> = Vec::new();
        shader_vec.push(GLShader {
            shader_src: VERT_SHADER_SRC,
            shader_type: gl::VERTEX_SHADER
        });

        shader_vec.push(GLShader {
            shader_src: FRAG_SHADER_SRC,
            shader_type: gl::FRAGMENT_SHADER
        });

        let compiled_shaders = OpenGLContext::compile_shaders_vec(shader_vec);
        let shader_program = OpenGLContext::link_shaders_vec(compiled_shaders);
        
        let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
        let verticies: [f32; 18] = [
             // first triangle
            -0.9, -0.5, 0.0,  // left
            -0.0, -0.5, 0.0,  // right
            -0.45, 0.5, 0.0,  // top
            // second triangle
            0.0, -0.5, 0.0,  // left
            0.9, -0.5, 0.0,  // right
            0.45, 0.5, 0.0   // top
        ];
        let indicies = [
            0, 1, 3,
            1, 2, 3
        ];

        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
            (verticies.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &verticies[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
            (indicies.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &indicies[0] as *const i32 as *const c_void,
            gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * mem::size_of::<GLfloat>() as GLsizei, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        Self {
            shader_program: shader_program,
            vao: vao,
            vbo: vbo,
            ebo: ebo
        }
    }

    unsafe fn compile_shaders_vec(shaders: Vec<GLShader>) -> Vec<GLuint> {
        let mut compiled_shaders: Vec<GLuint> = Vec::new();
        for shader in shaders {
            compiled_shaders.push(shader.compile());
        }

        compiled_shaders
    }

    unsafe fn link_shaders_vec(compiled_shaders: Vec<GLuint>) -> GLuint {
        let mut success = gl::FALSE as GLint;
        let mut info_log: Vec<u8> = Vec::with_capacity(512);
        let shader_program: GLuint = gl::CreateProgram();

        // Attatch the shaders
        for i in &compiled_shaders {
            gl::AttachShader(shader_program, *i);
        }

        gl::LinkProgram(shader_program);
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(shader_program, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", str::from_utf8(&info_log).unwrap());
        }

        // Delete the shaders
        for i in &compiled_shaders {
            gl::DeleteShader(*i);
        }

        // Return the final linked shader program
        shader_program
    }
}

struct OpenGLApplication<'a> {
    screen_width: u32,
    screen_height: u32,
    title: &'a str,
    glfw: glfw::Glfw,
    window: glfw::Window,
    window_events: Receiver<(f64, glfw::WindowEvent)>,
    opengl_context: OpenGLContext
}

impl<'a> OpenGLApplication<'a> {
    fn initialize(width: u32, height: u32, title: &'a str) -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

        let (mut window, events) = glfw.create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW Window");

        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        let opengl_context = unsafe { OpenGLContext::initialize() };
        Self {
            screen_width: width,
            screen_height: height,
            title: title,
            glfw: glfw,
            window: window,
            window_events: events,
            opengl_context: opengl_context
        }
    }

    fn run(&mut self) {
        while !self.window.should_close() {
            self.process_window_events();

            unsafe {
                gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::UseProgram(self.opengl_context.shader_program);
                gl::BindVertexArray(self.opengl_context.vao);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            }

            self.window.swap_buffers();
            self.glfw.poll_events();
        }
    }

    fn process_window_events(&mut self) {
        for (_, event) in glfw::flush_messages(&self.window_events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                        self.screen_width = width as u32;
                        self.screen_height = height as u32;
                    } 
                },
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true);
                },
                _ => {}
            }
        }
    }
}

pub fn main() {
    let mut app: OpenGLApplication = OpenGLApplication::initialize(SCREEN_WIDTH, SCREEN_HEIGHT, "OpenGL Learning");
    app.run();
}
