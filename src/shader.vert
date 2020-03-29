#version 100

// shader validator: http://shdr.bkcore.com/
uniform vec2 viewSize;
// layout(location = 0) in vec2 vertex;
// layout(location = 1) in vec2 tcoord;
attribute vec2 vertex;
attribute vec2 tcoord;
varying lowp vec2 ftcoord;
varying highp vec2 fpos;

void main(void) {
    ftcoord = tcoord;
    fpos = vertex;
    gl_Position = vec4(2.0 * vertex.x / viewSize.x - 1.0, 1.0 - 2.0 * vertex.y / viewSize.y, 0, 1);
}
