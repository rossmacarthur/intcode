import init, { assemble, next } from './wasm/intcode_wasm.js';
import hello from 'bundle-text:../../examples/hello-world.s';
import { Mode } from './syntax.js';

let editor = ace.edit("editor");
let terminal = document.getElementById("terminal");
let button = document.getElementById("run");
let input = document.getElementById("input");

function delay(ms) {
    return x => {
      return new Promise(resolve => setTimeout(() => resolve(x), ms));
    };
}

function registerCopyToClipboard() {
    Array.from(document.getElementsByClassName('copy'))
        .forEach((e) => {
            let text = e.innerHTML;
            e.addEventListener('click', () => {
                navigator.clipboard.writeText(text)
                    .then(() => { e.innerHTML = "Copied intcode to clipboard!"; })
                    .then(delay(2000))
                    .then(() => { e.innerHTML = text; });
            });
        })
}

function feed(text) {
    try {
        let result = next(text);
        log(result.output, "output");
        if (result.state == "Complete") {
            log("Done!");
            reset();
        } else {
            input.classList.remove("disabled");
            input.focus();
        }
    } catch (err) {
        log("<span class='hl-error'>error</span>: " + err, "output");
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
    log("Canceled!");
    reset();
}

function log(innerHTML, classes) {
    let log = document.createElement('log');
    log.innerHTML = innerHTML;
    log.className = classes;
    terminal.appendChild(log);
}

function run() {
    button.onclick = cancel;
    button.classList.add("disabled");
    let result = assemble(editor.getValue());
    terminal.innerHTML = "";
    if (result.state == "Failed") {
        log("Failed to compile program.");
        log(result.output, "output")
        reset();
    } else {
        log("Successfully compiled program:");
        log(result.intcode, "output copy");
        log(result.output, "output");
        log("Executing program...");
        registerCopyToClipboard();
        feed(null);
    }
}

async function main() {
    await init();
    editor.setTheme("ace/theme/tomorrow_night_eighties");
    editor.session.setMode(new Mode());
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
