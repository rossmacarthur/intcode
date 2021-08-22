import init, { assemble, next } from './wasm/intcode_wasm.js';
import hello from 'bundle-text:../examples/hello-world.s'

let editor = ace.edit("editor");
let terminal = document.getElementById("terminal");
let button = document.getElementById("run");
let input = document.getElementById("input");

function feed(text) {
    try {
        let result = next(text);
        terminal.innerHTML += "<pre>" + result.output + "</pre>";
        if (result.state == "Complete") {
            terminal.innerHTML += "<pre>Done!</pre>";
            reset();
        } else {
            input.classList.remove("disabled");
            input.focus();
        }
    } catch (err) {
        terminal.innerHTML += "<pre>Error: " + err + "</pre>";
        reset();
    }
}

function reset() {
    input.value = "";
    button.onclick = run;
    input.classList.add("disabled");
    button.classList.remove("disabled");
}

function cancel() {
    terminal.innerHTML += "<pre>Canceled!</pre>";
    reset();
}

function run() {
    button.onclick = cancel;
    button.classList.add("disabled");
    let result = assemble(editor.getValue());
    terminal.innerHTML = "<pre>" + result.output + "</pre>";
    if (result.state == "Complete") {
        reset();
    } else {
        terminal.innerHTML += "<pre>Executing program...</pre>"
        feed(null);
    }
}

async function main() {
    await init();
    editor.setTheme("ace/theme/tomorrow_night_eighties");
    editor.session.setMode("ace/mode/assembly_x86");
    editor.session.setValue(hello);
    button.onclick = run;
    input.addEventListener("keypress", (e) => {
        if (e.key == 'Enter') {
            feed(input.value + "\n");
            input.value = "";
        }
    })
}

main();
