export const State = {
  YIELDED: 1,
  WAITING: 2,
  COMPLETE: 3,
};
Object.freeze(State);

export async function assemble(setComputerState, code) {
  setComputerState(State.WAITING);
}

export async function next(setComputerState, input) {
  setComputerState(State.COMPLETE);
}

export async function cancel(setComputerState) {
  setComputerState(State.COMPLETE);
}
