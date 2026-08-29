export abstract class Base {
  abstract run(): void;
}

export enum Mode {
  On,
  Off,
}

export function* generate(): Generator<number> {
  yield 1;
}

export async function fetchIt(): Promise<void> {}

export namespace outer {
  export namespace inner {
    export function deep(): void {}
  }
}
