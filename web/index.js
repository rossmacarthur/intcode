import init, { assemble, next } from './wasm/intcode_wasm.js';
import hello from 'bundle-text:../examples/hello-world.s'

let editor = ace.edit("editor");
let terminal = document.getElementById("terminal");
let button = document.getElementById("run");
let input = document.getElementById("input");

function feed(text) {
    try {
        let result = next(text);
        terminal.innerHTML += "<log class='output'>" + result.output + "</log>";
        if (result.state == "Complete") {
            terminal.innerHTML += "<log>Done!</log>";
            reset();
        } else {
            input.classList.remove("disabled");
            input.focus();
        }
    } catch (err) {
        terminal.innerHTML += "<log class='output'><span class='hl-error'>error</span>: " + err + "</log>";
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
    terminal.innerHTML += "<log>Canceled!</log>";
    reset();
}

function run() {
    button.onclick = cancel;
    button.classList.add("disabled");
    let result = assemble(editor.getValue());
    if (result.state == "Complete") {
        terminal.innerHTML = "<log>Failed to compile program.</log>"
        terminal.innerHTML += "<log class='output'>" + result.output + "</log>";
        reset();
    } else {
        terminal.innerHTML = "<log>Successfully compiled program.</log>"
        terminal.innerHTML += "<log class='output'>" + result.output + "</log>";
        terminal.innerHTML += "<log>Executing program...</log>"
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
