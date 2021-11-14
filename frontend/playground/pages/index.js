import dynamic from "next/dynamic";

const Editor = dynamic(
  async () => {
    const ace = await import("react-ace");
    require("../hack/syntax.js");
    require("ace-builds/src-noconflict/theme-tomorrow_night_eighties");
    return ace;
  },
  {
    loading: () => <>Loading...</>,
    ssr: false,
  }
);

function Home() {
  return (
    <div class="h-screen w-screen bg-gray-light text-white font-mono flex flex-col">
      <div class="m-2">
        <button class="bg-blue hover:bg-purple transition-colors
                       text-white text-base tracking-widest uppercase rounded p-1 w-24">Run</button>
      </div>

      <div class="h-full grid grid-rows-5 md:grid-flow-col gap-2 m-2 mt-0">
        <div class="row-span-5">
          <Editor
            height="100%"
            width="100%"
            fontSize="1rem"
            name="editor"
            mode="assembly_intcode"
            theme="tomorrow_night_eighties"
          />
        </div>

        <div class="row-span-2 h-56 md:h-full p-2 rounded shadow bg-gray-darker text-blue overflow-scroll">
          Output
        </div>

        <div class="row-span-3 h-56 md:h-full p-2 rounded shadow bg-gray-darker text-purple overflow-scroll">
          Terminal
        </div>
      </div>
    </div>
  );
}

export default Home;
