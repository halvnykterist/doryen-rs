in vec2 aVertexPosition;
in vec2 aTextureCoord;
out vec2 vTextureCoord;
void main(void) {
    gl_Position = vec4(aVertexPosition.xy, 0.0, 1.0);
    vTextureCoord = aTextureCoord;
}
