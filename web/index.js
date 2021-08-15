export function editor_init() {
    editor = ace.edit("editor");
    editor.setTheme("ace/theme/tomorrow_night_eighties");
    editor.session.setMode("ace/mode/assembly_x86");
    return editor;
}

export function editor_text(editor) {
    return editor.getValue();
}
