Reasonably optimized Minecraft-like voxel game written in Rust<br>
Uses a custom engine that interfaces OpenGL and Vulkan<br>
Featuring custom, safe RAII wrappers over raw api handles

WASDQE + mouse for movement

Dependencies<br>
The installation will attempt to build GLFW from source, so you will need CMake<br>
On linux systems you will also need Wayland or X11 development packages respectively<br>

Installation<br>
Clone the repository and run cargo run --release<br>
Debug builds will attempt to load Vulkan validation layers, create a debug context for OpenGL, and force an X11 backend on linux systems



<img width="1281" height="716" alt="image" src="https://github.com/user-attachments/assets/545b1bf8-c952-4553-9764-3209c11707ba" />
