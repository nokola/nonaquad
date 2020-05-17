#version 100

precision highp float;

uniform mat4 scissorMat;
uniform mat4 paintMat;
uniform vec4 innerCol;
uniform vec4 outerCol;
uniform vec2 scissorExt;
uniform vec2 scissorScale;
uniform vec2 extent;
uniform float radius;
uniform float feather;
uniform float strokeMult;
uniform float strokeThr;

// texture type:
// 0: RGBA, premultiplied
// 1: RGBA, not premultiplied
// 2: Alpha texture, alpha value is stored in .a (miniquad always stores in .a for alpha textures)
uniform int texType;
uniform int type;

uniform sampler2D tex;
varying vec2 ftcoord;
varying vec2 fpos;
//out vec4 outColor;

float sdroundrect(vec2 pt, vec2 ext, float rad) {
    vec2 ext2 = ext - vec2(rad,rad);
    vec2 d = abs(pt) - ext2;
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0)) - rad;
}

float scissorMask(vec2 p) {
    vec2 sc = (abs((mat3(scissorMat) * vec3(p, 1.0)).xy) - scissorExt);
    sc = vec2(0.5,0.5) - sc * scissorScale;
    return clamp(sc.x, 0.0, 1.0) * clamp(sc.y, 0.0, 1.0);
}

float strokeMask() {
    return min(1.0, (1.0 - abs(ftcoord.x * 2.0 - 1.0)) * strokeMult) * min(1.0, ftcoord.y);
}

void main(void) {
    vec4 result;
    float scissor = scissorMask(fpos);
    float strokeAlpha = strokeMask();
    if (strokeAlpha < strokeThr) discard;

    if (type == 0) {
        // Gradient
        vec2 pt = (mat3(paintMat) * vec3(fpos,1.0)).xy;
        float d = clamp((sdroundrect(pt, extent, radius) + feather * 0.5) / feather, 0.0, 1.0);
        vec4 color = mix(innerCol, outerCol, d);
        color *= strokeAlpha * scissor;
        result = color;
    } else if (type == 1) {
        // Image
        vec2 pt = (mat3(paintMat) * vec3(fpos, 1.0)).xy / extent;
        vec4 color = texture2D(tex, pt);
        if (texType == 1) color = vec4(color.xyz * color.w, color.w); // premultiply non-premultiplied texture
        if (texType == 2) color = vec4(color.a); // alpha texture
        color *= innerCol;
        color *= strokeAlpha * scissor;
        result = color;
    } else if (type == 2) {
        // Stencil fill
        result = vec4(1, 1, 1, 1);
    } else if (type == 3) {
        // Textured tris
        vec4 color = texture2D(tex, ftcoord);
        if (texType == 1) color = vec4(color.xyz * color.w, color.w); // premultiply non-premultiplied texture
        if (texType == 2) color = vec4(color.a); // alpha texture
        color *= scissor;
        result = color * innerCol;
    }

    gl_FragColor = result;
    // gl_FragColor = vec4(1,0,0,1);
}
