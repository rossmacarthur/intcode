import dynamic from "next/dynamic";
import { useState } from "react";
import { Listbox, } from "@headlessui/react";
import { SelectorIcon } from "@heroicons/react/solid";

import exampleHelloWorld from "../../../examples/hello-world.ints";
import exampleEcho from "../../../examples/echo.ints";
import exampleFunction from "../../../examples/function.ints";

const examples = [
  { file: "./examples/hello-world.ints", code: exampleHelloWorld },
  { file: "./examples/echo.ints", code: exampleEcho },
  { file: "./examples/function.ints", code: exampleFunction },
]

export default function Index() {
  const [example, setExample] = useState(examples[0]);

  const onChange = (contents) => {
    const example = examples.find(example => example.code === contents);
    if (example) {
      setExample(example);
    } else {
      setExample({ file: "" })
    }
  }

  return (
    <div class="h-screen w-screen bg-gray-lighter text-white flex flex-col">
      <Header example={example} setExample={setExample}/>
      <Main contents={example.code} onChange={onChange}/>
    </div>
  );
}

function Header({ example, setExample }) {
  return (
    <div class="flex flex-row m-2 gap-1">
      <button class="w-24 p-2 rounded shadow-sm bg-blue hover:bg-purple transition-all
                     font-mono text-white text-lg tracking-widest uppercase">
        Run
      </button>

      <Examples selected={example} onChange={setExample} />
    </div>
  );
}

function Examples({ selected, onChange }) {
  return (
    <div class="w-72 font-mono text-sm italic cursor-pointer z-10">
      <Listbox value={selected} onChange={onChange}>
        <Listbox.Button class="inline-flex w-72 px-4 py-3 rounded shadow-sm bg-white hover:bg-gray-lightest
                               transition-all text-gray hover:text-black">
            <span class="flex-grow text-left">{selected.file}</span>
            <SelectorIcon class="-mr-2 ml-2 h-5 w-5" aria-hidden="true" />
          </Listbox.Button>

          <Listbox.Options class="absolute w-72 mt-1 max-h-60 overflow-auto rounded shadow-lg">
            {examples.map((example, idx) => (
              <Listbox.Option class="pl-4 pr-8 py-2 bg-white hover:bg-purple text-gray hover:text-white" key={idx} value={example}>
                {example.file}
              </Listbox.Option>
            ))}
          </Listbox.Options>
      </Listbox>
    </div>
  );
}

function Main({ contents, onChange }) {
  return (
    <div class="h-full grid grid-rows-5 md:grid-flow-col gap-2 m-2 mt-0 font-mono">
      <div class="row-span-5">
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

      <div class="row-span-2 h-56 md:h-full p-2 rounded shadow bg-gray-darker text-blue overflow-scroll">
        Output
      </div>

      <div class="row-span-3 h-56 md:h-full p-2 rounded shadow bg-gray-darker text-purple overflow-scroll">
        Terminal
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
