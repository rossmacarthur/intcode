import dynamic from "next/dynamic";
import { useEffect, createRef, useState } from "react";
import { Listbox } from "@headlessui/react";
import { SelectorIcon } from "@heroicons/react/solid";

import exampleHelloWorld from "../../../examples/hello-world.ints";
import exampleEcho from "../../../examples/echo.ints";
import exampleFunction from "../../../examples/function.ints";

export const State = {
  WAITING: 1,
  COMPLETE: 2,
};
Object.freeze(State);

const examples = [
  { file: "./examples/hello-world.ints", code: exampleHelloWorld },
  { file: "./examples/echo.ints", code: exampleEcho },
  { file: "./examples/function.ints", code: exampleFunction },
];

export default function Index() {
  // The editor instance TODO: surely we can use useRef somehow???
  const [editor, setEditor] = useState(null);
  // The intcode WASM module instance
  const [wasm, setWasm] = useState(null);
  // The currently selected example
  const [example, setExample] = useState(examples[0]);
  // The state of the computer
  const [state, setState] = useState(State.COMPLETE);
  // The output panel
  const [output, setOutput] = useState({
    compiledIntcode: "",
    compilerOutput: "",
    programOutput: [],
  });

  // Loads the WASM module
  useEffect(() => {
    const f = async () => {
      const instance = await import("../../wasm/pkg");
      instance.init();
      setWasm(instance);
    };
    f();
  });

  // Called when the editor is loaded
  const onLoad = (editor) => {
    setEditor(editor);
  };

  // Called when the contents of the editor change
  const onEdited = (contents) => {
    const example = examples.find((example) => example.code === contents);
    if (example) {
      setExample(example);
    } else {
      setExample({ file: "" });
    }
  };

  // Called when the user clicks the "Run" button
  const onRun = () => {
    setState(State.WAITING);

    const result = wasm.assemble(editor.getValue());
    if (result.state == "Failed") {
      setOutput({
        compiledIntcode: result.intcode,
        compilerOutput: result.output,
      });
      setState(State.COMPLETE);
      return;
    }

    const result2 = wasm.next(null);
    setOutput({
      compiledIntcode: result.intcode,
      compilerOutput: result.output,
      programOutput: [result2.output],
    });
    if (result2.state == "Complete") {
      setState(State.COMPLETE);
    }
    // Machine wants input, leave in WAITING state
  };

  // Cancelled when the user clicks the "Cancel" button
  const onCancel = () => {
    setState(State.COMPLETE);
  };

  // Called when the user inputs a value
  const onInput = (input) => {
    const result = wasm.next(input + "\n");
    setOutput({
      ...output,
      programOutput: output.programOutput.concat(result.output),
    });
    if (result.state == "Complete") {
      setState(State.COMPLETE);
    }
    // Machine still wants input, leave in WAITING state
  };

  return (
    <div class="h-screen w-screen flex flex-col bg-gray-lighter text-white font-mono">
      <Header
        example={example}
        setExample={(e) => {
          setExample(e);
          setState(State.COMPLETE);
        }}
        state={state}
        onRun={onRun}
        onCancel={onCancel}
      />
      <Main
        contents={example.code}
        state={state}
        onLoad={onLoad}
        onEdited={onEdited}
        onInput={onInput}
        output={output}
      />
    </div>
  );
}

function Header({ example, setExample, state, onRun, onCancel }) {
  return (
    <div class="flex flex-row p-2 pb-0 gap-1">
      <Button state={state} onRun={onRun} onCancel={onCancel} />
      <Examples selected={example} onChange={setExample} />
    </div>
  );
}

function Button({ state, onRun, onCancel }) {
  switch (state) {
    case State.WAITING:
      return (
        <button
          class="group w-28 p-2 rounded shadow-sm bg-purple transition-all
              font-mono text-white text-md tracking-widest uppercase"
          onClick={onCancel}
        >
          <span
            class="group-hover:hidden inline-flex align-middle h-5 w-5 rounded-full
                border-white border-t-transparent border-2 animate-spin-fast transition-all"
          ></span>
          <span class="hidden group-hover:inline transition-all">Cancel</span>
        </button>
      );
    case State.COMPLETE:
      return (
        <button
          class="w-28 p-2 rounded shadow-sm bg-blue hover:bg-purple transition-all
              font-mono text-white text-md tracking-widest uppercase transition-all"
          onClick={onRun}
        >
          <span>Run</span>
        </button>
      );
  }
}

function Examples({ selected, onChange }) {
  return (
    <div class="w-72 font-mono text-sm italic cursor-pointer z-10">
      <Listbox value={selected} onChange={onChange}>
        <Listbox.Button
          class="inline-flex w-72 px-4 py-3 rounded shadow-sm bg-white hover:bg-gray-lightest
                               transition-all text-gray hover:text-black"
        >
          <span class="flex-grow text-left">{selected.file}</span>
          <SelectorIcon class="-mr-2 ml-2 h-5 w-5" aria-hidden="true" />
        </Listbox.Button>

        <Listbox.Options class="absolute w-72 mt-1 max-h-60 overflow-auto rounded shadow-lg">
          {examples.map((example, idx) => (
            <Listbox.Option
              class="pl-4 pr-8 py-2 bg-white hover:bg-purple text-gray hover:text-white"
              key={idx}
              value={example}
            >
              {example.file}
            </Listbox.Option>
          ))}
        </Listbox.Options>
      </Listbox>
    </div>
  );
}

function Main(props) {
  const onKeyUp = (e) => {
    if (e.key === "Enter") {
      const input = e.target.value;
      e.target.value = "";
      props.onInput(input);
    }
  };

  return (
    <div class="grid auto md:grid-cols-7 gap-2 p-2 font-mono">
      <div class="flex flex-col col-span-4 auto">
        <div class="flex-none p-2 rounded-t-md bg-gray-dark text-gray-light text-sm uppercase tracking-widest">
          Editor
        </div>
        <div class="flex-grow">
          <Editor
            height="100%"
            width="100%"
            fontSize="1rem"
            name="editor"
            mode="assembly_intcode"
            theme="tomorrow_night_eighties"
            onLoad={props.onLoad}
            value={props.contents}
            onChange={props.onEdited}
          />
        </div>
      </div>

      <div class="flex flex-col col-span-3 auto">
        <div class="flex-none p-2 rounded-t-md bg-gray-dark text-gray-light text-sm uppercase tracking-widest">
          Output
        </div>

        <div class="flex-grow p-2 rounded-b-md bg-gray-darker text-gray-light text-md overflow-scroll">
          <div class="flex flex-col gap-4">
            {props.output.compiledIntcode && (
              <div>
                <p class="text-green">Compiled intcode:</p>
                <p class="break-all">{props.output.compiledIntcode}</p>
              </div>
            )}

            {props.output.compilerOutput && (
              <div>
                <pre
                  dangerouslySetInnerHTML={{
                    __html: props.output.compilerOutput,
                  }}
                ></pre>
              </div>
            )}

            {props.output.compiledIntcode && (
              <div>
                <p class="text-green">Program output:</p>
                {props.output.programOutput.map((item) => (
                  <pre>{item}</pre>
                ))}
              </div>
            )}

            {props.state == State.WAITING && (
              <div>
                <p class="text-yellow pb-2">Program input:</p>
                <input
                  class="w-full block py-1 px-2 rounded-sm focus:outline-blue bg-gray text-white"
                  type="text"
                  name="name"
                  autoFocus
                  autoComplete="off"
                  onKeyUp={onKeyUp}
                />
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

const Editor = dynamic(
  async () => {
    const ace = await import("react-ace");
    require("../hack/syntax.js");
    require("ace-builds/src-noconflict/theme-tomorrow_night_eighties");
    return ace;
  },
  {
    loading: () => <div class="text-black">Loading editor...</div>,
    ssr: false,
  }
);
