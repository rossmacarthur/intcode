import dynamic from "next/dynamic";
import { useState } from "react";
import { Listbox } from "@headlessui/react";
import { SelectorIcon } from "@heroicons/react/solid";

import * as intcode from "../hack/intcode";

import exampleHelloWorld from "../../../examples/hello-world.ints";
import exampleEcho from "../../../examples/echo.ints";
import exampleFunction from "../../../examples/function.ints";

const examples = [
  { file: "./examples/hello-world.ints", code: exampleHelloWorld },
  { file: "./examples/echo.ints", code: exampleEcho },
  { file: "./examples/function.ints", code: exampleFunction },
];

export default function Index() {
  const [example, setExample] = useState(examples[0]);
  const [computerState, setComputerState] = useState(intcode.State.COMPLETE);

  const onChange = (contents) => {
    const example = examples.find((example) => example.code === contents);
    if (example) {
      setExample(example);
    } else {
      setExample({ file: "" });
    }
  };

  const onRun = () => {
    intcode.assemble(setComputerState, example.code);
  };

  const onInput = (input) => {
    intcode.next(setComputerState, input);
  };

  return (
    <div class="h-screen w-screen bg-gray-lighter text-white flex flex-col">
      <Header
        example={example}
        setExample={setExample}
        computerState={computerState}
        onRun={onRun}
      />
      <Main
        contents={example.code}
        computerState={computerState}
        onChange={onChange}
        onInput={onInput}
      />
    </div>
  );
}

function Header({ example, setExample, computerState, onRun }) {
  const disabled = computerState != intcode.State.COMPLETE;

  return (
    <div class="flex flex-row m-2 gap-1">
      {disabled ? (
        <button
          class="w-24 p-2 rounded shadow-sm bg-purple transition-all
                   font-mono text-white"
          disabled
        >
          <span
            class="inline-flex align-middle h-5 w-5 rounded-full
                  border-white border-t-transparent border-2 animate-spin-fast"
          ></span>
        </button>
      ) : (
        <button
          class="w-24 p-2 rounded shadow-sm bg-blue hover:bg-purple transition-all
                font-mono text-white text-lg tracking-widest uppercase"
          onClick={onRun}
        >
          Run
        </button>
      )}
      <Examples selected={example} onChange={setExample} />
    </div>
  );
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

function Main({ contents, onChange, computerState, onInput }) {
  const onKeyUp = (e) => {
    if (e.key === "Enter") {
      const input = e.target.value;
      e.target.value = "";
      onInput(input);
    }
  };

  return (
    <div class="h-full grid grid-rows-5 md:grid-cols-7 gap-2 m-2 mt-0 font-mono">
      <div class="row-span-5 md:col-span-4">
        <Editor
          height="100%"
          width="100%"
          fontSize="1rem"
          name="editor"
          mode="assembly_intcode"
          theme="tomorrow_night_eighties"
          value={contents}
          onChange={onChange}
        />
      </div>

      <div class="row-span-2 md:col-span-3 h-56 md:h-full rounded-md shadow bg-gray-darker text-blue overflow-scroll">
        <div class="flex p-2 bg-gray-dark">Output</div>
      </div>

      <div class="row-span-3 md:col-span-3 flex flex-col h-56 md:h-full rounded-md shadow bg-gray-darker text-purple overflow-scroll">
        <div class="flex p-2 bg-gray-dark">Terminal</div>
        <div class="flex-grow"></div>
        <div class="p-2">
          {computerState == intcode.State.WAITING && (
            <input
              class="w-full block py-1 px-2 rounded focus:outline-blue bg-gray text-white"
              type="text"
              name="name"
              autoFocus
              autoComplete="off"
              onKeyUp={onKeyUp}
            />
          )}
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
